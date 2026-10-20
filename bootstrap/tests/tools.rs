//! The tools written in AIL (`tools/`) are AIL programs too: their checks
//! and `@test` suites run with the bootstrap's own tests.

use std::path::Path;
use std::process::Command;

fn ail() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ail"))
}

fn tool(rel: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(rel).display().to_string()
}

/// `tools/edit/edit.ail` (ADR 0180) checks without diagnostics and passes
/// its headless `@test` suite.
#[test]
fn editor_checks_clean_and_passes_its_tests() {
    let path = tool("tools/edit/edit.ail");
    let check = ail().args(["check", &path]).output().unwrap();
    let stderr = String::from_utf8_lossy(&check.stderr);
    assert!(check.status.success() && stderr.is_empty(), "ail check {path}:\n{stderr}");
    let test = ail().args(["test", &path]).output().unwrap();
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test.status.success() && stdout.contains(" 0 failed"), "ail test {path}:\n{stdout}");
}
