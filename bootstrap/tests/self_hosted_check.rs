//! The checker written in AIL (`compiler/check/`, ADR 0377) against this
//! one: on every example, case, mistake, package and `tools/edit`, the AIL
//! checker's diagnostics must equal `ail check --json`'s (rule, file, line,
//! column, in order), counting on both sides only the rules it reports and
//! the lines it checks so far. It prints those with each program's
//! diagnostics. With nothing skipped (slice 4), the two must also agree on
//! every diagnostic except the package manifest's capability checks, which
//! `ail` makes before the checker runs and the AST dump doesn't carry.
//!
//! There is one checker, `compiler/check/` (ADR 0865): it may report MORE than
//! this one, but only rules listed in `checker_known_new.txt`, and never LESS.

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
    for dir in ["examples", "tests/cases", "tests/mistakes", "tools/perfbench/programs"] {
        out.extend(with_ext(&r.join(dir), "ail").into_iter().map(rel));
    }
    let mut pkgs: Vec<PathBuf> = fs::read_dir(r.join("tests/cases/packages"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    pkgs.sort();
    out.extend(pkgs.into_iter().map(rel));
    // The agent corpus: one directory per program.
    let mut agent: Vec<PathBuf> = fs::read_dir(r.join("tools/tokbench/corpus/programs"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    agent.sort();
    out.extend(agent.into_iter().map(rel));
    out.push("tools/edit".into());
    out.push("compiler".into());
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

/// eff.cap.4-5 come from `bootstrap/src/packages.rs`, before checking.
const MANIFEST_RULES: &[&str] = &["eff.cap.4", "eff.cap.5"];

fn key(line: &str) -> Key {
    (str_field(line, "rule"), str_field(line, "file"), num_field(line, "line"), num_field(line, "col"))
}

/// A diagnostic's fixes, from `"fixes":[` to the end of its JSON line: the
/// settled edits `ail check` and the AIL checker (through `fix/resolve.ail`) write.
fn fixes_of(line: &str) -> String {
    line.find("\"fixes\":[").map(|i| line[i..].to_string()).unwrap_or_default()
}

/// The rules the AIL checker may report beyond the Rust one: one per line,
/// `#` starts a comment.
fn known_new() -> Vec<String> {
    let text = fs::read_to_string(root().join("tests/checker_known_new.txt")).unwrap_or_default();
    text.lines().map(|l| l.split('#').next().unwrap().trim().to_string()).filter(|l| !l.is_empty()).collect()
}

/// Whether `got` holds every diagnostic of `want`, in order, and nothing else
/// except diagnostics of the `known_new` rules.
fn covers(want: &[Key], got: &[Key], known_new: &[String]) -> bool {
    let mut next = 0;
    for g in got {
        if want.get(next) == Some(g) {
            next += 1;
        } else if !known_new.contains(&g.0) {
            return false;
        }
    }
    next == want.len()
}

struct Section {
    diags: Vec<Key>,
    /// Each diagnostic's fixes, beside `diags`.
    fixes: Vec<String>,
    /// Each diagnostic's message, beside `diags`.
    messages: Vec<String>,
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
            sections.insert(current.clone(), Section { diags: Vec::new(), fixes: Vec::new(), messages: Vec::new(), rules: Vec::new(), skips: Vec::new() });
        } else if let Some(r) = line.strip_prefix("rules ") {
            sections.get_mut(&current).unwrap().rules = r.split(' ').map(String::from).collect();
        } else if let Some(s) = line.strip_prefix("skip ") {
            let mut w = s.rsplitn(3, ' ');
            let last = w.next().unwrap().parse().unwrap();
            let first = w.next().unwrap().parse().unwrap();
            let file = w.next().unwrap_or("").to_string();
            sections.get_mut(&current).unwrap().skips.push((file, first, last));
        } else if line.starts_with('{') {
            let sec = sections.get_mut(&current).unwrap();
            sec.diags.push(key(line));
            sec.messages.push(str_field(line, "message"));
            sec.fixes.push(fixes_of(line));
        }
    }
    let known = known_new();
    let mut agree = 0;
    let mut failures = Vec::new();
    let (mut agree_all, mut total_diags, mut matched_diags, mut same_text) = (0usize, 0usize, 0usize, 0usize);
    let mut differ_all: Vec<String> = Vec::new();
    let (mut fixes_total, mut same_fixes) = (0usize, 0usize);
    let mut fix_diffs: Vec<String> = Vec::new();
    for (prog, path) in &dumps {
        let sec = sections.get(&path.to_string_lossy().into_owned()).expect("a section per dump");
        let out = ail().args(["check", "--json", prog]).output().unwrap();
        let counted = |(rule, file, line, _): &Key| {
            sec.rules.contains(rule) && !sec.skips.iter().any(|(f, a, b)| f == file && (*a..=*b).contains(line))
        };
        let want: Vec<Key> =
            String::from_utf8_lossy(&out.stderr).lines().filter(|l| l.starts_with('{')).map(key).filter(counted).collect();
        let got: Vec<Key> = sec.diags.iter().filter(|k| counted(k)).cloned().collect();
        if covers(&want, &got, &known) {
            agree += 1;
        } else {
            failures.push(format!("{prog}\n  ail check: {want:?}\n  AIL:       {got:?}"));
        }
        // The slice 4 report: every rule and line, nothing skipped.
        let lines: Vec<&str> = std::str::from_utf8(&out.stderr)
            .unwrap()
            .lines()
            .filter(|l| l.starts_with('{') && !MANIFEST_RULES.contains(&str_field(l, "rule").as_str()))
            .collect();
        let all: Vec<Key> = lines.iter().map(|l| key(l)).collect();
        total_diags += all.len();
        matched_diags += all.iter().filter(|k| sec.diags.contains(k)).count();
        if covers(&all, &sec.diags, &known) {
            agree_all += 1;
            // Wording isn't gated: the report counts the messages that match.
            same_text += lines.iter().zip(&sec.messages).filter(|(l, m)| str_field(l, "message") == **m).count();
            for (l, got) in lines.iter().zip(&sec.fixes) {
                fixes_total += usize::from(l.contains("\"fixes\":"));
                let want = fixes_of(l);
                if want == *got {
                    same_fixes += usize::from(!want.is_empty());
                } else {
                    fix_diffs.push(format!("{prog}: {} line {}\n  ail check: {want:?}\n  AIL:       {got:?}", str_field(l, "rule"), num_field(l, "line")));
                }
            }
        } else {
            differ_all.push(format!("{prog}\n  ail check: {all:?}\n  AIL:       {:?}", sec.diags));
        }
    }
    let _ = fs::remove_dir_all(&dir);
    println!("self-hosted checker agrees on {agree} of {} programs", dumps.len());
    println!("it types {checked} of {bodies} function bodies");
    println!(
        "with nothing skipped, it agrees on {agree_all} of {} programs and reports {matched_diags} of {total_diags} diagnostics",
        dumps.len()
    );
    println!("{same_text} of those {total_diags} diagnostics have the same message");
    println!("{same_fixes} of {fixes_total} diagnostics with fixes have the same fixes");
    assert!(fix_diffs.is_empty(), "{} fix disagreements:\n{}", fix_diffs.len(), fix_diffs.join("\n"));
    assert!(failures.is_empty(), "{} disagreements:\n{}", failures.len(), failures.join("\n"));
    assert!(
        differ_all.is_empty(),
        "{} disagreements with nothing skipped:\n{}",
        differ_all.len(),
        differ_all.join("\n")
    );
}
