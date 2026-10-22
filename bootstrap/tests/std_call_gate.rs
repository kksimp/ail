//! Every public function and method of the plain-AIL std modules
//! (`bootstrap/src/std/**/*.ail`) is called by a test, a corpus program or an
//! example (std.use.1). A new `pub fn` with no call fails here; a function that
//! can't be called deterministically goes on `UNTESTABLE` with its reason.

use std::fs;
use std::path::{Path, PathBuf};

/// (file, function, why no case calls it). An entry that is now called fails
/// too, so the list only shrinks.
const UNTESTABLE: &[(&str, &str, &str)] = &[
    ("time/date.ail", "now_local", "its value is the clock in the machine's zone"),
    ("net/http.ail", "serve_dir", "it serves until the program stops, so a case cannot end it"),
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn files(dir: &Path, exts: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            files(&p, exts, out);
        } else if p.extension().and_then(|x| x.to_str()).is_some_and(|x| exts.contains(&x)) {
            out.push(p);
        }
    }
    out.sort();
}

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Whether `text` holds `needle` not preceded by a word character.
fn has_call(text: &str, needle: &str) -> bool {
    text.match_indices(needle).any(|(i, _)| !text[..i].chars().next_back().is_some_and(is_word))
}

#[test]
fn every_public_std_function_is_called_by_a_test() {
    let r = root();
    let mut callers = String::new();
    for (dir, exts) in [
        (r.join("tests"), &["ail", "rs", "test"][..]),
        (r.join("../tools/tokbench/corpus"), &["ail"][..]),
        (r.join("../examples"), &["ail"][..]),
    ] {
        let mut fs_ = Vec::new();
        files(&dir, exts, &mut fs_);
        for f in fs_ {
            // This file lists names as strings; it must not count as a caller.
            if f.file_name().is_some_and(|n| n == "std_call_gate.rs") {
                continue;
            }
            if let Ok(t) = fs::read_to_string(&f) {
                callers.push_str(&t);
                callers.push('\n');
            }
        }
    }
    let mut std_files = Vec::new();
    files(&r.join("src/std"), &["ail"], &mut std_files);
    let (mut missing, mut stale, mut seen) = (Vec::new(), Vec::new(), 0);
    for f in std_files {
        let rel = f.strip_prefix(r.join("src/std")).unwrap().to_string_lossy().replace('\\', "/");
        let stem = f.file_stem().unwrap().to_string_lossy().to_string();
        let parent = f.parent().unwrap().file_name().unwrap().to_string_lossy().to_string();
        for line in fs::read_to_string(&f).unwrap().lines() {
            let Some(rest) = line.strip_prefix("pub fn ") else { continue };
            let head: String = rest.chars().take_while(|c| is_word(*c) || *c == '.').collect();
            let (name, calls): (&str, Vec<String>) = match head.split_once('.') {
                Some((_, m)) => (m, vec![format!(".{m}(")]),
                None => (&head, vec![format!("{stem}.{head}("), format!("{parent}.{head}(")]),
            };
            seen += 1;
            let called =
                calls.iter().any(|c| has_call(&callers, c) || (c.starts_with('.') && callers.contains(c.as_str())));
            let listed = UNTESTABLE.iter().any(|(file, n, _)| *file == rel && *n == name);
            if !called && !listed {
                missing.push(format!("{rel}: {name}"));
            }
            if called && listed {
                stale.push(format!("{rel}: {name}"));
            }
        }
    }
    assert!(seen > 50, "found only {seen} public std functions: the scan is broken");
    assert!(
        missing.is_empty(),
        "public std functions no test calls (add a case in bootstrap/tests/cases, or an UNTESTABLE entry with the reason):\n{}",
        missing.join("\n")
    );
    assert!(stale.is_empty(), "UNTESTABLE entries that a test now calls (remove them):\n{}", stale.join("\n"));
}
