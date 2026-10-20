//! The checker written in AIL (`compiler/check/`, ADR 0377) against this
//! one: on every example, case, mistake, package and `tools/edit`, the AIL
//! checker's diagnostics must equal `ail check --json`'s (rule, file, line,
//! column, in order), counting on both sides only the rules it reports and
//! the lines it checks so far. It prints those with each program's
//! diagnostics.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn ail() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_ail"));
    c.current_dir(root());
    c
}

fn with_ext(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == ext))
        .collect();
    v.sort();
    v
}

/// Every program of the corpus, relative to the repository root.
fn programs() -> Vec<String> {
    let r = root();
    let rel = |p: PathBuf| p.strip_prefix(&r).unwrap().to_string_lossy().into_owned();
    let mut out: Vec<String> = Vec::new();
    for dir in ["examples", "bootstrap/tests/cases", "bootstrap/tests/mistakes"] {
        out.extend(with_ext(&r.join(dir), "ail").into_iter().map(rel));
    }
    let mut pkgs: Vec<PathBuf> = fs::read_dir(r.join("bootstrap/tests/cases/packages"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    pkgs.sort();
    out.extend(pkgs.into_iter().map(rel));
    out.push("tools/edit".into());
    out
}

/// The string value of `"key":"..."` in a line of JSON (no escapes in keys).
fn str_field(line: &str, key: &str) -> String {
    let pat = format!("\"{key}\":\"");
    let Some(i) = line.find(&pat) else { return String::new() };
    let rest = &line[i + pat.len()..];
    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(n) = chars.next() {
                    out.push(n);
                }
            }
            '"' => break,
            c => out.push(c),
        }
    }
    out
}

fn num_field(line: &str, key: &str) -> usize {
    let pat = format!("\"{key}\":");
    let i = line.find(&pat).unwrap_or_else(|| panic!("no {key} in {line}"));
    line[i + pat.len()..].chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap()
}

/// (rule, file, line, column) of each diagnostic line.
type Key = (String, String, usize, usize);

fn key(line: &str) -> Key {
    (str_field(line, "rule"), str_field(line, "file"), num_field(line, "line"), num_field(line, "col"))
}

struct Section {
    diags: Vec<Key>,
    rules: Vec<String>,
    skips: Vec<(String, usize, usize)>,
}

#[test]
fn ail_checker_agrees_with_the_bootstrap() {
    let dir = std::env::temp_dir().join(format!("ail-self-hosted-check-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut dumps: Vec<(String, PathBuf)> = Vec::new();
    for (i, prog) in programs().iter().enumerate() {
        let out = ail().args(["api", "--ast", "--program", prog]).output().unwrap();
        let text = String::from_utf8_lossy(&out.stdout);
        // A program the front end rejects never reaches the checker.
        if !out.status.success() || text.lines().count() != 1 {
            continue;
        }
        let path = dir.join(format!("{i}.json"));
        fs::write(&path, text.as_bytes()).unwrap();
        dumps.push((prog.clone(), path));
    }
    // The AIL checker, in four batches at once.
    let chunks: Vec<Vec<(String, PathBuf)>> = dumps.chunks(dumps.len().div_ceil(4)).map(|c| c.to_vec()).collect();
    let outputs: Vec<String> = std::thread::scope(|s| {
        let handles: Vec<_> = chunks
            .iter()
            .map(|chunk| {
                s.spawn(move || {
                    let mut cmd = ail();
                    cmd.args(["run", "compiler", "check"]);
                    // Two paths at least, so every section has its header.
                    for (_, p) in chunk {
                        cmd.arg(p);
                    }
                    if chunk.len() == 1 {
                        cmd.arg(&chunk[0].1);
                    }
                    let out = cmd.output().unwrap();
                    assert!(
                        out.status.success() || out.status.code() == Some(1),
                        "the AIL checker failed: {}",
                        String::from_utf8_lossy(&out.stderr)
                    );
                    String::from_utf8(out.stdout).unwrap()
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut sections: std::collections::HashMap<String, Section> = Default::default();
    let mut current = String::new();
    let (mut checked, mut bodies) = (0usize, 0usize);
    for line in outputs.iter().flat_map(|o| o.lines()) {
        if let Some(b) = line.strip_prefix("bodies ") {
            let mut w = b.split(' ').map(|n| n.parse::<usize>().unwrap());
            checked += w.next().unwrap();
            bodies += w.next().unwrap();
            continue;
        }
        if let Some(p) = line.strip_prefix("== ") {
            current = p.to_string();
            sections.insert(current.clone(), Section { diags: Vec::new(), rules: Vec::new(), skips: Vec::new() });
        } else if let Some(r) = line.strip_prefix("rules ") {
            sections.get_mut(&current).unwrap().rules = r.split(' ').map(String::from).collect();
        } else if let Some(s) = line.strip_prefix("skip ") {
            let mut w = s.rsplitn(3, ' ');
            let last = w.next().unwrap().parse().unwrap();
            let first = w.next().unwrap().parse().unwrap();
            let file = w.next().unwrap_or("").to_string();
            sections.get_mut(&current).unwrap().skips.push((file, first, last));
        } else if line.starts_with('{') {
            sections.get_mut(&current).unwrap().diags.push(key(line));
        }
    }
    let mut agree = 0;
    let mut failures = Vec::new();
    for (prog, path) in &dumps {
        let sec = sections.get(&path.to_string_lossy().into_owned()).expect("a section per dump");
        let out = ail().args(["check", "--json", prog]).output().unwrap();
        let counted = |(rule, file, line, _): &Key| {
            sec.rules.contains(rule) && !sec.skips.iter().any(|(f, a, b)| f == file && (*a..=*b).contains(line))
        };
        let want: Vec<Key> =
            String::from_utf8_lossy(&out.stderr).lines().filter(|l| l.starts_with('{')).map(key).filter(counted).collect();
        let got: Vec<Key> = sec.diags.iter().filter(|k| counted(k)).cloned().collect();
        if want == got {
            agree += 1;
        } else {
            failures.push(format!("{prog}\n  ail check: {want:?}\n  AIL:       {got:?}"));
        }
    }
    let _ = fs::remove_dir_all(&dir);
    println!("self-hosted checker agrees on {agree} of {} programs", dumps.len());
    println!("it types {checked} of {bodies} function bodies");
    assert!(failures.is_empty(), "{} disagreements:\n{}", failures.len(), failures.join("\n"));
}
