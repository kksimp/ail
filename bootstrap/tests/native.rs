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
