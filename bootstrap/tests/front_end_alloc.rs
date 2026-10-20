//! The front-end allocator (ADR 0332) must not change any output: `ail check`
//! and `ail api` print the same bytes and exit the same way with it and with
//! the system allocator (`AIL_SYSTEM_ALLOC=1`), on every example and case,
//! accepted or rejected, derive generators included.

use std::path::Path;
use std::process::{Command, Output};

fn ail(args: &[&str], system: bool) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ail"));
    cmd.args(args).current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/.."));
    if system {
        cmd.env("AIL_SYSTEM_ALLOC", "1");
    }
    cmd.output().unwrap()
}

fn programs() -> Vec<String> {
    let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/.."));
    let mut out = vec!["tools/edit/edit.ail".to_string()];
    for dir in ["examples", "bootstrap/tests/cases"] {
        let mut files: Vec<String> = std::fs::read_dir(root.join(dir))
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".ail"))
            .map(|n| format!("{dir}/{n}"))
            .collect();
        files.sort();
        out.extend(files);
    }
    out
}

#[test]
fn front_end_allocator_output_is_byte_identical() {
    let mut differ = Vec::new();
    for program in programs() {
        for args in [vec!["check", &program], vec!["api", "--symbols", &program]] {
            let (fast, plain) = (ail(&args, false), ail(&args, true));
            if fast.stdout != plain.stdout || fast.stderr != plain.stderr || fast.status.code() != plain.status.code() {
                differ.push(args.join(" "));
            }
        }
    }
    assert!(differ.is_empty(), "output differs under the front-end allocator: {differ:?}");
}

#[test]
fn effects_on_the_editor_is_byte_identical() {
    let args = ["effects", "tools/edit/edit.ail"];
    let (fast, plain) = (ail(&args, false), ail(&args, true));
    assert!(fast.status.success(), "{}", String::from_utf8_lossy(&fast.stderr));
    assert_eq!(fast.stdout, plain.stdout);
}
