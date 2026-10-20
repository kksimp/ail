//! The mistakes corpus (gram.help.2, ADR 0262): each `tests/mistakes/NAME.ail`
//! is a plausible error of an AI writing AIL as if it were Python, Rust, Go or
//! TypeScript. For each:
//!
//! - `NAME.rules`: the rule IDs of the diagnostics, in report order. The first
//!   names the actual mistake and every following one is a mistake of its own,
//!   so a file with one slip lists one rule (gram.rec.1).
//! - every error carries a `help` an agent can act on (gram.help.1).
//! - `NAME.fixed`, when it exists: the text `ail fix` must produce. It must
//!   check clean and be stable. Without one, `ail fix` must not claim success.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn ail() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ail"))
}

fn corpus() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/mistakes");
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "ail"))
        .collect();
    files.sort();
    files
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ail-mistakes-{}-{name}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// The error diagnostics (JSON lines) that `ail check` printed for `path`.
fn errors(path: &Path) -> Vec<String> {
    let out = ail().args(["check", "--json"]).arg(path).output().unwrap();
    String::from_utf8_lossy(&out.stderr)
        .lines()
        .filter(|l| l.contains("\"severity\":\"error\""))
        .map(str::to_string)
        .collect()
}

fn rule_of(line: &str) -> String {
    line.split("\"rule\":\"").nth(1).unwrap().split('"').next().unwrap().to_string()
}

#[test]
fn each_mistake_is_named_once_with_help() {
    let mut failures = Vec::new();
    for path in corpus() {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let got = errors(&path);
        let rules: Vec<String> = got.iter().map(|l| rule_of(l)).collect();
        let want: Vec<String> =
            fs::read_to_string(path.with_extension("rules")).unwrap().lines().map(str::to_string).collect();
        if rules != want {
            failures.push(format!("{name}: rules differ\n  expected {want:?}\n  got      {rules:?}"));
        }
        for line in &got {
            if !line.contains("\"help\":\"") {
                failures.push(format!("{name}: a diagnostic has no help\n  {line}"));
            }
            if rule_of(line) == "bootstrap.unsupported" {
                failures.push(format!("{name}: a mistake is reported as unsupported\n  {line}"));
            }
        }
    }
    assert!(failures.is_empty(), "{} problem(s):\n\n{}", failures.len(), failures.join("\n\n"));
}

#[test]
fn fixes_produce_the_recorded_text() {
    let mut failures = Vec::new();
    for path in corpus() {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let tmp = scratch(&name);
        let work = tmp.join(format!("{name}.ail"));
        fs::copy(&path, &work).unwrap();
        let out = ail().arg("fix").arg(&work).output().unwrap();
        let fixed = path.with_extension("fixed");
        let check = ail().arg("check").arg(&work).output().unwrap();
        if fixed.exists() {
            let want = fs::read_to_string(&fixed).unwrap();
            let got = fs::read_to_string(&work).unwrap();
            if got != want {
                failures.push(format!("{name}: fixed text differs\n--- expected\n{want}--- actual\n{got}"));
            }
            if check.status.code() != Some(0) {
                failures
                    .push(format!("{name}: the fixed file doesn't check\n{}", String::from_utf8_lossy(&check.stderr)));
            }
            let again = ail().args(["fix", "--json"]).arg(&work).output().unwrap();
            if !String::from_utf8_lossy(&again.stdout).contains("\"applied\":0,\"rounds\":0") {
                failures.push(format!("{name}: a second `ail fix` still changes the file"));
            }
        } else if check.status.code() == Some(0) || out.status.code() == Some(0) {
            failures.push(format!("{name}: has no .fixed file, but `ail fix` resolved it; record the result"));
        }
        fs::remove_dir_all(&tmp).unwrap();
    }
    assert!(failures.is_empty(), "{} problem(s):\n\n{}", failures.len(), failures.join("\n\n"));
}

/// The targets of ADR 0262: at least 60 mistakes, and `ail fix` resolves at
/// least 60% of them.
#[test]
fn the_corpus_meets_its_targets() {
    let files = corpus();
    let fixed = files.iter().filter(|p| p.with_extension("fixed").exists()).count();
    assert!(files.len() >= 60, "the corpus has {} programs", files.len());
    assert!(fixed * 100 >= files.len() * 60, "`ail fix` resolves only {fixed} of {}", files.len());
}
