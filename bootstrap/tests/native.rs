//! The native backend in AIL (`compiler/`, ADRs 0384-0393), driven through
//! `ail run compiler`: each `tests/native/NAME.ail` has the MIR it lowers to
//! (`NAME.mir`, ir-architecture.md §4) or the problems it reports
//! (`NAME.err`, native.subset.1).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn cases() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(root().join("bootstrap/tests/native"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "ail"))
        .collect();
    out.sort();
    out
}

/// `ail run compiler ARGS` from the repository root.
fn compiler(args: &[&str]) -> (bool, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_ail"))
        .current_dir(root())
        .args(["run", "compiler"])
        .args(args)
        .output()
        .unwrap();
    (out.status.success(), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
}

// native.subset.1, native.build.2
#[test]
fn mir_matches_the_expected_dumps() {
    let mut failures = Vec::new();
    for case in cases() {
        let rel = case.strip_prefix(root()).unwrap().to_string_lossy().replace('\\', "/");
        let (ok, out, err) = compiler(&["mir", &rel]);
        let mir = case.with_extension("mir");
        let errs = case.with_extension("err");
        let (want, got) = if mir.exists() {
            if !ok {
                failures.push(format!("{rel}: failed:\n{err}"));
                continue;
            }
            (fs::read_to_string(&mir).unwrap(), out)
        } else {
            let problems: Vec<&str> = err.lines().filter(|l| l.contains(": error[native.")).collect();
            if ok {
                failures.push(format!("{rel}: expected problems, but it lowered"));
                continue;
            }
            (fs::read_to_string(&errs).unwrap_or_default(), problems.join("\n") + "\n")
        };
        if want != got {
            failures.push(format!("{rel}: differs from the expected output; got:\n{got}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// ADR 0722: each `native.subset.1` problem of `subset_help.ail` is followed by
/// its `help:` line, exactly as in `subset_help.help`.
#[test]
fn subset_problems_print_their_help_lines() {
    let case = root().join("bootstrap/tests/native/subset_help.ail");
    let rel = case.strip_prefix(root()).unwrap().to_string_lossy().replace('\\', "/");
    let (_, _, err) = compiler(&["mir", &rel]);
    let want = fs::read_to_string(case.with_extension("help")).unwrap();
    let mut lines = Vec::new();
    let mut prev_is_problem = false;
    for l in err.lines() {
        if l.starts_with(&rel) {
            lines.push(l);
            prev_is_problem = true;
        } else if prev_is_problem && l.starts_with("help:") {
            lines.push(l);
            prev_is_problem = false;
        } else {
            prev_is_problem = false;
        }
    }
    assert_eq!(lines.join("\n") + "\n", want, "help lines differ");
}

/// `ail build` on each `tests/native/run/NAME.ail`, then the executable's
/// output against `ail run`'s: stdout, the exit code, and a trap's message
/// and excerpt (the interpreter's snapshot lines aside, trace.snap.1).
/// `target` is passed to `--target`; `runner` runs the executable (an
/// emulator), or it runs directly.
fn executables_match(target: Option<&str>, runner: Option<&str>) {
    let dir = root().join("bootstrap/tests/native/run");
    let mut cases: Vec<PathBuf> = fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).collect();
    cases.sort();
    let cases: Vec<PathBuf> = cases.into_iter().filter(|p| p.extension().is_some_and(|x| x == "ail")).collect();
    let tmp = std::env::temp_dir().join(format!("ail-native-{}-{}", target.unwrap_or("host"), std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    // Four workers take programs in turn. Each failure is kept with its index,
    // so the report reads in the same sorted order as a serial run.
    let next = AtomicUsize::new(0);
    let mut failures: Vec<(usize, String)> = std::thread::scope(|s| {
        let workers: Vec<_> = (0..4)
            .map(|_| {
                s.spawn(|| {
                    let mut found = Vec::new();
                    loop {
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        let Some(case) = cases.get(i) else { break };
                        if let Some(failure) = executable_matches(case, &tmp, target, runner) {
                            found.push((i, failure));
                        }
                    }
                    found
                })
            })
            .collect();
        workers.into_iter().flat_map(|w| w.join().unwrap()).collect()
    });
    failures.sort_by_key(|(i, _)| *i);
    let _ = fs::remove_dir_all(&tmp);
    let failures: Vec<String> = failures.into_iter().map(|(_, f)| f).collect();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// Builds one `tests/native/run` program and compares its executable with
/// `ail run`. Returns the failure report, or `None` when they agree. Every
/// path it writes is named after the program, so programs can run at once.
fn executable_matches(case: &Path, tmp: &Path, target: Option<&str>, runner: Option<&str>) -> Option<String> {
    let rel = case.strip_prefix(root()).unwrap().to_string_lossy().replace('\\', "/");
    let stem = case.file_stem().unwrap();
    let exe = tmp.join(stem);
    // Scratch paths for the `os_*` programs, which can't build a path yet
    // (std.fs, ADR 0580): a file, a second file, a directory with a nested
    // one, and a file inside it. Each program gets its own directory.
    let scratch = tmp.join(format!("scratch-{}", stem.to_string_lossy()));
    fs::create_dir_all(&scratch).unwrap();
    let scratch_env = [
        ("AIL_TEST_A", scratch.join("a.txt")),
        ("AIL_TEST_B", scratch.join("b.txt")),
        ("AIL_TEST_DIR", scratch.join("d1")),
        ("AIL_TEST_MID", scratch.join("d1/d2")),
        ("AIL_TEST_DEEP", scratch.join("d1/d2/d3")),
        ("AIL_TEST_IN", scratch.join("d1/f.txt")),
    ];
    let mut build = Command::new(env!("CARGO_BIN_EXE_ail"));
    build.current_dir(root()).args(["build", &rel, "-o"]).arg(&exe);
    if let Some(t) = target {
        build.args(["--target", t]);
    }
    let built = build.output().unwrap();
    if !built.status.success() {
        return Some(format!("{rel}: `ail build` failed:\n{}", String::from_utf8_lossy(&built.stderr)));
    }
    // Standard input is `NAME.in` when there is one, else empty.
    let input = || match fs::File::open(case.with_extension("in")) {
        Ok(f) => Stdio::from(f),
        Err(_) => Stdio::null(),
    };
    let want = Command::new(env!("CARGO_BIN_EXE_ail"))
        .current_dir(root())
        .envs(scratch_env.clone())
        .stdin(input())
        .args(["run", &rel])
        .output()
        .unwrap();
    let got = match runner {
        Some(r) => {
            Command::new(r).arg(&exe).current_dir(root()).envs(scratch_env.clone()).stdin(input()).output().unwrap()
        }
        None => Command::new(&exe).current_dir(root()).envs(scratch_env.clone()).stdin(input()).output().unwrap(),
    };
    let snapshot =
        |l: &&str| !(l.starts_with("  at ") && l.contains(" in ") || l.starts_with("note:") || l.starts_with("help:"));
    let want_err: Vec<&str> = std::str::from_utf8(&want.stderr).unwrap().lines().filter(snapshot).collect();
    let got_err: Vec<&str> = std::str::from_utf8(&got.stderr).unwrap().lines().collect();
    if want.stdout == got.stdout && want.status.code() == got.status.code() && want_err == got_err {
        return None;
    }
    Some(format!(
        "{rel}: differs from `ail run`\n--- ail run (exit {:?}):\n{}{}\n--- native (exit {:?}):\n{}{}",
        want.status.code(),
        String::from_utf8_lossy(&want.stdout),
        want_err.join("\n"),
        got.status.code(),
        String::from_utf8_lossy(&got.stdout),
        got_err.join("\n"),
    ))
}

// native.build.1, expr.arith.1, expr.arith.3, type.coll.4, type.num.9, expr.bit.2
#[test]
#[cfg(any(all(target_os = "linux", target_arch = "x86_64"), all(target_os = "macos", target_arch = "aarch64")))]
fn executables_match_the_interpreter() {
    executables_match(None, None);
}

/// The same for Linux ARM64 executables, when a qemu user-mode emulator is
/// installed (`qemu-aarch64-static` or `qemu-aarch64`); otherwise skipped.
#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn arm64_executables_match_the_interpreter_under_qemu() {
    let runner = ["qemu-aarch64-static", "qemu-aarch64"]
        .into_iter()
        .find(|q| Command::new(q).arg("--version").output().is_ok_and(|o| o.status.success()));
    match runner {
        Some(q) => executables_match(Some("aarch64-linux"), Some(q)),
        None => eprintln!("skipped: no qemu-aarch64 emulator"),
    }
}

/// A program outside the native subset builds nothing (native.subset.1).
#[test]
fn build_refuses_programs_outside_the_subset() {
    let tmp = std::env::temp_dir().join(format!("ail-native-refused-{}", std::process::id()));
    let out = Command::new(env!("CARGO_BIN_EXE_ail"))
        .current_dir(root())
        .args(["build", "bootstrap/tests/native/subset.ail", "-o"])
        .arg(&tmp)
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success() && !tmp.exists(), "built a program outside the subset");
    assert!(err.contains("error[native.subset.1]"), "{err}");
}

/// `compiler/rt/native/unicode.ail` is what `tools/native/gen_unicode.ail`
/// writes from the interpreter's tables (native.str.1, ADR 0567), so a
/// toolchain with newer Unicode data fails here until the file is regenerated.
#[test]
fn unicode_tables_are_current() {
    let out = Command::new(env!("CARGO_BIN_EXE_ail"))
        .current_dir(root())
        .args(["run", "tools/native/gen_unicode.ail"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let have = fs::read_to_string(root().join("compiler/rt/native/unicode.ail")).unwrap();
    assert!(
        String::from_utf8(out.stdout).unwrap() == have,
        "compiler/rt/native/unicode.ail is stale: run `./ail run tools/native/gen_unicode.ail > compiler/rt/native/unicode.ail`"
    );
}

/// run.limit.5, run.limit.6: a native program stops at the limits of
/// `AIL_TIMEOUT_MS`, `AIL_MAX_MEMORY_MB` and the defaults `ail build` bakes,
/// with the trap line `ail run` prints (less its position), after its output
/// so far, and exit status 1. `runner` runs the executable (an emulator).
fn limits_stop_native_programs(target: Option<&str>, runner: Option<&str>) {
    let tmp = std::env::temp_dir().join(format!("ail-limits-{}-{}", target.unwrap_or("host"), std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    // The trap line without its `FILE:LINE:COL: ` prefix.
    let message = |stderr: &[u8]| {
        let text = String::from_utf8_lossy(stderr).into_owned();
        let line = text.lines().find(|l| l.contains("trap[")).unwrap_or("").to_string();
        line.find("trap[").map(|i| line[i..].to_string()).unwrap_or_default()
    };
    // (program, `ail run` option, value, environment variable, bake flag)
    for (name, option, value, var, flag) in [
        ("grow", "--max-memory", "16", "AIL_MAX_MEMORY_MB", "--max-memory"),
        ("spin", "--timeout", "300", "AIL_TIMEOUT_MS", "--timeout"),
    ] {
        let src = format!("bootstrap/tests/native/limits/{name}.ail");
        let want = Command::new(env!("CARGO_BIN_EXE_ail")).current_dir(root()).args(["run", option, value, &src]).output().unwrap();
        for baked in [false, true] {
            let exe = tmp.join(format!("{name}{}", if baked { "-baked" } else { "" }));
            let mut build = Command::new(env!("CARGO_BIN_EXE_ail"));
            build.current_dir(root()).args(["build", &src, "-o"]).arg(&exe);
            if let Some(t) = target {
                build.args(["--target", t]);
            }
            if baked {
                build.args([flag, value]);
            }
            let built = build.output().unwrap();
            assert!(built.status.success(), "{src}: `ail build` failed:\n{}", String::from_utf8_lossy(&built.stderr));
            let mut run = match runner {
                Some(r) => {
                    let mut c = Command::new(r);
                    c.arg(&exe);
                    c
                }
                None => Command::new(&exe),
            };
            if !baked {
                run.env(var, value);
            }
            let got = run.output().unwrap();
            let label = format!("{src} ({}{})", var, if baked { ", baked" } else { "" });
            assert_eq!(got.status.code(), Some(1), "{label}: exit status");
            assert_eq!(got.stdout, want.stdout, "{label}: stdout");
            assert_eq!(message(&got.stderr), message(&want.stderr), "{label}: trap line");
        }
    }
    // run.limit.4: an invalid value in the environment ends the program with status 2.
    let exe = tmp.join("spin");
    let mut run = match runner {
        Some(r) => {
            let mut c = Command::new(r);
            c.arg(&exe);
            c
        }
        None => Command::new(&exe),
    };
    let bad = run.env("AIL_TIMEOUT_MS", "soon").output().unwrap();
    assert_eq!(bad.status.code(), Some(2), "a bad AIL_TIMEOUT_MS");
    let _ = fs::remove_dir_all(&tmp);
}

// run.limit.5, run.limit.6
#[test]
#[cfg(any(all(target_os = "linux", target_arch = "x86_64"), all(target_os = "macos", target_arch = "aarch64")))]
fn run_limits_stop_native_programs() {
    limits_stop_native_programs(None, None);
}

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn run_limits_stop_arm64_programs_under_qemu() {
    let runner = ["qemu-aarch64-static", "qemu-aarch64"]
        .into_iter()
        .find(|q| Command::new(q).arg("--version").output().is_ok_and(|o| o.status.success()));
    match runner {
        Some(q) => limits_stop_native_programs(Some("aarch64-linux"), Some(q)),
        None => eprintln!("skipped: no qemu-aarch64 emulator"),
    }
}

/// Every runtime source parses as AIL (gram.*): the self-hosted pipeline
/// compiles `compiler/rt/native` on its own, and `ail fmt --check DIR` skips
/// files with syntax errors, so a foreign token there (`&` for `bit_and`)
/// used to vanish silently and drop the rest of its statement from the
/// native code. Naming each file makes `fmt --check` report it (fmt.safe).
#[test]
fn runtime_sources_parse() {
    let dir = root().join("compiler/rt/native");
    let mut bad = Vec::new();
    for e in std::fs::read_dir(&dir).unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_some_and(|x| x == "ail") {
            let out = Command::new(env!("CARGO_BIN_EXE_ail")).args(["fmt", "--check"]).arg(&p).output().unwrap();
            if out.status.code() == Some(2) {
                bad.push(format!("{}: {}", p.display(), String::from_utf8_lossy(&out.stderr)));
            }
        }
    }
    assert!(bad.is_empty(), "runtime sources with syntax errors:\n{}", bad.join("\n"));
}

