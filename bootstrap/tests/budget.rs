//! `--budget N [--from K]` on every command (budget.out.1-6, ADR 0394).

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

mod common;

fn ail(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ail")).args(args).output().unwrap()
}

fn file(name: &str, src: &str) -> String {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("budget");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, src).unwrap();
    common::shown(&path)
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// Two warnings (holes) before one error, in source order.
const MIXED: &str = "fn f() int\n    todo\n\nfn g() int\n    todo\n\nfn main()\n    print(nope)\n";

/// budget.out.1: output that fits is unchanged, and the exit status is kept.
#[test]
fn fitting_output_is_unchanged() {
    let f = file("fits.ail", MIXED);
    let plain = ail(&["check", &f]);
    let budgeted = ail(&["check", "--budget", "100000", &f]);
    assert_eq!(text(&plain.stderr), text(&budgeted.stderr));
    assert_eq!(plain.status.code(), Some(2));
    assert_eq!(budgeted.status.code(), Some(2));
}

/// budget.out.3-4: the error comes before the warnings, and the note counts
/// the rest and continues with `--from`.
#[test]
fn errors_first_then_a_note() {
    let f = file("mixed.ail", MIXED);
    let all = text(&ail(&["check", &f]).stderr);
    assert!(all.find("warning[").unwrap() < all.find("error[").unwrap(), "{all}");
    let cut = text(&ail(&["check", "--budget", "10", &f]).stderr);
    assert!(cut.contains("error["), "{cut}");
    assert!(!cut.contains("warning["), "{cut}");
    assert!(cut.ends_with("not shown: 2 warnings; continue with --from 1]\n"), "{cut}");
    let next = text(&ail(&["check", "--budget", "10", "--from", "1", &f]).stderr);
    assert!(next.starts_with(&format!("{f}:2:")), "{next}");
    assert!(!next.contains("error["), "{next}");
}

/// budget.out.4: the JSON note.
#[test]
fn json_note() {
    let f = file("json.ail", MIXED);
    let cut = text(&ail(&["check", "--json", "--budget", "10", &f]).stderr);
    let last = cut.lines().last().unwrap();
    assert!(last.starts_with("{\"budget\":10,\"from\":0,\"shown\":1,\"total\":3,\"tokens\":"), "{last}");
    assert!(last.ends_with("\"omitted\":{\"warning\":2},\"next\":1}"), "{last}");
}

/// budget.out.3: a test summary is always shown; failures come first.
#[test]
fn tests_keep_their_summary() {
    let f = file("t.ail", "@test fn a()\n    assert(1 == 1)\n\n@test fn b()\n    assert(1 == 2)\n");
    let out = ail(&["test", "--budget", "5", &f]);
    let o = text(&out.stdout);
    assert!(o.starts_with("FAIL b:"), "{o}");
    assert!(o.contains("1 passed, 1 failed\n"), "{o}");
    assert!(o.ends_with("not shown: 1 passed test; continue with --from 1]\n"), "{o}");
    assert_eq!(out.status.code(), Some(1));
}

/// budget.out.1, std.env.1: `ail run` passes the program's own arguments on,
/// and a `--budget` after FILE belongs to the program.
#[test]
fn run_keeps_program_arguments() {
    let f = file("args.ail", "use std.env\n\nfn main()\n    for a in env.args()\n        print(a)\n");
    let out = ail(&["run", "--budget", "1", &f, "x", "--budget", "y"]);
    let o = text(&out.stdout);
    assert!(o.starts_with("x\n[budget 1: records 1-1 of 3"), "{o}");
}

/// budget.out.5: a bare `--budget` takes the command's default.
#[test]
fn bare_budget_is_the_default() {
    let f = file("bare.ail", MIXED);
    assert_eq!(text(&ail(&["check", "--budget", &f]).stderr), text(&ail(&["check", &f]).stderr));
    let bad = ail(&["check", "--from", "x", &f]);
    assert!(text(&bad.stderr).contains("budget.out.4"));
    assert_eq!(bad.status.code(), Some(2));
}
