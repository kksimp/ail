//! `ail edit PATH < EDITS` (edit.in.1-3, edit.id.1-3, edit.op.1-7, edit.out.1-4, ADRs 0397, 0399).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn edit(path: &Path, args: &[&str], edits: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ail"))
        .arg("edit")
        .args(args)
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(edits.as_bytes()).unwrap();
    child.wait_with_output().unwrap()
}

fn dir(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("edit").join(name);
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

const SRC: &str = "struct Point\n    x int\n\nfn Point.shift(mut self, d int)\n    x += d\n\n## Adds.\nfn add(a int, b int) int\n    requires a >= 0\n    a + b\n\nfn old() int\n    1\n\nfn main()\n    mut p := Point(x: add(1, 2))\n    p.shift(add(3, 4))\n    print(p)\n";

/// edit.op.1-6, edit.out.1-2: every op, formatted and checked.
#[test]
fn every_op() {
    let d = dir("ops");
    let f = d.join("p.ail");
    fs::write(&f, SRC).unwrap();
    let ops = r###"[
        {"op":"rename","id":"fn:add","to":"plus"},
        {"op":"replace_body","id":"fn:plus","body":"a+b+0"},
        {"op":"insert","after":"fn:plus","text":"## Twice.\nfn twice(n int) int\n    plus(n, n)"},
        {"op":"delete","id":"fn:old"},
        {"op":"import","module":"std.fs"},
        {"op":"rename","id":"type:Point","to":"Pt"},
        {"op":"rename","id":"field:Pt.x","to":"xx"},
        {"op":"replace","id":"fn:main","text":"fn main()\n    mut p := Pt(xx: twice(2))\n    p.shift(plus(3, 4))\n    print(p)"}
    ]"###;
    let out = edit(&f, &[], ops);
    assert_eq!(out.status.code(), Some(0), "{}{}", text(&out.stdout), text(&out.stderr));
    assert_eq!(text(&out.stdout), "edited 1 file; 0 errors\n");
    let want = "use std.fs\n\nstruct Pt\n    xx int\n\nfn Pt.shift(mut self, d int)\n    xx += d\n\n## Adds.\nfn plus(a int, b int) int\n    requires a >= 0\n    a + b + 0\n\n## Twice.\nfn twice(n int) int\n    plus(n, n)\n\nfn main()\n    mut p := Pt(xx: twice(2))\n    p.shift(plus(3, 4))\n    print(p)\n";
    assert_eq!(fs::read_to_string(&f).unwrap(), want);
}

