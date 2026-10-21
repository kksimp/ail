//! `ail test --update` (snap.update.1-6, ADR 0395).

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

mod common;

fn ail(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ail")).args(args).output().unwrap()
}

fn dir(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("snapshot").join(name);
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

const PROGRAM: &str = "## Doubles.\n##\n## ```ail\n## print(double(2))\n## # => 5\n## ```\nfn double(x int) int\n    x * 2\n\nfn greet(n str) str\n    \"hi, {n}!\"\n\n@test fn greets()\n    assert_eq(greet(\"bo\"), \"hello\")\n    assert_eq(double(-2), 7)\n    assert_eq([double(1), 3], [0])\n\nfn main()\n    print(greet(\"x\"))\n";

const UPDATED: &str = "## Doubles.\n##\n## ```ail\n## print(double(2))\n## # => 4\n## ```\nfn double(x int) int\n    x * 2\n\nfn greet(n str) str\n    \"hi, {n}!\"\n\n@test fn greets()\n    assert_eq(greet(\"bo\"), \"hi, bo!\")\n    assert_eq(double(-2), -4)\n    assert_eq([double(1), 3], [2, 3])\n\nfn main()\n    print(greet(\"x\"))\n";

/// snap.update.1-2, snap.update.4-5: literals, `# =>` lines and `.out` are rewritten,
/// the diff is printed and the tests then pass.
#[test]
fn update_rewrites_and_passes() {
    let d = dir("rewrites");
    let f = d.join("p.ail");
    fs::write(&f, PROGRAM).unwrap();
    fs::write(d.join("p.out"), "old\n").unwrap();
    let path = common::shown(&f);
    let out = ail(&["test", "--update", &path]);
    let o = text(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "{o}");
    assert!(o.contains("-    assert_eq(greet(\"bo\"), \"hello\")\n"), "{o}");
    assert!(o.contains("+    assert_eq(greet(\"bo\"), \"hi, bo!\")\n"), "{o}");
    assert!(o.contains("updated 5 snapshots in 2 files\n"), "{o}");
    assert!(o.ends_with("ok greets\nok doc:double\n2 passed, 0 failed\n"), "{o}");
    assert_eq!(fs::read_to_string(&f).unwrap(), UPDATED);
    assert_eq!(fs::read_to_string(d.join("p.out")).unwrap(), "hi, x!\n");
    let again = ail(&["test", "--update", &path]);
    assert!(text(&again.stdout).starts_with("updated 0 snapshots in 0 files\n"));
}

/// snap.update.6: `--dry-run` writes nothing; `--json` lists each change.
#[test]
fn dry_run_and_json() {
    let d = dir("dry");
    let f = d.join("p.ail");
    fs::write(&f, PROGRAM).unwrap();
    let path = common::shown(&f);
    let out = ail(&["test", "--update", "--dry-run", "--json", &path]);
    let o = text(&out.stdout);
    assert_eq!(fs::read_to_string(&f).unwrap(), PROGRAM);
    assert!(o.contains(&format!("{{\"snapshot\":\"assert_eq\",\"file\":\"{path}\",\"line\":14,\"col\":28,\"old\":\"\\\"hello\\\"\",\"new\":\"\\\"hi, bo!\\\"\"}}\n")), "{o}");
    assert!(o.contains("{\"snapshot\":\"doc\","), "{o}");
    assert!(o.contains("{\"snapshots\":true,\"dry_run\":true,\"updated\":4,\"files\":1,\"skipped\":0}\n"), "{o}");
}

/// snap.update.3: a literal reached with different values stays, and the
/// test fails; an expectation that is not a literal fails as usual.
#[test]
fn conflicts_and_non_literals_fail() {
    let d = dir("conflict");
    let f = d.join("p.ail");
    let src = "@test fn loops()\n    for i in 0..3\n        assert_eq(i * 2, 0)\n\n@test fn named()\n    want := 3\n    assert_eq(1 + 1, want)\n";
    fs::write(&f, src).unwrap();
    let out = ail(&["test", "--update", &common::shown(&f)]);
    let o = text(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{o}");
    assert!(o.contains("left 1 whose value differed between runs (snap.update.3)"), "{o}");
    assert!(o.contains("FAIL loops: trap[test.assert.1]: assertion failed: left is 4, right is 0"), "{o}");
    assert!(o.contains("FAIL named:"), "{o}");
    assert_eq!(fs::read_to_string(&f).unwrap(), src);
}
