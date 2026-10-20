//! Embeds the specification and the ADR index into the binary, so that
//! `ail explain` is self-contained (ADR 0146). Every `docs/spec/*.md` chapter
//! is included, so a new chapter needs no change here.

use std::fmt::Write as _;
use std::path::Path;

fn main() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let docs = Path::new(&manifest).join("../docs");
    let spec = docs.join("spec");
    let index = docs.join("decisions/README.md");
    println!("cargo:rerun-if-changed={}", spec.display());
    println!("cargo:rerun-if-changed={}", index.display());

    let mut chapters: Vec<_> = std::fs::read_dir(&spec)
        .expect("reading docs/spec")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    chapters.sort();
    let mut out = String::from("pub const SPEC: &[(&str, &str)] = &[\n");
    for p in &chapters {
        let name = p.file_name().unwrap().to_str().unwrap();
        let _ = writeln!(out, "    ({name:?}, include_str!({:?})),", p.canonicalize().unwrap().display().to_string());
    }
    out.push_str("];\n");
    let _ = writeln!(
        out,
        "pub const ADR_INDEX: &str = include_str!({:?});",
        index.canonicalize().unwrap().display().to_string()
    );
    std::fs::write(Path::new(&std::env::var("OUT_DIR").unwrap()).join("docs.rs"), out).unwrap();
}
