//! The native runtime's global slots (native-abi.md §8): every
//! name passed to `global(NAME)` in `compiler/rt/native/` is a constant with
//! its own slot below the image's `globals` count. Two names share a slot
//! only when the later one's doc comment says it `shares` the slot. A code
//! generator constant of the same name must have the same value.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

struct Decl {
    value: u64,
    at: String,
    shares: bool,
}

/// `NAME := N` at the start of a line, with whether the comment lines just
/// above it say `shares`.
fn constants(path: &Path) -> Vec<(String, Decl)> {
    let text = fs::read_to_string(path).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    let rel = path.strip_prefix(root()).unwrap().to_string_lossy().replace('\\', "/");
    let mut out = Vec::new();
    for (k, line) in lines.iter().enumerate() {
        let line = line.strip_prefix("pub ").unwrap_or(line);
        let Some((name, value)) = line.split_once(" := ") else { continue };
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_') {
            continue;
        }
        let Ok(value) = value.trim().parse::<u64>() else { continue };
        let mut shares = false;
        let mut j = k;
        while j > 0 && lines[j - 1].trim_start().starts_with('#') {
            j -= 1;
            shares |= lines[j].contains("shares");
        }
        out.push((name.to_string(), Decl { value, at: format!("{rel}:{}", k + 1), shares }));
    }
    out
}

/// The names passed to `global(...)`.
fn slot_names(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(i) = rest.find("global(") {
        rest = &rest[i + "global(".len()..];
        let name: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
        if rest[name.len()..].starts_with(')') && name.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_') {
            out.push(name);
        }
    }
    out
}

#[test]
fn runtime_global_slots_are_distinct() {
    let dir = root().join("compiler/rt/native");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).collect();
    files.sort();
    let mut decls: BTreeMap<String, Vec<Decl>> = BTreeMap::new();
    let mut used = Vec::new();
    for f in files.iter().filter(|p| p.extension().is_some_and(|x| x == "ail")) {
        for (name, d) in constants(f) {
            decls.entry(name).or_default().push(d);
        }
        used.extend(slot_names(&fs::read_to_string(f).unwrap()));
    }
    used.sort();
    used.dedup();
    let image = fs::read_to_string(root().join("compiler/codegen/image.ail")).unwrap();
    let count: u64 = image
        .split("globals: ")
        .nth(1)
        .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
        .and_then(|s| s.parse().ok())
        .expect("`globals: N` in compiler/codegen/image.ail");
    let mut failures = Vec::new();
    let mut by_slot: BTreeMap<u64, Vec<(&str, &Decl)>> = BTreeMap::new();
    for name in &used {
        match decls.get(name).map(Vec::as_slice) {
            None | Some([]) => failures.push(format!("`global({name})`: no `{name} := N` in compiler/rt/native")),
            Some([d]) => {
                if d.value >= count {
                    failures.push(format!("{}: `{name}` is slot {}, but the image has {count} globals", d.at, d.value));
                }
                by_slot.entry(d.value).or_default().push((name, d));
            }
            Some(ds) => failures.push(format!(
                "`{name}` is declared {} times: {}",
                ds.len(),
                ds.iter().map(|d| d.at.as_str()).collect::<Vec<_>>().join(", ")
            )),
        }
    }
    for (slot, names) in &by_slot {
        if names.len() > 1 && names.iter().filter(|(_, d)| !d.shares).count() > 1 {
            failures.push(format!(
                "slot {slot} is used by {}; give each its own slot, or say `shares` in the doc comment of the one that may",
                names.iter().map(|(n, d)| format!("`{n}` ({})", d.at)).collect::<Vec<_>>().join(" and ")
            ));
        }
    }
    for generator in ["compiler/x64/gen.ail", "compiler/a64/gen.ail"] {
        for (name, d) in constants(&root().join(generator)) {
            if let Some([rt]) = decls.get(&name).map(Vec::as_slice) {
                if used.contains(&name) && rt.value != d.value {
                    failures.push(format!("{}: `{name}` is {}, but the runtime's ({}) is {}", d.at, d.value, rt.at, rt.value));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
