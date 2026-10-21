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
//! - ```` ```ail rules=ID,ID ````: a runnable example that demonstrates the
//!   listed rules (a checked block covers them, like `error=RULE` does). Every
//!   rule of the batch chapters must be covered; see `list_uncovered`.
//!
//! - ```` ```ail norun ````: checks clean but isn't run: it traps on purpose or
//!   needs a file or input the harness doesn't have. Combine with `rules=`.
//! - ```` ```ail flags=A,B ````: the program is checked with those words added
//!   to `ail check` (for example `--deterministic`); it isn't run. Combine with
//!   `error=RULE` when the flags reject it.
//! - ```` ```ail tool=A,B,FILE expect=TEXT ````: runs `ail A B FILE` on the block
//!   (`FILE` is the block as a file; `DIR` is its directory) and requires exit
//!   status 0 and that stdout contains TEXT. The tool's output
//!   format is non-normative (`docs/design/semantic-api.md`); this checks that
//!   the tool runs on the example and prints what the rule describes.
//! - ```` ```ail pkg=DIR status=N ````: with `tool=`, `DIR` is a copy of the case
//!   package `bootstrap/tests/cases/packages/DIR` and the tool must exit N.
//! - ```` ```ail stdin=NAME ````: with `tool=`, the tool reads the file `NAME`
//!   of the package directory on its standard input (`ail edit` takes its edits there).
//! - ```` ```ail fixture=DIR ````: a block with no code. `ail check` runs on the
//!   case fixture `bootstrap/tests/cases/DIR` and must report every rule the
//!   block cites (its `rules=` list).
//!
//! Every program that checks clean and has a `main` also runs under both
//! interpreter tiers (ADR 0361); the two outputs must match exactly.

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
    run: bool,
    rules: Vec<String>,
    flags: Vec<String>,
    tool: Option<(Vec<String>, String)>,
    pkg: Option<String>,
    status: i32,
    stdin: Option<String>,
    fixture: Option<String>,
    code: String,
}

/// Chapters changed by the batch: every rule in them needs a runnable example.
const BATCH_CHAPTERS: [&str; 5] = ["04-types.md", "05-expressions.md", "17-std.md", "18-effects.md", "22-tools.md"];

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
                        let (mut expect, mut wrap, mut run, mut rules) = (
                            Expect::Clean,
                            name.starts_with("quick-ref") || name.starts_with("language-card"),
                            true,
                            Vec::new(),
                        );
                        let (mut flags, mut tool, mut fixture, mut pkg, mut status, mut stdin) =
                            (Vec::new(), None, None, None, 0, None);
                        let mut want = None;
                        for t in &tags {
                            if let Some(v) = t.strip_prefix("flags=") {
                                flags = v.split(',').map(str::to_string).collect();
                            } else if let Some(v) = t.strip_prefix("tool=") {
                                tool = Some((v.split(',').map(str::to_string).collect(), String::new()));
                            } else if let Some(v) = t.strip_prefix("expect=") {
                                want = Some(v.to_string());
                            } else if let Some(v) = t.strip_prefix("fixture=") {
                                fixture = Some(v.to_string());
                            } else if let Some(v) = t.strip_prefix("pkg=") {
                                pkg = Some(v.to_string());
                            } else if let Some(v) = t.strip_prefix("status=") {
                                status = v.parse().unwrap();
                            } else if let Some(v) = t.strip_prefix("stdin=") {
                                stdin = Some(v.to_string());
                            }
                        }
                        if let (Some((_, text)), Some(w)) = (&mut tool, want) {
                            *text = w;
                        }
                        for t in &tags {
                            if t.starts_with("flags=")
                                || t.starts_with("tool=")
                                || t.starts_with("expect=")
                                || t.starts_with("fixture=")
                                || t.starts_with("pkg=")
                                || t.starts_with("status=")
                                || t.starts_with("stdin=")
                            {
                                continue;
                            }
                            if t == "fragment" {
                                wrap = true;
                            } else if let Some(r) = t.strip_prefix("error=") {
                                expect = Expect::Error(r.to_string());
                                rules.push(r.to_string());
                            } else if let Some(ids) = t.strip_prefix("rules=") {
                                rules.extend(ids.split(',').map(str::to_string));
                            } else if t == "norun" {
                                run = false;
                            } else if t == "skip=bootstrap" || t == "skip=context" {
                                expect = Expect::Skip;
                            } else {
                                return Err(format!("{name}:{start}: unknown fence annotation `{t}`"));
                            }
                        }
                        out.push(Block {
                            chapter: name.clone(),
                            line: start,
                            expect,
                            wrap,
                            run,
                            rules,
                            flags,
                            tool,
                            fixture,
                            pkg,
                            status,
                            stdin,
                            code,
                        });
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

