//! `ail test --native` (native.test.1-4): the runner builds each `@test`
//! function natively, runs it, and compares it with the interpreter.

use std::path::Path;
use std::process::Command;

fn run(args: &[&str]) -> (i32, String) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let out = Command::new(env!("CARGO_BIN_EXE_ail"))
        .current_dir(&root)
        .args(["test", "--native"])
        .args(args)
        .output()
        .unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap())
}

// native.test.1, native.test.2, native.test.3
#[test]
fn passing_and_failing_tests_match_the_interpreter() {
    let (code, out) = run(&["bootstrap/tests/native_test/mixed.ail"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("match bootstrap/tests/native_test/mixed.ail::adds (native pass, interpreter pass)"), "{out}");
    assert!(out.contains("match bootstrap/tests/native_test/mixed.ail::fails_on_purpose (native fail, interpreter fail)"), "{out}");
    assert!(out.contains("assertion failed at line 14"), "the failure names the original line: {out}");
    assert!(out.contains("not-native bootstrap/tests/native_test/mixed.ail::uses_defer"), "{out}");
    assert!(out.contains("skipped bootstrap/tests/native_test/mixed.ail::holds_for_all"), "{out}");
}

// native.test.4
#[test]
fn json_lines_end_with_a_summary() {
    let (code, out) = run(&["--json", "bootstrap/tests/native_test/mixed.ail"]);
    assert_eq!(code, 0, "{out}");
    let last = out.lines().last().unwrap();
    assert_eq!(last, "{\"summary\":true,\"match\":2,\"differ\":0,\"not-native\":1,\"skipped\":1}");
}
