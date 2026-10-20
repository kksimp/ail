//! Spec conformance harness (ADR 0002): every fenced `ail` block in
//! `docs/spec/*.md`, `docs/quick-reference.md` and `docs/language-card.md` is
//! checked with `ail check --json`.
//!
//! The fence line says what to expect (documented in `docs/spec/README.md`):
//! - ```` ```ail ````: must check clean (warnings allowed).
//! - ```` ```ail fragment ````: a snippet that isn't a whole program. Column-0
//!   declarations (`struct`, `enum`, `fn`, `trait`, `impl`, `const`, `use`, `unsafe fn`, `extern`,
//!   `type`, `@attribute`, with their indented bodies) stay top level, and
//!   every other line becomes the body of a generated `fn main()`. A block
//!   that declares its own `main` is used as written.
//!   In `quick-reference.md` and `language-card.md` (they spend tokens on every
//!   fence) every block without a `main` is a fragment, and untagged.
//! - ```` ```ail error=RULE ````: must report RULE among its errors; handled
//!   as `fragment` when the block has no `main`.
//! - ```` ```ail skip=bootstrap ````: uses a feature the bootstrap doesn't
//!   implement yet; counted, not run.
//! - ```` ```ail skip=context ````: an illustration that relies on
//!   declarations the chapter doesn't show; counted, not run.

use std::fs;
use std::path::Path;
use std::process::Command;

enum Expect {
    Clean,
    Error(String),
    Skip,
}

struct Block {
    chapter: String,
    line: usize,
    expect: Expect,
    wrap: bool,
    code: String,
}

fn blocks(dir: &Path, extra: &[&Path]) -> Result<Vec<Block>, String> {
    let mut chapters: Vec<_> = fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    chapters.sort();
    chapters.extend(extra.iter().map(|p| p.to_path_buf())); // the quick reference and the language card
    let mut out = Vec::new();
    for path in chapters.iter().filter(|p| p.extension().is_some_and(|x| x == "md")) {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let text = fs::read_to_string(path).unwrap();
        let mut open: Option<(usize, Vec<String>, String)> = None;
        for (i, l) in text.lines().enumerate() {
            let n = i + 1;
            match &mut open {
                None => {
                    if let Some(tags) = l.strip_prefix("```ail") {
                        if !tags.is_empty() && !tags.starts_with(' ') {
                            continue; // another info string such as `ailx`
                        }
                        open = Some((n, tags.split_whitespace().map(str::to_string).collect(), String::new()));
                    }
                }
                Some((start, tags, code)) => {
                    if l.starts_with("```") {
                        let (start, tags, code) = (*start, tags.clone(), std::mem::take(code));
                        open = None;
                        let (mut expect, mut wrap) = (Expect::Clean, name.starts_with("quick-ref") || name.starts_with("language-card"));
                        for t in &tags {
                            if t == "fragment" {
                                wrap = true;
                            } else if let Some(r) = t.strip_prefix("error=") {
                                expect = Expect::Error(r.to_string());
                            } else if t == "skip=bootstrap" || t == "skip=context" {
                                expect = Expect::Skip;
                            } else {
                                return Err(format!("{name}:{start}: unknown fence annotation `{t}`"));
                            }
                        }
                        out.push(Block { chapter: name.clone(), line: start, expect, wrap, code });
                    } else {
                        code.push_str(l);
                        code.push('\n');
                    }
                }
            }
        }
        if let Some((start, ..)) = open {
            return Err(format!("{name}:{start}: unclosed code fence"));
        }
    }
    Ok(out)
}

fn has_main(code: &str) -> bool {
    code.lines().any(|l| l.starts_with("fn main"))
}

const DECLS: [&str; 13] =
    ["struct ", "enum ", "error ", "unique ", "fn ", "unsafe fn ", "extern ", "trait ", "impl ", "const ", "use ", "type ", "@", "pub ", "#"];

/// Split a fragment into top-level declarations and a `main` body.
fn wrap_fragment(code: &str) -> String {
    let (mut decls, mut body) = (String::new(), String::new());
    let mut in_decl = false;
    for l in code.lines() {
        if !l.is_empty() && !l.starts_with(' ') {
            in_decl = DECLS.iter().any(|d| l.starts_with(d));
        }
        if in_decl {
            decls.push_str(l);
            decls.push('\n');
        } else if l.is_empty() {
            body.push('\n');
        } else {
            body.push_str("    ");
            body.push_str(l);
            body.push('\n');
        }
    }
    format!("{decls}\nfn main()\n{}", if body.trim().is_empty() { "    print(\"\")\n".to_string() } else { body })
}

#[test]
fn spec_examples() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let docs = root.join("../docs");
    let blocks = blocks(&docs.join("spec"), &[&docs.join("quick-reference.md"), &docs.join("language-card.md")])
        .unwrap_or_else(|e| panic!("{e}"));
    let tmp = Path::new(env!("CARGO_TARGET_TMPDIR")).join("spec");
    fs::create_dir_all(&tmp).unwrap();
    let (mut run, mut skipped, mut failures) = (0, 0, Vec::new());
    for b in &blocks {
        if matches!(b.expect, Expect::Skip) {
            skipped += 1;
            continue;
        }
        run += 1;
        let code = if (b.wrap || matches!(b.expect, Expect::Error(_))) && !has_main(&b.code) {
            wrap_fragment(&b.code)
        } else {
            b.code.clone()
        };
        let file = tmp.join(format!("{}_{}.ail", b.chapter.trim_end_matches(".md"), b.line));
        fs::write(&file, &code).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_ail")).args(["check", "--json"]).arg(&file).output().unwrap();
        let stderr = String::from_utf8_lossy(&out.stderr);
        let errors: Vec<&str> = stderr
            .lines()
            .filter(|l| l.contains("\"severity\":\"error\""))
            .filter_map(|l| l.split("\"rule\":\"").nth(1))
            .map(|r| r.split('"').next().unwrap())
            .collect();
        let at = format!("{}:{}", b.chapter, b.line);
        match &b.expect {
            Expect::Clean if !errors.is_empty() => {
                failures.push(format!("{at}: expected clean, got errors {errors:?}\n{stderr}"))
            }
            Expect::Error(r) if !errors.contains(&r.as_str()) => {
                failures.push(format!("{at}: expected error {r}, got {errors:?}\n{stderr}"))
            }
            _ => {}
        }
    }
    eprintln!("spec examples: {run} checked, {skipped} skipped (skip=bootstrap)");
    assert!(failures.is_empty(), "{} spec example(s) failed:\n\n{}", failures.len(), failures.join("\n\n"));
}