const DECLS: [&str; 15] = [
    "struct ",
    "enum ",
    "error ",
    "unique ",
    "fn ",
    "unsafe fn ",
    "extern ",
    "trait ",
    "impl ",
    "const ",
    "use ",
    "type ",
    "@",
    "pub ",
    "#",
];

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

/// Rule IDs defined in `docs/spec` (a line starting `**id**`), with the chapter file each is in.
fn spec_rules(spec: &Path) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut files: Vec<_> = fs::read_dir(spec).unwrap().map(|e| e.unwrap().path()).collect();
    files.sort();
    for path in files.iter().filter(|p| p.extension().is_some_and(|x| x == "md")) {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        for l in fs::read_to_string(path).unwrap().lines() {
            if let Some(rest) = l.strip_prefix("**")
                && let Some(id) = rest.split("**").next()
                && !rest[id.len()..].contains("[removed in")
                && id.contains('.')
                && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.')
            {
                out.push((id.to_string(), name.clone()));
            }
        }
    }
    out
}

/// Runs one program under both interpreter tiers (ADR 0361) and returns an error if they differ.
/// Examples run in their own scratch directory, so the file examples can make and remove files there.
fn run_both_tiers(file: &Path) -> Result<(), String> {
    // A snippet that opens a window waits for it to close (fine headless on Linux,
    // a real window on macOS), so it is checked but not run, as tests/bytecode.rs does.
    if fs::read_to_string(file).is_ok_and(|src| src.contains("ui.open(")) {
        return Ok(());
    }
    let mut outputs = Vec::new();
    for tier in [&[][..], &["--no-bytecode"][..]] {
        let out = Command::new(env!("CARGO_BIN_EXE_ail"))
            .arg("run")
            .args(tier)
            .arg(file)
            .current_dir(file.parent().unwrap())
            .output()
            .unwrap();
        if !out.status.success() {
            return Err(format!("`ail run {}` failed:\n{}", tier.join(" "), String::from_utf8_lossy(&out.stderr)));
        }
        outputs.push(out.stdout);
    }
    if outputs[0] != outputs[1] {
        return Err("bytecode and tree-walk tiers print different output".to_string());
    }
    Ok(())
}

