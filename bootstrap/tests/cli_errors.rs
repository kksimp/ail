//! Diagnostics that the case harness cannot check (it rejects exit status 2 and
//! gives a program a readable stdin and a working directory): `eff.cap.3`,
//! `std.env.3`, `std.io.1` and `std.io.2`, each by running the binary as a
//! process. Every test uses its own temporary directory and passes `AIL_BINARY`
//! (ADR 0814), so the tests can run in parallel.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn ail() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ail"));
    cmd.env("AIL_BINARY", env!("CARGO_BIN_EXE_ail"));
    cmd
}

/// A fresh directory for one test, named by the process and the test.
fn scratch(test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ail-cli-errors-{}-{test}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn program(dir: &PathBuf, src: &str) -> PathBuf {
    let path = dir.join("main.ail");
    fs::write(&path, src).unwrap();
    path
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// eff.cap.3: `--deny` takes capabilities, and `io` is an effect.
#[test]
fn deny_names_an_effect() {
    let dir = scratch("deny");
    let main = program(&dir, "fn main()\n    print(1)\n");
    let out = ail().args(["run", "--deny", "io"]).arg(&main).output().unwrap();
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    assert!(stderr(&out).contains("(eff.cap.3)"), "{}", stderr(&out));
    let _ = fs::remove_dir_all(&dir);
}

/// std.env.3: `env.cwd()` traps when the working directory was removed.
#[cfg(unix)]
#[test]
fn cwd_removed() {
    use std::os::unix::process::CommandExt;
    let dir = scratch("cwd");
    let main = program(&dir, "use std.env\n\nfn main()\n    print(env.cwd())\n");
    let gone = dir.join("gone");
    fs::create_dir_all(&gone).unwrap();
    let mut cmd = ail();
    cmd.arg("run").arg(&main).current_dir(&gone);
    let target = gone.clone();
    // SAFETY: the closure only calls rmdir in the child, between fork and exec.
    unsafe {
        cmd.pre_exec(move || fs::remove_dir(&target));
    }
    let out = cmd.output().unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stderr(&out).contains("trap[std.env.3]"), "{}", stderr(&out));
    let _ = fs::remove_dir_all(&dir);
}

/// std.io.1 and std.io.2: reading standard input fails when it is a directory.
#[cfg(unix)]
fn stdin_fails(test: &str, body: &str, rule: &str) {
    let dir = scratch(test);
    let main = program(&dir, &format!("use std.io\n\nfn main()\n    {body}\n"));
    let stdin = fs::File::open(&dir).unwrap();
    let out = ail().arg("run").arg(&main).stdin(stdin).output().unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stderr(&out).contains(&format!("trap[{rule}]")), "{}", stderr(&out));
    let _ = fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[test]
fn read_line_fails() {
    stdin_fails("read_line", "print(io.read_line())", "std.io.1");
}

#[cfg(unix)]
#[test]
fn read_all_fails() {
    stdin_fails("read_all", "print(io.read_all().len())", "std.io.2");
}
