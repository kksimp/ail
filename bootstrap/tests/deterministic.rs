//! End-to-end tests of `ail run --deterministic` and `--seed`
//! (std.random.3, std.random.5).

use std::process::Command;

fn ail(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ail"))
        .args(args)
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/.."))
        .output()
        .unwrap()
}

const UNSEEDED: &str = "bootstrap/tests/deterministic/unseeded.ail";

#[test]
fn seed_makes_a_random_program_repeat() {
    let a = ail(&["run", "--seed", "11", UNSEEDED]);
    let b = ail(&["run", "--seed", "11", UNSEEDED]);
    let c = ail(&["run", "--seed", "12", UNSEEDED]);
    assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
    assert_eq!(a.stdout, b.stdout);
    assert_ne!(a.stdout, c.stdout);
}

#[test]
fn deterministic_rejects_random_clock_and_proc() {
    for program in [UNSEEDED, "bootstrap/tests/deterministic/clock.ail", "bootstrap/tests/deterministic/proc.ail"] {
        let out = ail(&["run", "--deterministic", program]);
        assert_eq!(out.status.code(), Some(2), "{program}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("eff.cap.1"), "{program}");
    }
}

#[test]
fn deterministic_runs_a_program_without_them_identically_twice() {
    let a = ail(&["run", "--deterministic", "examples/hello.ail"]);
    let b = ail(&["run", "--deterministic", "examples/hello.ail"]);
    assert!(a.status.success());
    assert_eq!(a.stdout, b.stdout);
}

#[test]
fn seed_needs_an_integer() {
    let out = ail(&["run", "--seed", "x", UNSEEDED]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("std.random.3"));
}
