//! `ail diff OLD NEW` (diff.out.1-7, ADRs 0396, 0399).

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

mod common;

fn ail(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ail")).args(args).output().unwrap()
}

fn dir(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("diff").join(name);
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

const OLD: &str = "fn add(a int, b int) int\n    a + b\n\nfn gone() int\n    1\n\nfn same() int\n    2\n\n@test fn adds()\n    assert(add(1, 2) == 3)\n\nfn main()\n    print(add(1, 2))\n";

const NEW: &str = "## Adds.\npub fn add(a int, b int, c int) int\n    requires c >= 0\n    a + b\n\nfn same() int\n    2\n\nfn fresh() int\n    todo\n\n@test fn adds()\n    assert(add(1, 2, 0) == 4)\n\nfn main()\n    print(add(1, 2, 0))\n";

fn pair(name: &str) -> (String, String) {
    let d = dir(name);
    fs::create_dir_all(d.join("old")).unwrap();
    fs::create_dir_all(d.join("new")).unwrap();
    fs::write(d.join("old/p.ail"), OLD).unwrap();
    fs::write(d.join("new/p.ail"), NEW).unwrap();
    (common::shown(&d.join("old/p.ail")), common::shown(&d.join("new/p.ail")))
}

/// diff.out.2-4, diff.out.6-7: declarations, diagnostics and tests, as text.
#[test]
fn text_report() {
    let (old, new) = pair("text");
    let out = ail(&["diff", &old, &new]);
    let o = text(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{o}{}", text(&out.stderr));
    for want in [
        "~ fn:add (doc, body changed)\n    signature: fn add(a int, b int) int -> fn add(a int, b int, c int) int\n    pub: false -> true\n    requires: (none) -> [\"n >= 0\"]\n    +requires n >= 0\n",
        "+ fn:fresh fn fresh() int\n~ fn:adds (body changed)\n    -assert(add(1, 2) == 3)\n    +assert(add(1, 2, 0) == 4)\n~ fn:main (body changed)\n    -print(add(1, 2))\n    +print(add(1, 2, 0))\n",
        "- fn:gone fn gone() int\n",
        &format!("+ warning[expr.hole.1] {new}:10: hole: expected `int`; in scope: nothing\n"),
        "test adds: pass -> fail: trap[test.assert.1]: assertion failed\n",
        "1 added, 1 removed, 3 changed; diagnostics +1 -0; failing tests 0 -> 1\n",
    ] {
        let want = want.replace("n >= 0", "c >= 0");
        assert!(o.contains(&want), "missing {want:?} in\n{o}");
    }
    assert!(!o.contains("fn:same"), "{o}");
}

/// diff.out.6: the same program twice is no change, exit 0.
#[test]
fn no_change() {
    let (old, _) = pair("same");
    let out = ail(&["diff", "--no-tests", &old, &old]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(text(&out.stdout), "0 added, 0 removed, 0 changed; diagnostics +0 -0; tests not run\n");
}

/// diff.out.2-4: one JSON line per change, then a summary; a test that now
/// fails carries `"status":"fail"`.
#[test]
fn json_lines() {
    let (old, new) = pair("json");
    let o = text(&ail(&["diff", "--json", &old, &new]).stdout);
    assert!(o.contains("{\"change\":\"api\",\"op\":\"added\",\"id\":\"fn:fresh\",\"kind\":\"fn\",\"signature\":\"fn fresh() int\"}\n"), "{o}");
    assert!(o.contains("{\"change\":\"test\",\"test\":\"adds\",\"old\":\"pass\",\"new\":\"fail\",\"status\":\"fail\","), "{o}");
    assert!(o.contains("\"pub\":{\"old\":\"false\",\"new\":\"true\"}"), "{o}");
    let last = o.lines().last().unwrap();
    assert!(last.starts_with("{\"summary\":true,"), "{last}");
    assert!(last.ends_with("\"added\":1,\"removed\":1,\"changed\":3,\"diagnostics_added\":1,\"diagnostics_removed\":0,\"tests_changed\":1,\"failing_before\":0,\"failing_after\":1}"), "{last}");
}

/// diff.out.5: `--rev REV PATH` compares against git's copy.
#[test]
fn against_a_git_revision() {
    // Skipped when `git` is missing (under Wine it is not installed): the test
    // builds its own git repository to compare against.
    if Command::new("git").arg("--version").output().is_err() {
        eprintln!("skipped: no `git`");
        return;
    }
    let d = dir("rev");
    let git = |args: &[&str]| {
        let ok = Command::new("git")
            .args(["-c", "user.name=t", "-c", "user.email=t@t", "-c", "commit.gpgsign=false"])
            .args(args)
            .current_dir(&d)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        assert!(ok, "git {args:?}");
    };
    git(&["init", "-q"]);
    fs::write(d.join("p.ail"), OLD).unwrap();
    git(&["add", "p.ail"]);
    git(&["commit", "-q", "-m", "old"]);
    fs::write(d.join("p.ail"), NEW).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_ail"))
        .args(["diff", "--rev", "HEAD", "--no-tests", "p.ail"])
        .current_dir(&d)
        .output()
        .unwrap();
    let o = text(&out.stdout);
    assert!(o.ends_with("1 added, 1 removed, 3 changed; diagnostics +1 -0; tests not run\n"), "{o}{}", text(&out.stderr));
    let bad = Command::new(env!("CARGO_BIN_EXE_ail"))
        .args(["diff", "--rev", "nosuchrev", "p.ail"])
        .current_dir(&d)
        .output()
        .unwrap();
    assert_eq!(bad.status.code(), Some(2));
    assert!(text(&bad.stderr).contains("(diff.out.5)"));
}
