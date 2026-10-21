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

/// `path` relative to the repository root, with `/` separators.
fn rel_path(path: &Path) -> String {
    path.strip_prefix(root()).unwrap().to_string_lossy().replace('\\', "/")
}

/// The target `ail build` picks when none is given.
fn host_target() -> &'static str {
    if cfg!(target_os = "macos") {
        "aarch64-macos"
    } else {
        "x86_64-linux"
    }
}

fn write_exe(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

/// Hexadecimal text as bytes; whitespace is skipped.
fn unhex(text: &str) -> Option<Vec<u8>> {
    let digits: Vec<u8> = text.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if digits.len() % 2 != 0 {
        return None;
    }
    digits.chunks(2).map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok()).collect()
}

/// ADR 0814: the executables of `files` (relative to the root, in order),
/// built in one `ail run compiler build-many` so the compiler loads once. Each
/// file is checked first, as `ail build` does. `None` when any file fails; the
/// caller then builds each file on its own, which reports the failure per file.
fn build_batch(files: &[String], target: Option<&str>) -> Option<Vec<Vec<u8>>> {
    for rel in files {
        let checked =
            Command::new(env!("CARGO_BIN_EXE_ail")).current_dir(root()).args(["check", rel]).output().unwrap();
        if !checked.status.success() {
            return None;
        }
    }
    let out = Command::new(env!("CARGO_BIN_EXE_ail"))
        .current_dir(root())
        .args(["run", "compiler", "build-many", "compiler/rt/native", target.unwrap_or(host_target()), "0", "0"])
        .arg(root().join("bootstrap/src/std"))
        .args(files)
        .output()
        .unwrap();
    if !out.status.success() {
        return None;
    }
    // Each program's output starts with `== PATH`, then its hex (native.build.3).
    let text = String::from_utf8(out.stdout).ok()?;
    let exes: Vec<Vec<u8>> = text
        .split("== ")
        .skip(1)
        .map(|s| unhex(s.split_once('\n').map_or("", |(_, hex)| hex)))
        .collect::<Option<_>>()?;
    (exes.len() == files.len()).then_some(exes)
}

