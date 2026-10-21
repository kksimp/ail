//! The shared table of constructs outside the native subset (ADR 0746): every row
//! has a construct, a wording and an owner; each wording occurs in a backend
//! diagnostic (`compiler/`) or in the coverage report, so the table cannot drift.

use std::fs;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn rows() -> Vec<Vec<String>> {
    let text = fs::read_to_string(repo().join("docs/design/native-subset-constructs.md")).unwrap();
    text.lines()
        .filter(|l| l.starts_with('|'))
        .map(|l| l.trim().trim_matches('|').split('|').map(|c| c.trim().to_string()).collect::<Vec<_>>())
        .filter(|c| c.len() == 3 && c[0] != "Construct" && !c[0].starts_with("---"))
        .collect()
}

/// Every `.ail` file under `dir`, recursively.
fn ail_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            ail_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "ail") {
            out.push(path);
        }
    }
}

/// An owner is `-` (unowned), or comma-separated parts, each `#N` (a PR) or `lane N`.
fn owner_ok(owner: &str) -> bool {
    owner == "-"
        || owner.split(", ").all(|part| {
            let digits = part.strip_prefix('#').or_else(|| part.strip_prefix("lane "));
            digits.is_some_and(|d| !d.is_empty() && d.chars().all(|c| c.is_ascii_digit()))
        })
}

#[test]
fn every_row_has_a_valid_owner() {
    let rows = rows();
    assert!(rows.len() >= 20, "the table has {} rows", rows.len());
    for row in &rows {
        assert!(owner_ok(&row[2]), "bad owner `{}` for `{}`", row[2], row[0]);
    }
}

#[test]
fn every_wording_occurs_in_a_diagnostic_or_the_coverage_report() {
    let mut files = Vec::new();
    ail_files(&repo().join("compiler"), &mut files);
    let mut text = String::new();
    for f in files.iter().map(PathBuf::as_path).chain(std::iter::once(
        repo().join("docs/design/native-coverage.md").as_path(),
    )) {
        text.push_str(&fs::read_to_string(f).unwrap_or_default().replace('`', ""));
    }
    for row in rows() {
        assert!(text.contains(&row[1]), "wording `{}` for `{}` is in no diagnostic", row[1], row[0]);
    }
}
