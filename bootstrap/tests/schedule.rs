//! `--schedule=random[:SEED]` (conc.sched.1-2): a seed picks the task
//! switch points, so a failing interleaving replays exactly.

use std::process::Command;

fn test(args: &[&str]) -> (Option<i32>, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_ail"))
        .arg("test")
        .args(args)
        .arg("bootstrap/tests/schedule/race.ail")
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/.."))
        .output()
        .unwrap();
    (out.status.code(), String::from_utf8_lossy(&out.stdout).into_owned())
}

#[test]
fn a_seed_replays_its_interleaving() {
    // Seed 1 interleaves the two tasks' load and store; seed 2 doesn't.
    let (code, first) = test(&["--json", "--schedule=random:1"]);
    assert_eq!(code, Some(1), "{first}");
    assert!(first.contains("left is 1, right is 2"), "{first}");
    assert!(first.ends_with("{\"summary\":true,\"passed\":0,\"failed\":1,\"schedule_seed\":1}\n"), "{first}");
    for _ in 0..3 {
        assert_eq!(test(&["--json", "--schedule=random:1"]).1, first);
    }
    let (code, passed) = test(&["--json", "--schedule=random:2"]);
    assert_eq!(code, Some(0), "{passed}");
    assert!(passed.ends_with("{\"summary\":true,\"passed\":1,\"failed\":0}\n"), "{passed}");
}

#[test]
fn a_failure_prints_the_seed() {
    let (_, text) = test(&["--schedule=random:1"]);
    assert!(text.contains("schedule seed 1: replay with `ail test --schedule=random:1 "), "{text}");
}

#[test]
fn a_bad_schedule_is_a_usage_error() {
    let (code, _) = test(&["--schedule=sometimes"]);
    assert_eq!(code, Some(2));
}
