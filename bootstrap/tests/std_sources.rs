//! The standard modules written in AIL (`bootstrap/src/std`) are checked and
//! formatted like user code, so a break in one fails the selftest in the PR
//! that causes it (err.type.3 for a public function without error types,
//! `ail fmt` drift, or any other checker rule).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn ail() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ail"))
}

fn std_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/std")
}

/// The dotted names in `AIL_MODULES` of `src/stdlib.rs`, so a module added
/// there is covered without editing this test.
fn registered_modules() -> Vec<String> {
    let table = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/stdlib.rs")).unwrap();
    let start = table.find("const AIL_MODULES").expect("AIL_MODULES");
    let body = &table[start..];
    let body = &body[..body.find("];").unwrap()];
    body.lines().filter_map(|l| l.trim().strip_prefix("(\"")).map(|l| l[..l.find('"').unwrap()].to_string()).collect()
}

fn std_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            std_sources(&p, out);
        } else if p.extension().is_some_and(|x| x == "ail") {
            out.push(p);
        }
    }
}

#[test]
fn every_std_source_is_registered() {
    let mut files = Vec::new();
    std_sources(&std_dir(), &mut files);
    // `types.ail` is read with the prelude, not as a module.
    files.retain(|p| p.file_name().is_some_and(|n| n != "types.ail"));
    assert_eq!(
        files.len(),
        registered_modules().len(),
        "a file under bootstrap/src/std is missing from AIL_MODULES (or the reverse): {files:?}"
    );
}

#[test]
fn a_program_importing_every_std_module_checks() {
    let modules = registered_modules();
    assert!(modules.len() >= 7, "AIL_MODULES not read: {modules:?}");
    let mut src = String::new();
    for m in &modules {
        src.push_str(&format!("use {m}\n"));
    }
    src.push_str("\nfn main()\n    print(1)\n");
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("std_sources");
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("all_std.ail");
    fs::write(&file, src).unwrap();
    let out = ail().arg("check").arg(&file).output().unwrap();
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    assert!(out.status.success() && !text.contains("error["), "std modules fail `ail check`:\n{text}");
}

#[test]
fn std_sources_are_in_canonical_layout() {
    let out = ail().args(["fmt", "--check"]).arg(std_dir()).output().unwrap();
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    assert!(out.status.success(), "run `./ail fmt bootstrap/src/std`:\n{text}");
}
