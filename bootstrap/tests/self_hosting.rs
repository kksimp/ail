//! The self-hosted front end against the bootstrap (ADR 0371): the syntax
//! dumps (api.tok.1-3, api.ast.1-3) and, file by file over every `.ail`
//! file in the repository, the AIL lexer and parser.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn ail() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ail"))
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("self_hosting").join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Every `.ail` file in the repository, sorted: the corpus each stage is
/// compared on.
fn corpus() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else { return };
        for e in entries {
            let p = e.unwrap().path();
            let name = p.file_name().unwrap().to_string_lossy().to_string();
            if p.is_dir() {
                if !matches!(name.as_str(), "target" | ".git" | ".toolchain" | ".verify") {
                    walk(&p, out);
                }
            } else if name.ends_with(".ail") {
                out.push(p);
            }
        }
    }
    let mut files = Vec::new();
    walk(&root(), &mut files);
    files.sort();
    files
}

fn run(cmd: &mut Command) -> String {
    let out = cmd.output().unwrap();
    assert!(out.status.success(), "{cmd:?}: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn tokens_list_one_token_per_line() {
    let hello = root().join("examples/hello.ail");
    let text = run(ail().args(["api", "--tokens"]).arg(&hello));
    assert_eq!(
        text,
        "1:1 kw fn\n1:4 id main\n1:8 sym (\n1:9 sym )\n1:10 nl\n2:1 indent\n2:5 id print\n2:10 sym (\n\
         2:11 str \"Hello, world!\"\n2:26 sym )\n2:27 nl\n3:1 dedent\n3:1 eof\n"
    );
    // api.tok.2: interpolation parts, a format specification, comments and diagnostics.
    let dir = scratch("tokens");
    let f = dir.join("parts.ail");
    fs::write(&f, "x := \"a{b:>4}\" # note\ny := 0x\n").unwrap();
    let text = run(ail().args(["api", "--tokens"]).arg(&f));
    assert_eq!(
        text,
        "1:1 id x\n1:3 sym :=\n1:6 interp\nlit \"a\"\n\
         fmt {\"fill\":null,\"align\":\">\",\"width\":4.0,\"precision\":null,\"hex\":false}\n\
         1:9 id b\n1:10 eof\nend\n1:22 nl\n2:1 id y\n2:3 sym :=\n2:6 int 0\n2:8 nl\n3:1 eof\n\
         1:16 comment \"# note\"\n2:6 diag lex.num.1\n"
    );
}

#[test]
fn ast_dumps_read_back_through_the_ail_syntax_tree() {
    let dir = scratch("ast");
    let mut n = 0;
    for f in corpus() {
        let text = run(ail().args(["api", "--ast"]).arg(&f));
        assert!(text.starts_with("{\"fns\":"), "{}: {text}", f.display());
        fs::write(dir.join(format!("{n:04}.json")), text).unwrap();
        n += 1;
    }
    // api.ast.3: the whole program, prelude and derive output included.
    for f in ["examples/json.ail", "tools/edit/edit.ail", "bootstrap/tests/cases/packages/geometry"] {
        let text = run(ail().args(["api", "--ast", "--program"]).arg(root().join(f)));
        assert!(text.contains("\"line\":10737418"), "{f}: no prelude");
        fs::write(dir.join(format!("{n:04}.json")), text).unwrap();
        n += 1;
    }
    let out = run(ail().arg("run").arg(root().join("bootstrap/tests/selfhost/roundtrip")).arg(&dir));
    assert_eq!(out, format!("checked {n}\n"));
    assert!(n > 700, "{n} files");
}

/// Runs the AIL front end (`ail run compiler COMMAND`) once over the
/// whole corpus and compares each file's output with `ail api FLAG FILE`.
fn compare_with_bootstrap(command: &str, flag: &str) {
    let files = corpus();
    let mut expected = String::new();
    for f in &files {
        expected.push_str(&format!("== {}\n", f.display()));
        expected.push_str(&run(ail().args(["api", flag]).arg(f)));
    }
    let actual = run(ail().arg("run").arg(root().join("compiler")).arg(command).args(&files));
    if actual != expected {
        // Report the first file whose output differs.
        let split = |s: &str| s.split("\n== ").map(str::to_string).collect::<Vec<_>>();
        for (a, e) in split(&actual).iter().zip(split(&expected).iter()) {
            assert_eq!(a, e, "`ail run compiler {command}` differs from `ail api {flag}`");
        }
        panic!("`ail run compiler {command}` gave {} bytes, `ail api {flag}` {}", actual.len(), expected.len());
    }
    assert!(files.len() > 700, "{} files", files.len());
}

/// The AIL lexer lists the same tokens, comments and diagnostics as the
/// bootstrap's on every file (ADR 0371).
#[test]
fn ail_lexer_matches_the_bootstrap() {
    compare_with_bootstrap("tokens", "--tokens");
}