/// Copies a directory tree (a case fixture) into `to`.
fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let dest = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &dest);
        } else {
            fs::copy(e.path(), dest).unwrap();
        }
    }
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
    let mut covered = std::collections::BTreeSet::new();
    let spec_ids = spec_rules(&docs.join("spec"));
    for b in &blocks {
        if matches!(b.expect, Expect::Skip) {
            // A skipped example is specified but not yet built into the bootstrap; it still
            // demonstrates its rules once the bootstrap runs it (ADR 0498).
            skipped += 1;
            for r in &b.rules {
                covered.insert(r.clone());
            }
            continue;
        }
        run += 1;
        let at = format!("{}:{}", b.chapter, b.line);
        let mut ok = true;
        if let Some(dir) = &b.fixture {
            // A case fixture (bootstrap/tests/cases) must report every rule the block cites.
            let out = Command::new(env!("CARGO_BIN_EXE_ail"))
                .args(["check", "--json"])
                .arg(root.join("tests/cases").join(dir))
                .output()
                .unwrap();
            let stderr = String::from_utf8_lossy(&out.stderr);
            for r in &b.rules {
                if !stderr.contains(&format!("\"rule\":\"{r}\"")) {
                    ok = false;
                    failures.push(format!("{at}: fixture {dir} doesn't report {r}\n{stderr}"));
                }
            }
        } else if let Some((args, want)) = &b.tool {
            // A tool runs on the example and prints the text the rule describes. `FILE` is the
            // example, `DIR` its package directory: the block as main.ail, or a copy of the case
            // fixture named by `pkg=`. The exit status must be `status=` (0 by default).
            let dir = tmp.join(format!("tool_{}_{}", b.chapter.trim_end_matches(".md"), b.line));
            let _ = fs::remove_dir_all(&dir);
            let file = dir.join("main.ail");
            if let Some(p) = &b.pkg {
                copy_dir(&root.join("tests/cases/packages").join(p), &dir);
            } else {
                fs::create_dir_all(&dir).unwrap();
                let code = if has_main(&b.code) { b.code.clone() } else { wrap_fragment(&b.code) };
                fs::write(&file, &code).unwrap();
            }
            let args: Vec<String> = args
                .iter()
                .map(|a| match a.as_str() {
                    "FILE" => file.to_string_lossy().to_string(),
                    "DIR" => dir.to_string_lossy().to_string(),
                    _ => a.clone(),
                })
                .collect();
            let mut cmd = Command::new(env!("CARGO_BIN_EXE_ail"));
            cmd.args(&args).current_dir(&dir);
            if let Some(name) = &b.stdin {
                cmd.stdin(fs::File::open(dir.join(name)).unwrap());
            } else {
                cmd.stdin(std::process::Stdio::null());
            }
            let out = cmd.output().unwrap();
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            if out.status.code() != Some(b.status) || !format!("{stdout}{stderr}").contains(want.as_str()) {
                ok = false;
                failures.push(format!(
                    "{at}: `ail {}` should exit {} and print {want:?}; exit {:?}\n{stdout}{stderr}",
                    args.join(" "),
                    b.status,
                    out.status.code(),
                ));
            }
        } else {
            let code = if (b.wrap || matches!(b.expect, Expect::Error(_))) && !has_main(&b.code) {
                wrap_fragment(&b.code)
            } else {
                b.code.clone()
            };
            let file = tmp.join(format!("{}_{}.ail", b.chapter.trim_end_matches(".md"), b.line));
            fs::write(&file, &code).unwrap();
            let out = Command::new(env!("CARGO_BIN_EXE_ail"))
                .args(["check", "--json"])
                .args(&b.flags)
                .arg(&file)
                .output()
                .unwrap();
            let stderr = String::from_utf8_lossy(&out.stderr);
            let errors: Vec<&str> = stderr
                .lines()
                .filter(|l| l.contains("\"severity\":\"error\""))
                .filter_map(|l| l.split("\"rule\":\"").nth(1))
                .map(|r| r.split('"').next().unwrap())
                .collect();
            match &b.expect {
                Expect::Clean if !errors.is_empty() => {
                    ok = false;
                    failures.push(format!("{at}: expected clean, got errors {errors:?}\n{stderr}"))
                }
                Expect::Error(r) if !errors.contains(&r.as_str()) => {
                    ok = false;
                    failures.push(format!("{at}: expected error {r}, got {errors:?}\n{stderr}"))
                }
                _ => {}
            }
            if ok && b.run && b.flags.is_empty() && matches!(b.expect, Expect::Clean) && has_main(&code) {
                if let Err(e) = run_both_tiers(&file) {
                    ok = false;
                    failures.push(format!("{at}: {e}"));
                }
            }
        }
        if ok {
            for r in &b.rules {
                if !spec_ids.iter().any(|(id, _)| id == r) {
                    failures.push(format!("{at}: example cites {r}, which no chapter defines"));
                }
                covered.insert(r.clone());
            }
        }
    }
    eprintln!("spec examples: {run} checked, {skipped} skipped (skip=bootstrap)");
    assert!(failures.is_empty(), "{} spec example(s) failed:\n\n{}", failures.len(), failures.join("\n\n"));
    // Each rule ID the chapter files define that no example demonstrates, for the batch chapters
    // (ADR 0002). `AIL_LIST_UNCOVERED=1` prints the list for every chapter instead.
    let uncovered: Vec<&(String, String)> = spec_ids.iter().filter(|(id, _)| !covered.contains(id)).collect();
    if std::env::var_os("AIL_LIST_UNCOVERED").is_some() {
        for (id, file) in &uncovered {
            eprintln!("uncovered {file} {id}");
        }
    }
    let batch: Vec<String> = uncovered
        .iter()
        .filter(|(_, file)| BATCH_CHAPTERS.contains(&file.as_str()))
        .map(|(id, file)| format!("{file} {id}"))
        .collect();
    assert!(
        batch.is_empty(),
        "{} rule(s) in batch chapters have no runnable example:\n{}",
        batch.len(),
        batch.join("\n")
    );
}
