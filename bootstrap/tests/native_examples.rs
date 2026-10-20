//! The native example programs (`examples/native_*.ail`, lane 186): each one
//! builds with `ail build` and its executable prints exactly `NAME.out`, the
//! output of `ail run` on the same file (native.build.1). The examples are
//! checked by the interpreter in `cases.rs`; this test checks the executables.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// native.build.1, native.build.3
#[test]
#[cfg(any(all(target_os = "linux", target_arch = "x86_64"), all(target_os = "macos", target_arch = "aarch64")))]
fn native_examples_print_their_expected_output() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut cases: Vec<PathBuf> = fs::read_dir(root.join("examples"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "ail"))
        .filter(|p| p.file_stem().is_some_and(|s| s.to_string_lossy().starts_with("native_")))
        .collect();
    cases.sort();
    assert!(!cases.is_empty(), "no examples/native_*.ail found");
    let tmp = std::env::temp_dir().join(format!("ail-native-examples-{}", std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    let mut failures = Vec::new();
    for case in &cases {
        let rel = case.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/");
        let exe = tmp.join(case.file_stem().unwrap());
        let built = Command::new(env!("CARGO_BIN_EXE_ail"))
            .current_dir(&root)
            .args(["build", &rel, "-o"])
            .arg(&exe)
            .output()
            .unwrap();
        if !built.status.success() {
            failures.push(format!("{rel}: `ail build` failed:\n{}", String::from_utf8_lossy(&built.stderr)));
            continue;
        }
        let got = Command::new(&exe).current_dir(&root).output().unwrap();
        let want = fs::read_to_string(case.with_extension("out")).unwrap();
        if !got.status.success() || String::from_utf8_lossy(&got.stdout) != want {
            failures.push(format!(
                "{rel}: the executable's output (exit {:?}) differs from {}:\n{}{}",
                got.status.code(),
                case.with_extension("out").display(),
                String::from_utf8_lossy(&got.stdout),
                String::from_utf8_lossy(&got.stderr),
            ));
        }
    }
    let _ = fs::remove_dir_all(&tmp);
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