/// edit.in.2: a failing edit stops the run and nothing is written.
#[test]
fn all_or_nothing() {
    let d = dir("atomic");
    let f = d.join("p.ail");
    fs::write(&f, SRC).unwrap();
    let out = edit(&f, &[], r#"[{"op":"delete","id":"fn:old"},{"op":"delete","id":"fn:nope"}]"#);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(text(&out.stderr), "error: edit 2 of 2: no declaration has the id `fn:nope` (edit.id.1); nothing was written\n");
    assert_eq!(fs::read_to_string(&f).unwrap(), SRC);
    let bad = edit(&f, &[], "[{\"op\":");
    assert!(text(&bad.stderr).contains("(edit.in.1)"));
}

/// edit.out.2-3: the new diagnostics; `--if-clean` and `--dry-run` write nothing.
#[test]
fn diagnostics_and_dry_runs() {
    let d = dir("diags");
    let f = d.join("p.ail");
    fs::write(&f, SRC).unwrap();
    let broken = r#"[{"op":"replace_body","id":"fn:main","body":"print(nope)"}]"#;
    let out = edit(&f, &["--if-clean"], broken);
    assert_eq!(out.status.code(), Some(2));
    assert!(text(&out.stderr).contains("error[expr.name.1]: unknown name `nope`"), "{}", text(&out.stderr));
    assert!(text(&out.stdout).starts_with("not written (--if-clean) 1 file; 1 error"), "{}", text(&out.stdout));
    assert_eq!(fs::read_to_string(&f).unwrap(), SRC);
    let out = edit(&f, &["--dry-run", "--json"], r#"[{"op":"delete","id":"fn:old"}]"#);
    let o = text(&out.stdout);
    assert!(o.contains("-fn old() int\n"), "{o}");
    assert!(o.contains("\"written\":false"), "{o}");
    assert_eq!(fs::read_to_string(&f).unwrap(), SRC);
    let out = edit(&f, &[], broken);
    assert_eq!(out.status.code(), Some(2));
    assert!(fs::read_to_string(&f).unwrap().contains("print(nope)"));
}

/// edit.id.2, edit.op.5: in a package, ids carry the module and a rename
/// reaches every file.
#[test]
fn package_rename() {
    let d = dir("pkg").join("app");
    fs::create_dir_all(d.join("geo")).unwrap();
    fs::write(d.join("geo/shapes.ail"), "pub fn origin() int\n    0\n").unwrap();
    fs::write(d.join("main.ail"), "use geo\n\nfn main()\n    print(geo.origin())\n").unwrap();
    let out = edit(&d, &[], r#"[{"op":"rename","id":"fn:geo.origin","to":"zero"}]"#);
    assert_eq!(out.status.code(), Some(0), "{}{}", text(&out.stdout), text(&out.stderr));
    assert_eq!(fs::read_to_string(d.join("geo/shapes.ail")).unwrap(), "pub fn zero() int\n    0\n");
    assert_eq!(fs::read_to_string(d.join("main.ail")).unwrap(), "use geo\n\nfn main()\n    print(geo.zero())\n");
}

const AVG: &str = "fn average(xs List[int]) int\n    mut total := 0\n    for x in xs\n        total += x\n    total / xs.len()\n\n@test fn empty_is_zero()\n    assert_eq(average([]), 0)\n";

/// edit.in.3, edit.id.3, edit.op.7, edit.out.4: a compact hunk under a bare
/// name, `+` lines written at column 0, then the tests.
#[test]
fn compact_hunks() {
    let d = dir("compact");
    let f = d.join("avg.ail");
    fs::write(&f, AVG).unwrap();
    let out = edit(&f, &["--test"], "average\n+if xs.len() == 0\n+    return 0\n mut total := 0\n");
    assert_eq!(out.status.code(), Some(0), "{}{}", text(&out.stdout), text(&out.stderr));
    assert_eq!(text(&out.stdout), "edited 1 file; 0 errors\nok empty_is_zero\n1 passed, 0 failed\n");
    assert!(fs::read_to_string(&f).unwrap().starts_with("fn average(xs List[int]) int\n    if xs.len() == 0\n        return 0\n    mut total"));
    // One `-` and one `+` line that match no whole line replace a unique part of one.
    let out = edit(&f, &[], "fn:average\n-total / xs.len()\n+total / xs.len().max(1)\n");
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(fs::read_to_string(&f).unwrap().contains("    total / xs.len().max(1)\n"));
    let out = edit(&f, &[], "average\n-total\n+sum\n");
    assert_eq!(out.status.code(), Some(2));
    assert!(text(&out.stderr).contains("`total` occurs 3 times; quote more of its line in `fn:average` (edit.op.7)"), "{}", text(&out.stderr));
    let out = edit(&f, &[], "average\n-nothing here\n+x\n");
    assert!(text(&out.stderr).contains("`nothing here` is not found"), "{}", text(&out.stderr));
}

/// edit.in.3: `rename OLD NEW` and lines that are neither.
#[test]
fn compact_rename_and_errors() {
    let d = dir("compact_rename");
    let f = d.join("avg.ail");
    fs::write(&f, AVG).unwrap();
    let out = edit(&f, &[], "rename average mean\n");
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(fs::read_to_string(&f).unwrap().contains("assert_eq(mean([]), 0)"));
    let out = edit(&f, &[], "mean is wrong\n");
    assert_eq!(out.status.code(), Some(2));
    assert!(text(&out.stderr).contains("line 1: expected an id"), "{}", text(&out.stderr));
}

/// edit.out.3: a preview or a refused edit of one file never writes it, even for a moment, so a
/// program checking the same file at that time can't read the edited text.
#[test]
fn single_file_checks_run_from_memory() {
    let d = dir("memory");
    let f = d.join("p.ail");
    fs::write(&f, SRC).unwrap();
    let before = fs::metadata(&f).unwrap().modified().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));
    let bad = r#"[{"op":"replace_body","id":"fn:old","body":"nope"}]"#;
    let out = edit(&f, &["--dry-run"], bad);
    assert_eq!(out.status.code(), Some(2), "{}", text(&out.stdout));
    let out = edit(&f, &["--if-clean"], bad);
    assert_eq!(out.status.code(), Some(2), "{}", text(&out.stdout));
    assert_eq!(fs::metadata(&f).unwrap().modified().unwrap(), before, "the file was rewritten");
    assert_eq!(fs::read_to_string(&f).unwrap(), SRC);
}
