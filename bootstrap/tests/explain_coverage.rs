//! `ail explain` and `ail fix` coverage (explain.cover.1-3, ADR 0335): every
//! rule a diagnostic can carry has an explain entry, every entry cites only
//! rules that exist, the list of rules with a fix matches what the corpora
//! produce, and an entry stays short.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn files(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            files(&p, ext, out);
        } else if p.extension().is_some_and(|x| x == ext) {
            out.push(p);
        }
    }
    out.sort();
}

/// Rule-ID-shaped tokens (`chapter.section.n`) in `text`.
fn ids_in(text: &str) -> BTreeSet<String> {
    let shaped = |s: &str| {
        let parts: Vec<&str> = s.split('.').collect();
        parts.len() == 3
            && parts[..2].iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_lowercase() || c == '_'))
            && !parts[2].is_empty()
            && parts[2].chars().all(|c| c.is_ascii_digit())
    };
    text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.'))
        .map(|w| w.trim_matches('.'))
        .filter(|w| shaped(w))
        .map(str::to_string)
        .collect()
}

/// The rule IDs defined in `docs/spec` (a line starting `**id**`).
fn spec_rules() -> BTreeSet<String> {
    let mut paths = Vec::new();
    files(&root().join("../docs/spec"), "md", &mut paths);
    let mut out = BTreeSet::new();
    for p in paths {
        for l in fs::read_to_string(p).unwrap().lines() {
            if let Some(rest) = l.strip_prefix("**")
                && let Some(id) = rest.split("**").next()
                && ids_in(id) == BTreeSet::from([id.to_string()])
            {
                out.insert(id.to_string());
            }
        }
    }
    out
}

fn ail() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ail"))
}

/// Rule IDs that appear as string literals in the sources: the ones a
/// diagnostic can carry.
fn emitted_rules() -> BTreeSet<String> {
    let mut paths = Vec::new();
    files(&root().join("src"), "rs", &mut paths);
    let mut out = BTreeSet::new();
    for p in paths {
        let text = fs::read_to_string(&p).unwrap();
        let mut rest = text.as_str();
        while let Some(i) = rest.find('"') {
            let after = &rest[i + 1..];
            let Some(j) = after.find('"') else { break };
            let lit = &after[..j];
            if !lit.contains(' ') && ids_in(lit) == BTreeSet::from([lit.to_string()]) {
                out.insert(lit.to_string());
            }
            rest = &after[j + 1..];
        }
    }
    out
}

/// IDs the sources spell as examples of a *missing* rule.
const NOT_RULES: &[&str] = &["expr.nope.1"];

// explain.cover.1
#[test]
fn every_emitted_rule_has_an_explain_entry() {
    let spec = spec_rules();
    let missing: Vec<String> =
        emitted_rules().into_iter().filter(|r| !spec.contains(r) && !NOT_RULES.contains(&r.as_str())).collect();
    assert!(missing.is_empty(), "rule IDs in src/ with no rule in docs/spec (add the rule, then `ail explain` has it): {missing:?}");
}

// explain.cover.2
#[test]
fn explain_entries_cite_only_rules_that_exist() {
    let spec = spec_rules();
    let mut paths = Vec::new();
    files(&root().join("../docs/spec"), "md", &mut paths);
    let mut bad = Vec::new();
    for p in paths {
        let text = fs::read_to_string(&p).unwrap();
        for (n, l) in text.lines().enumerate() {
            for id in ids_in(l) {
                if !spec.contains(&id) {
                    bad.push(format!("{}:{}: {id}", p.file_name().unwrap().to_string_lossy(), n + 1));
                }
            }
        }
    }
    assert!(bad.is_empty(), "docs/spec cites rules that no chapter defines:\n{}", bad.join("\n"));
}

// explain.cover.3
#[test]
fn rules_with_a_fix_match_the_corpora() {
    let mut paths = Vec::new();
    files(&root().join("tests"), "ail", &mut paths);
    files(&root().join("../examples"), "ail", &mut paths);
    let mut seen = BTreeSet::new();
    for p in &paths {
        let out = ail().args(["check", "--json"]).arg(p).output().unwrap();
        for l in String::from_utf8_lossy(&out.stderr).lines().chain(String::from_utf8_lossy(&out.stdout).lines()) {
            if l.contains("\"fixes\":[{")
                && let Some(rule) = l.split("\"rule\":\"").nth(1).and_then(|s| s.split('"').next())
            {
                seen.insert(rule.to_string());
            }
        }
    }
    let listed: BTreeSet<String> =
        fs::read_to_string(root().join("src/fixable.txt")).unwrap().lines().map(str::to_string).collect();
    let spec = spec_rules();
    assert_eq!(listed, seen, "src/fixable.txt is out of step with the fixes the test programs produce");
    assert!(listed.iter().all(|r| spec.contains(r)), "src/fixable.txt names a rule missing from docs/spec");
    let json = String::from_utf8(ail().args(["explain", "--json"]).output().unwrap().stdout).unwrap();
    assert!(json.contains(&format!("\"fixable\":{}", listed.len())), "{}", &json[..80.min(json.len())]);
    assert!(json.contains("\"rule\":\"trait.op.3\",\"chapter\":\"trait\"") && json.contains("\"fix\":true"));
}

// explain.cover.4: an entry stays short (ADR 0336). Measured with tokbench, 120
// tokens are about 400 characters of this text.
#[test]
fn explain_entries_stay_short() {
    let spec: Vec<String> = spec_rules().into_iter().collect();
    let mut total = 0usize;
    let mut longest = (0usize, String::new());
    for r in &spec {
        let out = ail().args(["explain", r]).output().unwrap();
        let n = String::from_utf8_lossy(&out.stdout).chars().count();
        total += n;
        if n > longest.0 {
            longest = (n, r.clone());
        }
    }
    let mean = total / spec.len();
    assert!(mean < 400, "mean explain entry is {mean} characters (about 120 tokens is 400); longest is {} ({} chars)", longest.1, longest.0);
}
