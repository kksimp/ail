//! The tools written in AIL (`tools/`) are AIL programs too: their checks
//! and `@test` suites run with the bootstrap's own tests.

use std::path::Path;
use std::process::Command;

fn ail() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ail"))
}

fn tool(rel: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(rel).display().to_string()
}

/// `tools/edit/edit.ail` (ADR 0180) checks without diagnostics and passes
/// its headless `@test` suite.
#[test]
fn editor_checks_clean_and_passes_its_tests() {
    let path = tool("tools/edit/edit.ail");
    let check = ail().args(["check", &path]).output().unwrap();
    let stderr = String::from_utf8_lossy(&check.stderr);
    assert!(check.status.success() && stderr.is_empty(), "ail check {path}:\n{stderr}");
    let test = ail().args(["test", &path]).output().unwrap();
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test.status.success() && stdout.contains(" 0 failed"), "ail test {path}:\n{stdout}");
}

/// Drives the editor headless (`--print` with keys) against the real
/// `ail api` and `ail fmt`, so a change to their output breaks this test
/// rather than the editor: hover, completion of locals and members, F12,
/// F2 rename and formatting on a fixture.
#[test]
fn editor_drives_the_real_semantic_api() {
    let dir = std::env::temp_dir().join(format!("ail-edit-smoke-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("smoke.ail");
    std::fs::copy(tool("tools/edit/testdata/smoke.ail"), &file).unwrap();
    let file = file.display().to_string();
    let screen = |flags: &[&str]| {
        let mut args = vec!["run".to_string(), tool("tools/edit/edit.ail"), "--print".to_string()];
        args.extend(flags.iter().map(|f| f.to_string()));
        args.push(file.clone());
        let out = ail().args(&args).env("AIL", env!("CARGO_BIN_EXE_ail")).output().unwrap();
        assert!(out.status.success(), "{flags:?}: {}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    let hover = screen(&["--at=11:15", "--key=ctrl+i"]);
    assert!(hover.contains(" fn add(a int, b int) int\n") && hover.contains(" ## Adds.\n"), "hover:\n{hover}");
    let locals = screen(&["--at=12:5", "--key=end", "--key=enter", "--type=to", "--key=ctrl+space"]);
    assert!(locals.contains(" total int (binding) "), "locals:\n{locals}");
    let members = screen(&["--at=12:5", "--key=end", "--key=enter", "--type=p."]);
    assert!(members.contains(" x x int ") && members.contains(" y y int "), "members:\n{members}");
    let definition = screen(&["--at=12:11", "--key=f12"]);
    assert!(definition.contains("Ln 11, Col 5"), "F12:\n{definition}");
    let rename = screen(&["--at=10:5", "--key=f2", "--type=t", "--key=enter"]);
    assert!(rename.contains("total := add(pt.x, pt.y)") && rename.contains("| renamed 3 `p`"), "F2:\n{rename}");
    // With the file broken `--refs` can't answer, so F2 falls back to words.
    let by_word = screen(&["--at=10:5", "--type=(", "--key=f2", "--type=t", "--key=enter"]);
    assert!(by_word.contains("total := add(pt.x, pt.y)") && !by_word.contains("Ln 10, Col 5"), "F2 by word:\n{by_word}");
    let squeeze = ["--at=7:6", "--key=delete", "--key=right", "--key=delete"];
    let squeezed = screen(&squeeze);
    assert!(squeezed.contains("  7     a+b\n"), "squeeze:\n{squeezed}");
    let format = screen(&[&squeeze[..], &["--key=alt+shift+f"]].concat());
    assert!(format.contains("  7     a + b\n") && format.contains("| formatted"), "format:\n{format}");
    std::fs::remove_dir_all(&dir).ok();
}

/// The README's key table is the editor's `--keys` output, the same list
/// the command palette shows (ADR 0291).
#[test]
fn editor_key_table_matches_the_readme() {
    let out = ail().args(["run", &tool("tools/edit/edit.ail"), "--keys"]).output().unwrap();
    assert!(out.status.success());
    let table = String::from_utf8_lossy(&out.stdout);
    let readme = std::fs::read_to_string(tool("tools/edit/README.md")).unwrap();
    assert!(readme.contains(table.trim_end()), "tools/edit/README.md's key table differs from `--keys`; paste its output:\n{table}");
}
