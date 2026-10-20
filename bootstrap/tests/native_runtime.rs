//! The native runtime's AIL source (lane 122, ADR 0404-0409): its `@test`
//! functions pass under the interpreter and under the tree walker, so the
//! number text, strings and lists agree with the interpreter's own.

use std::path::Path;
use std::process::Command;

fn test_runtime(extra: &[&str]) {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../compiler/rt/rt.ail");
    let out = Command::new(env!("CARGO_BIN_EXE_ail"))
        .arg("test")
        .args(extra)
        .arg(&file)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "ail test failed:\n{text}{}", String::from_utf8_lossy(&out.stderr));
    assert!(text.contains(" 0 failed"), "{text}");
}

#[test]
fn runtime_tests_pass_on_the_bytecode_tier() {
    test_runtime(&[]);
}

#[test]
fn runtime_tests_pass_on_the_tree_walker() {
    test_runtime(&["--no-bytecode"]);
}
