//! `ail fmt` end to end (ADR 0170): formatting doesn't change what a program
//! prints, and `--check` and `--stdout` behave as documented.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn ail() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ail"))
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("fmt");
    fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

#[test]
fn formatting_examples_keeps_their_output() {
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples");
    let mut files: Vec<PathBuf> = fs::read_dir(&examples).unwrap().map(|e| e.unwrap().path()).collect();
    files.sort();
    let mut checked = 0;
    for f in files.iter().filter(|p| p.extension().is_some_and(|x| x == "ail")) {
        let formatted = ail().args(["fmt", "--stdout"]).arg(f).output().unwrap();
        assert!(formatted.status.success(), "{}: {}", f.display(), String::from_utf8_lossy(&formatted.stderr));
        // ADR 0218: corpus programs find their fixtures and `ail` from their own
        // path, so a copy elsewhere can't run. Their layout must already be canonical.
        let source = fs::read(f).unwrap();
        if String::from_utf8_lossy(&source).contains("data_dir()") || source.windows(7).any(|w| w == b"/../ail") {
            assert_eq!(formatted.stdout, source, "{}: not in canonical layout", f.display());
            checked += 1;
            continue;
        }
        let copy = scratch(f.file_name().unwrap().to_str().unwrap());
        fs::write(&copy, &formatted.stdout).unwrap();
        let before = ail().arg("run").arg(f).output().unwrap();
        let after = ail().arg("run").arg(&copy).output().unwrap();
        assert_eq!(before.status.code(), after.status.code(), "{}", f.display());
        assert_eq!(before.stdout, after.stdout, "{}: output changed after formatting", f.display());
        checked += 1;
    }
    assert!(checked >= 6);
}

#[test]
fn check_reports_a_diff_and_fmt_rewrites() {
    let file = scratch("messy.ail");
    fs::write(&file, "fn main()\n    x:=[\n        1,\n        2]\n    print( x )\n").unwrap();
    let check = ail().args(["fmt", "--check"]).arg(&file).output().unwrap();
    assert_eq!(check.status.code(), Some(1));
    let diff = String::from_utf8_lossy(&check.stdout);
    assert!(diff.contains("\n-    print( x )\n") && diff.contains("\n+    print(x)\n"), "{diff}");
    assert!(diff.contains("+    x := [1, 2]\n"), "{diff}");
    let stdout = ail().args(["fmt", "--stdout"]).arg(&file).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&stdout.stdout), "fn main()\n    x := [1, 2]\n    print(x)\n");
    assert_eq!(ail().arg("fmt").arg(&file).status().unwrap().code(), Some(0));
    assert_eq!(fs::read_to_string(&file).unwrap(), "fn main()\n    x := [1, 2]\n    print(x)\n");
    assert_eq!(ail().args(["fmt", "--check"]).arg(&file).status().unwrap().code(), Some(0));
}

#[test]
fn a_file_with_errors_is_left_alone() {
    let file = scratch("broken.ail");
    fs::write(&file, "fn main()\n   print(1)\n").unwrap();
    let out = ail().arg("fmt").arg(&file).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("lex.block.2"));
    assert_eq!(fs::read_to_string(&file).unwrap(), "fn main()\n   print(1)\n");
}
