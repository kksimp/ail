//! ct.eval.3: a constant that runs too many statements stops with a rule-named trap.

use std::path::Path;
use std::process::Command;

fn run(limit: &str, program: &str) -> (i32, String, String) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let out = Command::new(env!("CARGO_BIN_EXE_ail"))
        .current_dir(&root)
        .env("AIL_CT_MAX_STEPS", limit)
        .args(["run", program])
        .output()
        .unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
}

#[test]
fn a_constant_over_the_limit_traps_and_names_itself() {
    let (code, _, err) = run("1000", "bootstrap/tests/ct_limit/spin.ail");
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("trap[ct.eval.3]"), "{err}");
    assert!(err.contains("the constant `BIG`"), "{err}");
}

#[test]
fn a_constant_under_the_limit_is_computed() {
    let (code, out, err) = run("100000", "bootstrap/tests/ct_limit/spin.ail");
    assert_eq!((code, out.trim()), (0, "499500"), "{err}");
}
