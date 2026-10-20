//! End-to-end tests: run the real `ail` binary on example programs.
//!
//! - `examples/NAME.ail` + `NAME.out`: must run and print exactly `NAME.out`.
//! - `bootstrap/tests/cases/NAME.ail` + `NAME.out`: same.
//! - `NAME.ail` + `NAME.rules`: the diagnostics' spec rule IDs (in report
//!   order) must equal the file's lines. The program must be rejected (exit 2)
//!   if any of them is an error; with only warnings, it is accepted (exit 0).
//! - `NAME.ail` + `NAME.test`: `ail test --json` must print exactly this on
//!   stdout, exiting 0 if no line has `"status":"fail"`, else 1.
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
    let test_file = expect("test");
    let name = path.display();
    if test_file.exists() {
        let out = ail().args(["test", "--json"]).arg(path).output().unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        let expected = fs::read_to_string(&test_file).unwrap();
        let want_exit = if expected.contains("\"status\":\"fail\"") { 1 } else { 0 };
        if out.status.code() != Some(want_exit) || stdout != expected {
            return Err(format!(
                "{name}: ail test output differs\n--- expected (exit {want_exit})\n{expected}--- actual (exit {:?})\n{stdout}",
                out.status.code()
            ));
        }
        Ok(())
    } else if err_file.exists() {
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

/// Diagnostics schema v1 (ADR 0145): version, span and machine-applicable fixes.
#[test]
fn json_diagnostics_have_spans_and_fixes() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases/diag_fixes.ail");
    let out = ail().args(["check", "--json"]).arg(&path).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(lines.len(), 2, "{stderr}");
    assert!(lines.iter().all(|l| l.starts_with("{\"api\":1,")), "{stderr}");
    // `mut total := total + i` becomes `total = total + i`.
    assert!(
        lines[0].contains(
            "\"end_line\":4,\"end_col\":18,\"fixes\":[{\"title\":\"assign to `total` with `=`\",\"edits\":[\
             {\"line\":4,\"col\":9,\"end_line\":4,\"end_col\":13,\"text\":\"\"},\
             {\"line\":4,\"col\":19,\"end_line\":4,\"end_col\":21,\"text\":\"=\"}]}]"
        ),
        "{stderr}"
    );
    assert!(
        lines[1].contains("\"edits\":[{\"line\":5,\"col\":5,\"end_line\":5,\"end_col\":9,\"text\":\"print\"}]"),
        "{stderr}"
    );
    let text = ail().arg("check").arg(&path).output().unwrap();
    assert!(String::from_utf8_lossy(&text.stderr).contains("\nfix: replace `prnt` with `print`"));
}

/// `ail explain` (ADR 0146): a known rule, JSON, and an unknown rule.
#[test]
fn explain_prints_rules() {
    let out = ail().args(["explain", "expr.hole.2"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout.starts_with("expr.hole.2 (docs/spec/05-expressions.md:"), "{stdout}");
    assert!(stdout.contains("Evaluating a hole MUST trap"), "{stdout}");
    assert!(stdout.contains("ADR 0147: Typed holes"), "{stdout}");

    let out = ail().args(["explain", "--json", "expr.bind.4"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.starts_with("{\"api\":1,\"rule\":\"expr.bind.4\",\"file\":\"docs/spec/05-expressions.md\""));
    assert!(stdout.contains("{\"id\":\"0039\",\"title\":"), "{stdout}");
    assert!(stdout.contains("\"example\":\"fn total("), "{stdout}");

    let out = ail().args(["explain", "expr.bind.99"]).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        stderr.starts_with("error: unknown rule `expr.bind.99`\nhelp: known rules nearby: expr.bind.1"),
        "{stderr}"
    );

    let out = ail().args(["explain", "bootstrap.unsupported"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
}

/// `ail api` (ADR 0150) outlines every example and every accepted case,
/// with and without `--symbols`, and rejects a program with errors (api.out.5).
#[test]
fn api_outlines_every_accepted_program() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = ail_files(&root.join("../examples"));
    files.extend(ail_files(&root.join("tests/cases")).into_iter().filter(|p| p.with_extension("out").exists()));
    for path in files {
        for extra in [None, Some("--symbols"), Some("--pretty"), Some("--text")] {
            let out = ail().arg("api").args(extra).arg(&path).output().unwrap();
            assert_eq!(out.status.code(), Some(0), "{} {extra:?}: {}", path.display(), String::from_utf8_lossy(&out.stderr));
        }
    }
    let rejected = root.join("tests/cases/condition_not_bool.ail");
    let out = ail().args(["api", "--json"]).arg(&rejected).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("\"rule\":"));
    let out = ail().args(["api", "--bogus", "x.ail"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}

/// test.prop.3: `--cases` and `--seed` fix a property's inputs, so a run
/// replays exactly.
#[test]
fn property_options_replay() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases/test_prop_fail.ail");
    let run = |extra: &[&str]| {
        let out = ail().args(["test", "--json"]).args(extra).arg(&path).output().unwrap();
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    let a = run(&["--seed", "7", "--cases", "50"]);
    assert_eq!(a, run(&["--cases", "50", "--seed", "7"]));
    assert!(a.contains("\"seed\":7,") && a.contains("\"cases\":50"), "{a}");
    assert_ne!(a, run(&["--seed", "8", "--cases", "50"]));
    let out = ail().args(["test", "--cases", "x"]).arg(&path).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}
