//! `ail fix` (fix.apply.1-4): each `tests/fix/NAME.ail` is copied to a
//! scratch directory and fixed. The result must equal `NAME.fixed`, check
//! without errors, and be stable (a second `ail fix` applies nothing).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn ail() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ail"))
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ail-fix-test-{}-{name}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn fixtures_are_fixed_and_check_clean() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fix");
    let mut count = 0;
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|x| x != "ail") {
            continue;
        }
        count += 1;
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let tmp = scratch(&name);
        let work = tmp.join(format!("{name}.ail"));
        fs::copy(&path, &work).unwrap();

        let out = ail().arg("fix").arg(&work).output().unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert_eq!(out.status.code(), Some(0), "{name}: ail fix\n{stdout}{}", String::from_utf8_lossy(&out.stderr));
        let want = fs::read_to_string(path.with_extension("fixed")).unwrap();
        assert_eq!(fs::read_to_string(&work).unwrap(), want, "{name}: fixed text differs\n{stdout}");

        let check = ail().arg("check").arg(&work).output().unwrap();
        assert_eq!(check.status.code(), Some(0), "{name}: fixed file doesn't check\n{}", String::from_utf8_lossy(&check.stderr));

        let again = ail().args(["fix", "--json"]).arg(&work).output().unwrap();
        let again = String::from_utf8_lossy(&again.stdout);
        assert!(again.contains("\"applied\":0,\"rounds\":0") && again.contains("\"clean\":true"), "{name}: not stable\n{again}");
        fs::remove_dir_all(&tmp).unwrap();
    }
    assert!(count >= 6, "expected the fix fixtures, found {count}");
}

#[test]
fn dry_run_prints_a_diff_and_leaves_the_file() {
    let tmp = scratch("dry");
    let work = tmp.join("x.ail");
    let src = "fn main()\n    n := 1\n    n = 2\n    print(n)\n";
    fs::write(&work, src).unwrap();
    let out = ail().args(["fix", "--dry-run"]).arg(&work).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "{stdout}");
    assert!(stdout.contains("+    mut n := 1"), "{stdout}");
    assert!(stdout.contains("would apply 1 fix(es)"), "{stdout}");
    assert_eq!(fs::read_to_string(&work).unwrap(), src);
    fs::remove_dir_all(&tmp).unwrap();
}

#[test]
fn unfixable_errors_exit_1() {
    let tmp = scratch("unfixable");
    let work = tmp.join("x.ail");
    fs::write(&work, "fn main()\n    print(y)\n").unwrap();
    let out = ail().args(["fix", "--json"]).arg(&work).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{stdout}");
    assert!(stdout.contains("\"applied\":0") && stdout.contains("\"clean\":false"), "{stdout}");
    fs::remove_dir_all(&tmp).unwrap();
}
