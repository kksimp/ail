//! End-to-end tests: run the real `ail` binary on example programs.
//!
//! - `examples/NAME.ail` + `NAME.out`: must run and print exactly `NAME.out`.
//! - `bootstrap/tests/cases/NAME.ail` + `NAME.out`: same.
//! - `NAME.ail` + `NAME.rules`: the diagnostics' spec rule IDs (in report
//!   order) must equal the file's lines. The program must be rejected (exit 2)
//!   if any of them is an error; with only warnings, it is accepted (exit 0).
//! - `NAME.ail` + `NAME.trap`: must trap (exit 1) with the rule ID in the
//!   file, after printing exactly `NAME.out` if that exists.
//! - `NAME.ail` + `NAME.err`: `main` must fail (exit 1) with exactly this
//!   error report on stderr, where `$FILE` stands for the program's path,
//!   after printing exactly `NAME.out` if that exists.
//! - `bootstrap/tests/cases/packages/NAME/`: a package (mod.unit.5), run as
//!   `ail run NAME`, with the same expectations in `NAME/expected.out`,
//!   `expected.rules`, `expected.trap` or `expected.err`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn ail() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ail"))
}

fn ail_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "ail"))
        .collect();
    files.sort();
    files
}

fn check_case(path: &Path) -> Result<(), String> {
    check_run(path, &|ext| path.with_extension(ext))
}

/// Run `path`, a file or a package directory, against the expectation
/// files that `expect` names by extension.
fn check_run(path: &Path, expect: &dyn Fn(&str) -> PathBuf) -> Result<(), String> {
    let out_file = expect("out");
    let rules_file = expect("rules");
    let trap_file = expect("trap");
    let err_file = expect("err");
    let name = path.display();
    if err_file.exists() {
        let out = ail().arg("run").arg(path).output().unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        let want = fs::read_to_string(&err_file).unwrap().replace("$FILE", &path.display().to_string());
        if out.status.code() != Some(1) || stderr != want {
            return Err(format!(
                "{name}: expected exit 1 and this report\n--- expected\n{want}--- actual (exit {:?})\n{stderr}",
                out.status.code()
            ));
        }
        let expected = fs::read_to_string(&out_file).unwrap_or_default();
        if stdout != expected {
            return Err(format!("{name}: output differs\n--- expected\n{expected}--- actual\n{stdout}"));
        }
        Ok(())
    } else if trap_file.exists() {
        let out = ail().arg("run").arg(path).output().unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        let rule = fs::read_to_string(&trap_file).unwrap().trim().to_string();
        if out.status.code() != Some(1) || !stderr.contains(&format!("trap[{rule}]")) {
            return Err(format!("{name}: expected trap[{rule}] (exit 1), got {:?}\n{stderr}", out.status.code()));
        }
        let expected = fs::read_to_string(&out_file).unwrap_or_default();
        if stdout != expected {
            return Err(format!("{name}: output differs\n--- expected\n{expected}--- actual\n{stdout}"));
        }
        Ok(())
    } else if out_file.exists() {
        let out = ail().arg("run").arg(path).output().unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        let expected = fs::read_to_string(&out_file).unwrap();
        if !out.status.success() {
            return Err(format!("{name}: exit {:?}\n{}", out.status.code(), String::from_utf8_lossy(&out.stderr)));
        }
        if stdout != expected {
            return Err(format!("{name}: output differs\n--- expected\n{expected}--- actual\n{stdout}"));
        }
        Ok(())
    } else if rules_file.exists() {
        let out = ail().args(["check", "--json"]).arg(path).output().unwrap();
        let stderr = String::from_utf8_lossy(&out.stderr);
        let want_exit = if stderr.contains("\"severity\":\"error\"") { 2 } else { 0 };
        if out.status.code() != Some(want_exit) {
            return Err(format!("{name}: expected exit {want_exit}, got {:?}\n{stderr}", out.status.code()));
        }
        let got: Vec<String> = stderr
            .lines()
            .filter_map(|l| l.split("\"rule\":\"").nth(1))
            .map(|r| r.split('"').next().unwrap().to_string())
            .collect();
        let want: Vec<String> = fs::read_to_string(&rules_file).unwrap().lines().map(str::to_string).collect();
        if got != want {
            return Err(format!("{name}: rules differ\n  expected {want:?}\n  got      {got:?}\n{stderr}"));
        }
        Ok(())
    } else {
        Err(format!("{name}: needs a .out, .rules, .trap or .err file"))
    }
}

fn run_dir(dir: &Path) {
    let failures: Vec<String> = ail_files(dir).iter().filter_map(|p| check_case(p).err()).collect();
    assert!(failures.is_empty(), "{} case(s) failed:\n\n{}", failures.len(), failures.join("\n\n"));
}

#[test]
fn examples() {
    run_dir(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples"));
}

#[test]
fn cases() {
    run_dir(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases"));
}

#[test]
fn packages() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases/packages");
    let mut dirs: Vec<PathBuf> =
        fs::read_dir(&root).unwrap().map(|e| e.unwrap().path()).filter(|p| p.is_dir()).collect();
    dirs.sort();
    let failures: Vec<String> =
        dirs.iter().filter_map(|d| check_run(d, &|ext| d.join(format!("expected.{ext}"))).err()).collect();
    assert!(failures.is_empty(), "{} package(s) failed:\n\n{}", failures.len(), failures.join("\n\n"));
}

#[test]
fn usage_errors_exit_2() {
    let out = ail().arg("nonsense").output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}
