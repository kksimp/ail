//! The native backend in AIL (`compiler/`, ADRs 0384-0393), driven through
//! `ail run compiler`: each `tests/native/NAME.ail` has the MIR it lowers to
//! (`NAME.mir`, ir-architecture.md §4) or the problems it reports
//! (`NAME.err`, native.subset.1).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn cases() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(root().join("bootstrap/tests/native"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "ail"))
        .collect();
    out.sort();
    out
}

/// `ail run compiler ARGS` from the repository root.
fn compiler(args: &[&str]) -> (bool, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_ail"))
        .current_dir(root())
        .args(["run", "compiler"])
        .args(args)
        .output()
        .unwrap();
    (out.status.success(), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
}

// native.subset.1, native.build.2
#[test]
fn mir_matches_the_expected_dumps() {
    let mut failures = Vec::new();
    for case in cases() {
        let rel = case.strip_prefix(root()).unwrap().to_string_lossy().replace('\\', "/");
        let (ok, out, err) = compiler(&["mir", &rel]);
        let mir = case.with_extension("mir");
        let errs = case.with_extension("err");
        let (want, got) = if mir.exists() {
            if !ok {
                failures.push(format!("{rel}: failed:\n{err}"));
                continue;
            }
            (fs::read_to_string(&mir).unwrap(), out)
        } else {
            let problems: Vec<&str> = err.lines().filter(|l| l.contains(": error[native.")).collect();
            if ok {
                failures.push(format!("{rel}: expected problems, but it lowered"));
                continue;
            }
            (fs::read_to_string(&errs).unwrap_or_default(), problems.join("\n") + "\n")
        };
        if want != got {
            failures.push(format!("{rel}: differs from the expected output; got:\n{got}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// `ail build` on each `tests/native/run/NAME.ail`, then the executable's
/// output against `ail run`'s: stdout, the exit code, and a trap's message
/// and excerpt (the interpreter's snapshot lines aside, trace.snap.1).
// native.build.1, expr.arith.1, expr.arith.3, type.coll.4, type.num.9, expr.bit.2
#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn executables_match_the_interpreter() {
    let dir = root().join("bootstrap/tests/native/run");
    let mut cases: Vec<PathBuf> = fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).collect();
    cases.sort();
    let tmp = std::env::temp_dir().join(format!("ail-native-{}", std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    let mut failures = Vec::new();
    for case in cases.iter().filter(|p| p.extension().is_some_and(|x| x == "ail")) {
        let rel = case.strip_prefix(root()).unwrap().to_string_lossy().replace('\\', "/");
        let exe = tmp.join(case.file_stem().unwrap());
        let built = Command::new(env!("CARGO_BIN_EXE_ail"))
            .current_dir(root())
            .args(["build", &rel, "-o"])
            .arg(&exe)
            .output()
            .unwrap();
        if !built.status.success() {
            failures.push(format!("{rel}: `ail build` failed:\n{}", String::from_utf8_lossy(&built.stderr)));
            continue;
        }
        let want = Command::new(env!("CARGO_BIN_EXE_ail")).current_dir(root()).args(["run", &rel]).output().unwrap();
        let got = Command::new(&exe).current_dir(root()).output().unwrap();
        let snapshot = |l: &&str| {
            !(l.starts_with("  at ") && l.contains(" in ") || l.starts_with("note:") || l.starts_with("help:"))
        };
        let want_err: Vec<&str> = std::str::from_utf8(&want.stderr).unwrap().lines().filter(snapshot).collect();
        let got_err: Vec<&str> = std::str::from_utf8(&got.stderr).unwrap().lines().collect();
        if want.stdout != got.stdout || want.status.code() != got.status.code() || want_err != got_err {
            failures.push(format!(
                "{rel}: differs from `ail run`\n--- ail run (exit {:?}):\n{}{}\n--- native (exit {:?}):\n{}{}",
                want.status.code(),
                String::from_utf8_lossy(&want.stdout),
                want_err.join("\n"),
                got.status.code(),
                String::from_utf8_lossy(&got.stdout),
                got_err.join("\n"),
            ));
        }
    }
    let _ = fs::remove_dir_all(&tmp);
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// A program outside the native subset builds nothing (native.subset.1).
#[test]
fn build_refuses_programs_outside_the_subset() {
    let tmp = std::env::temp_dir().join(format!("ail-native-refused-{}", std::process::id()));
    let out = Command::new(env!("CARGO_BIN_EXE_ail"))
        .current_dir(root())
        .args(["build", "bootstrap/tests/native/subset.ail", "-o"])
        .arg(&tmp)
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success() && !tmp.exists(), "built a program outside the subset");
    assert!(err.contains("error[native.subset.1]"), "{err}");
}
