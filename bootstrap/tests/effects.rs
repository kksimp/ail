//! End-to-end tests of `ail effects` and `--deny` (eff.infer.6, eff.cap.1-2).

use std::process::Command;

fn ail(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ail"))
        .args(args)
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/.."))
        .output()
        .unwrap()
}

#[test]
fn effects_of_commands_example() {
    let out = ail(&["effects", "examples/commands.ail"]);
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), include_str!("effects/commands.effects"));
}

#[test]
fn deny_all_runs_a_program_without_capabilities() {
    let out = ail(&["run", "--deny", "all", "examples/hello.ail"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let out = ail(&["check", "--deny", "fs,proc", "examples/commands.ail"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn deny_rejects_an_unknown_capability() {
    let out = ail(&["run", "--deny", "disk", "examples/hello.ail"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("eff.cap.2"));
}
