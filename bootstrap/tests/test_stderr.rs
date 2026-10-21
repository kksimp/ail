//! `ail test` captures a test's standard error like its standard output
//! (test.run.2, lane 215): a passing test's `io.eprint` is not shown, and a
//! failing test's is shown under its `FAIL` line, marked `!`. `ail run`
//! still writes standard error straight to the terminal.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn ail() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ail"))
}

fn program(name: &str, src: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("test_stderr");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.ail"));
    fs::write(&path, src).unwrap();
    path
}

const SRC: &str = r#"use std.io
@test fn quiet_pass()
    io.eprint("noise from a passing test")

@test fn loud_fail()
    io.eprint("boom on stderr")
    assert(1 == 2)
"#;

#[test]
fn failing_test_shows_its_stderr_and_passing_test_hides_it() {
    let path = program("captures", SRC);
    let out = ail().arg("test").arg(&path).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{stdout}");
    assert!(stdout.contains("  ! boom on stderr"), "{stdout}");
    assert!(!stdout.contains("noise from a passing test"), "{stdout}");
    assert!(!String::from_utf8_lossy(&out.stderr).contains("noise from a passing test"));
}

#[test]
fn ail_run_still_writes_standard_error_directly() {
    let path = program("run", "use std.io\nfn main()\n    io.eprint(\"direct\")\n");
    let out = ail().arg("run").arg(&path).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stderr), "direct\n");
}
