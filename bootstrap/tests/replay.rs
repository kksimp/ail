//! Record and replay (replay.rec.1-5, replay.cap.1-2, replay.run.1-3,
//! ADR 0398).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn dir(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("replay").join(name);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// `ail ARGS` in `cwd`, with `stdin` on standard input.
fn ail(cwd: &Path, args: &[&str], stdin: &str) -> Output {
    use std::io::Write;
    let mut child = Command::new(env!("CARGO_BIN_EXE_ail"))
        .args(args)
        .current_dir(cwd)
        .env("REPLAY_NAME", "first")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
    child.wait_with_output().unwrap()
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

const PROGRAM: &str = r#"fn main() !
    t := time.now_ms()
    n := random.int(1, 1000000)
    data := fs.read_text("input.txt")?
    name := env.get("REPLAY_NAME") else "nobody"
    line := io.read_line() else "eof"
    gone := fs.read_text("missing.txt") else e
        "{e.kind}"
    print("{t} {n} {name} {line} {data.lines().len()} {gone}")
    xs := [1, 2]
    print(xs[data.lines().len()])
"#;

/// replay.rec.1-2, replay.run.1-2: a replay prints what the recorded run
/// printed and traps at the same place, though the world has changed.
#[test]
fn a_replay_repeats_the_run() {
    let d = dir("repeat");
    fs::write(d.join("prog.ail"), PROGRAM).unwrap();
    fs::write(d.join("input.txt"), "a\nb\nc\n").unwrap();
    let first = ail(&d, &["run", "--record", "r.json", "prog.ail"], "hello\n");
    assert_eq!(first.status.code(), Some(1), "{}", text(&first.stderr));
    assert!(text(&first.stdout).contains(" first hello 3 not_found"), "{}", text(&first.stdout));
    assert!(text(&first.stderr).contains("prog.ail:11:13: trap[type.coll.4]"), "{}", text(&first.stderr));
    let rec = fs::read_to_string(d.join("r.json")).unwrap();
    assert!(rec.starts_with("{\"ail_record\":1,\"program\":\"prog.ail\""), "{rec}");
    assert!(rec.contains("{\"call\":\"io.read_line\",\"line\":6,\"col\":16,\"result\":\"hello\"}"), "{rec}");
    // The world changes; the replay doesn't see it, and needs no FILE.
    fs::write(d.join("input.txt"), "x\n").unwrap();
    let again = ail(&d, &["run", "--replay", "r.json"], "");
    assert_eq!(text(&again.stdout), text(&first.stdout));
    assert_eq!(text(&again.stderr), text(&first.stderr));
    assert_eq!(again.status.code(), Some(1));
}

/// replay.run.3: a program that changed since the recording traps at the
/// first call that differs.
#[test]
fn a_changed_program_diverges() {
    let d = dir("diverge");
    fs::write(d.join("prog.ail"), "fn main()\n    print(env.get(\"REPLAY_NAME\") else \"\")\n").unwrap();
    assert!(ail(&d, &["run", "--record", "r.json", "prog.ail"], "").status.success());
    fs::write(d.join("prog.ail"), "fn main()\n    print(fs.exists(\"x\"))\n").unwrap();
    let out = ail(&d, &["run", "--replay", "r.json", "prog.ail"], "");
    let err = text(&out.stderr);
    assert!(err.contains("prog.ail:2:14: trap[replay.run.3]"), "{err}");
    assert!(err.contains("the recording's next effect is `env.get`"), "{err}");
}

/// replay.cap.2: a call that returns a handle is marked, and the replay
/// stops there; replay.rec.3: a trap inside a call repeats.
#[test]
fn handles_stop_and_traps_repeat() {
    let d = dir("handles");
    let exe = env!("CARGO_BIN_EXE_ail").replace('\\', "/");
    fs::write(d.join("start.ail"), format!("fn main() !\n    print(\"a\")\n    p := proc.start(\"{exe}\", [\"version\"])?\n")).unwrap();
    ail(&d, &["run", "--record", "s.json", "start.ail"], "");
    assert!(fs::read_to_string(d.join("s.json")).unwrap().contains("\"call\":\"proc.start\",\"line\":3,\"col\":15,\"uncaptured\":true"));
    let out = ail(&d, &["run", "--replay", "s.json"], "");
    assert_eq!(text(&out.stdout), "a\n");
    assert!(text(&out.stderr).contains("trap[replay.cap.2]"), "{}", text(&out.stderr));

    fs::write(d.join("sleep.ail"), "fn main()\n    time.sleep_ms(-1)\n").unwrap();
    let first = ail(&d, &["run", "--record", "t.json", "sleep.ail"], "");
    let again = ail(&d, &["run", "--replay", "t.json"], "");
    assert!(text(&first.stderr).contains("trap[std.time.2]"));
    assert_eq!(text(&again.stderr), text(&first.stderr));
}

/// replay.rec.1, replay.rec.5, replay.run.1: `ail test --record` keeps the failed tests' effects, and
/// `--replay` runs those tests again with the same numbers.
#[test]
fn tests_record_their_failures() {
    let d = dir("tests");
    let src = "fn greeting() str\n    \"hi {env.get(\"REPLAY_NAME\") else \"x\"} {random.int(1, 1000000)}\"\n\n@test\nfn passes()\n    assert_eq(1 + 1, 2)\n\n@test\nfn flaky()\n    assert_eq(greeting(), \"hi first 0\")\n";
    fs::write(d.join("t.ail"), src).unwrap();
    let first = ail(&d, &["test", "--record", "r.json", "t.ail"], "");
    assert_eq!(first.status.code(), Some(1));
    let rec = fs::read_to_string(d.join("r.json")).unwrap();
    assert!(rec.contains("{\"test\":\"flaky\"}\n{\"call\":\"env.get\""), "{rec}");
    assert!(!rec.contains("passes"), "{rec}");
    let again = ail(&d, &["test", "--replay", "r.json"], "");
    let (a, b) = (text(&first.stdout), text(&again.stdout));
    let fail = |s: &str| s.lines().find(|l| l.starts_with("FAIL flaky")).map(str::to_string);
    assert_eq!(fail(&a), fail(&b));
    assert!(b.contains("0 passed, 1 failed"), "{b}");
}

/// replay.rec.1: the options need `run` or `test` and a file.
#[test]
fn bad_options_are_explained() {
    let d = dir("options");
    let out = ail(&d, &["check", "--record", "r.json", "x.ail"], "");
    assert_eq!(out.status.code(), Some(2));
    assert!(text(&out.stderr).contains("(replay.rec.1)"));
    let out = ail(&d, &["run", "--replay", "nothing.json"], "");
    assert_eq!(out.status.code(), Some(2));
    assert!(text(&out.stderr).contains("can't read the recording nothing.json"), "{}", text(&out.stderr));
}