/// ADR 0814: a batch build gives the same bytes as one `ail build` per program.
#[test]
#[cfg(any(all(target_os = "linux", target_arch = "x86_64"), all(target_os = "macos", target_arch = "aarch64")))]
fn batch_builds_match_separate_builds() {
    let dir = root().join("bootstrap/tests/native/run");
    let mut all: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "ail"))
        .collect();
    all.sort();
    // Five programs spread over the directory.
    let sample: Vec<String> = all.iter().step_by(all.len() / 5).map(|p| rel_path(p)).collect();
    let batch = build_batch(&sample, None).expect("the batch build failed");
    let tmp = std::env::temp_dir().join(format!("ail-batch-{}", std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    for (i, rel) in sample.iter().enumerate() {
        let exe = tmp.join(format!("{i}"));
        let built = Command::new(env!("CARGO_BIN_EXE_ail"))
            .current_dir(root())
            .args(["build", rel, "-o"])
            .arg(&exe)
            .output()
            .unwrap();
        assert!(built.status.success(), "{rel}: `ail build` failed");
        assert!(fs::read(&exe).unwrap() == batch[i], "{rel}: the batch build differs from `ail build`");
    }
    let _ = fs::remove_dir_all(&tmp);
}

/// The interpreter's report line as the native one prints it (trace.frame.1,
/// ADR 0641): a frame's values and locals are left out, `f(a = 1)` being
/// `f(..)`, and the elision line has no `--verbose` hint. The interpreter's
/// `note:` and `help:` lines have no native counterpart.
fn interp_line(l: &str) -> Option<String> {
    if l.starts_with("note:") || l.starts_with("help:") || l.starts_with("  values:") {
        return None;
    }
    if l.starts_with("  at ") {
        if let Some(i) = l.find(" in ") {
            let name = &l[i + 4..];
            let open = name.find('(')?;
            let args = if name[open..].starts_with("()") { "()" } else { "(..)" };
            return Some(format!("{}{}{}", &l[..i + 4], &name[..open], args));
        }
    }
    if l.starts_with("  \u{2026} ") {
        return Some(l.split(" (--verbose").next().unwrap().to_string());
    }
    Some(l.to_string())
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
    let mut cases: Vec<PathBuf> = cases.into_iter().filter(|p| p.extension().is_some_and(|x| x == "ail")).collect();
    // On macOS two groups of programs differ from `ail run` for known reasons:
    // `proc.run` is not built there yet (native.subset.1), and the native tanh
    // (fdlibm) is within 1 ulp of the macOS libm tanh that the interpreter calls
    // (ADR 0592). They run in the Linux pass.
    if cfg!(target_os = "macos") || target.is_some_and(|t| t.contains("macos")) {
        const MACOS_KNOWN: &[&str] =
            &["os_err_fields", "os_error_values", "proc_output_methods", "proc_run", "tensor_tanh"];
        cases.retain(|p| !MACOS_KNOWN.contains(&p.file_stem().unwrap().to_string_lossy().as_ref()));
    }
    let tmp = std::env::temp_dir().join(format!("ail-native-{}-{}", target.unwrap_or("host"), std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    // Four workers take programs in turn. Each failure is kept with its index,
    // so the report reads in the same sorted order as a serial run.
    let next = AtomicUsize::new(0);
    // ADR 0814 step 3: each quarter of the programs is built in one
    // `build-many` run, so the compiler loads four times, not once per program.
    // A quarter whose batch fails is built per program in the workers below.
    let rels: Vec<String> = cases.iter().map(|p| rel_path(p)).collect();
    let quarter = rels.len().div_ceil(4);
    let prebuilt: Vec<Option<Vec<u8>>> = std::thread::scope(|s| {
        let batches: Vec<_> = rels.chunks(quarter).map(|chunk| s.spawn(move || build_batch(chunk, target))).collect();
        let mut out = Vec::new();
        for (chunk, batch) in rels.chunks(quarter).zip(batches) {
            match batch.join().unwrap() {
                Some(exes) => out.extend(exes.into_iter().map(Some)),
                None => out.extend(std::iter::repeat_with(|| None).take(chunk.len())),
            }
        }
        out
    });
    let mut failures: Vec<(usize, String)> = std::thread::scope(|s| {
        let workers: Vec<_> = (0..4)
            .map(|_| {
                s.spawn(|| {
                    let mut found = Vec::new();
                    loop {
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        let Some(case) = cases.get(i) else { break };
                        if let Some(failure) = executable_matches(case, &tmp, target, runner, prebuilt[i].as_deref()) {
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
fn executable_matches(
    case: &Path,
    tmp: &Path,
    target: Option<&str>,
    runner: Option<&str>,
    prebuilt: Option<&[u8]>,
) -> Option<String> {
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
    match prebuilt {
        Some(bytes) => write_exe(&exe, bytes),
        None => {
            let mut build = Command::new(env!("CARGO_BIN_EXE_ail"));
            build.current_dir(root()).args(["build", &rel, "-o"]).arg(&exe);
            if let Some(t) = target {
                build.args(["--target", t]);
            }
            let built = build.output().unwrap();
            if !built.status.success() {
                return Some(format!("{rel}: `ail build` failed:\n{}", String::from_utf8_lossy(&built.stderr)));
            }
        }
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
    let want_err: Vec<String> = std::str::from_utf8(&want.stderr).unwrap().lines().filter_map(interp_line).collect();
    let got_err: Vec<String> = std::str::from_utf8(&got.stderr).unwrap().lines().map(str::to_string).collect();
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

/// Each `tests/native/stack/NAME.ail` against `NAME.out` and the first line
/// of `NAME.err`: recursion bounded by the stack rather than the
/// interpreter's call count (err.trap.1, ADR 0535).
fn stack_cases(target: Option<&str>, runner: Option<&str>) {
    let dir = root().join("bootstrap/tests/native/stack");
    let tmp = std::env::temp_dir().join(format!("ail-native-stack-{}-{}", target.unwrap_or("host"), std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    let mut failures = Vec::new();
    let mut cases: Vec<PathBuf> = fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).collect();
    cases.sort();
    for case in cases.iter().filter(|p| p.extension().is_some_and(|x| x == "ail")) {
        let rel = case.strip_prefix(root()).unwrap().to_string_lossy().replace('\\', "/");
        let exe = tmp.join(case.file_stem().unwrap());
        let mut build = Command::new(env!("CARGO_BIN_EXE_ail"));
        build.current_dir(root()).args(["build", &rel, "-o"]).arg(&exe);
        if let Some(t) = target {
            build.args(["--target", t]);
        }
        let built = build.output().unwrap();
        if !built.status.success() {
            failures.push(format!("{rel}: `ail build` failed:\n{}", String::from_utf8_lossy(&built.stderr)));
            continue;
        }
        let got = match runner {
            Some(r) => Command::new(r).arg(&exe).current_dir(root()).output().unwrap(),
            None => Command::new(&exe).current_dir(root()).output().unwrap(),
        };
        let want_out = fs::read_to_string(case.with_extension("out")).unwrap();
        let want_err = fs::read_to_string(case.with_extension("err")).unwrap();
        let got_err = String::from_utf8_lossy(&got.stderr);
        if String::from_utf8_lossy(&got.stdout) != want_out
            || got.status.code() != Some(1)
            || got_err.lines().next() != want_err.lines().next()
        {
            failures.push(format!(
                "{rel}: (exit {:?})\n{}{}",
                got.status.code(),
                String::from_utf8_lossy(&got.stdout),
                got_err
            ));
        }
    }
    let _ = fs::remove_dir_all(&tmp);
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

// err.trap.1
#[test]
#[cfg(any(all(target_os = "linux", target_arch = "x86_64"), all(target_os = "macos", target_arch = "aarch64")))]
fn native_recursion_traps_when_the_stack_runs_out() {
    stack_cases(None, None);
}

/// The same under qemu for Linux ARM64, when an emulator is installed.
#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn arm64_recursion_traps_when_the_stack_runs_out_under_qemu() {
    let runner = ["qemu-aarch64-static", "qemu-aarch64"]
        .into_iter()
        .find(|q| Command::new(q).arg("--version").output().is_ok_and(|o| o.status.success()));
    match runner {
        Some(q) => stack_cases(Some("aarch64-linux"), Some(q)),
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

/// A task program builds for macOS: its threads come from libSystem, so the
/// Mach-O file carries a bind table naming `pthread_create` (native.task.1,
/// ADR 0599); a program with no threads imports nothing.
#[test]
fn tasks_build_for_macos_with_imports() {
    let tmp = std::env::temp_dir().join(format!("ail-native-macos-{}", std::process::id()));
    let build = |name: &str| {
        let out = Command::new(env!("CARGO_BIN_EXE_ail"))
            .current_dir(root())
            .args(["build", &format!("bootstrap/tests/native/run/{name}.ail"), "--target", "aarch64-macos", "-o"])
            .arg(&tmp)
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        fs::read(&tmp).unwrap()
    };
    let has = |bytes: &[u8], needle: &[u8]| bytes.windows(needle.len()).any(|w| w == needle);
    assert!(has(&build("task_basic"), b"_pthread_create\0"));
    assert!(!has(&build("arith"), b"_pthread_create\0"));
    let _ = fs::remove_file(&tmp);
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
        let want = Command::new(env!("CARGO_BIN_EXE_ail"))
            .current_dir(root())
            .args(["run", option, value, &src])
            .output()
            .unwrap();
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
