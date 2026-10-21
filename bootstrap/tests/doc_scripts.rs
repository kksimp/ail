//! The documentation checks in `tools/` (ADR 0002): relative links resolve
//! (`check-doc-links.py`) and the decision index, glossary and quick-reference
//! `std.*` rows have no duplicates (`check-doc-dupes.py`). Both run with
//! `python3 -I` from the repository root. When `python3` is missing (Windows
//! without it) the test prints why it skipped and passes.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn run_check(script: &str) {
    let root = repo_root();
    if Command::new("python3").arg("--version").output().is_err() {
        eprintln!("doc_scripts: python3 not found; skipping {script}");
        return;
    }
    let out = Command::new("python3")
        .args(["-I", &format!("tools/{script}")])
        .current_dir(&root)
        .output()
        .unwrap_or_else(|e| panic!("doc_scripts: could not run {script}: {e}"));
    assert!(
        out.status.success(),
        "{script} failed:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn doc_links_resolve() {
    run_check("check-doc-links.py");
}

#[test]
fn doc_index_and_glossary_have_no_duplicate_rows() {
    run_check("check-doc-dupes.py");
}
