//! `ail build FILE [-o OUT]` and `ail api --mir FILE`: drivers for the
//! native backend, which is written in AIL under `compiler/` (ADRs 0384,
//! 0385). This runs it on the interpreter, then writes the executable it
//! prints as hexadecimal (native.build.3); bytes go through text until the
//! standard library writes them.

use crate::build_cache;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// The repository's `compiler/` directory: `AIL_COMPILER`, or the one beside
/// the `bootstrap/target/PROFILE/ail` that is running.
fn compiler_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("AIL_COMPILER") {
        return Some(PathBuf::from(dir));
    }
    // `bootstrap/target/PROFILE/ail`, or `bootstrap/target/TRIPLE/PROFILE/ail`
    // for a cross build: the first ancestor with a `compiler/` directory.
    let exe = std::env::current_exe().ok()?;
    exe.ancestors().skip(1).take(6).map(|a| a.join("compiler")).find(|d| d.join("main.ail").exists())
}

/// The repository root, beside `compiler/` (or the current directory).
pub fn repo_root() -> PathBuf {
    compiler_dir()
        .and_then(|d| d.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// The methods derive generators wrote for the program at `file`
/// (ct.gen.1), in a file for the native backend (ADR 0807): for each
/// deriving `impl`, `@@derive MODULE TYPE.TRAIT` (`-` for the root module)
/// and the generated text. When the program's text names JSON or
/// `Default`, a last block `@@prelude` holds the prelude, whose `json`,
/// `jsonderive` and `default` modules the backend lowers as the program
/// reaches them (ADR 0808). `None` when there is neither.
fn derived_file(file: &str) -> Option<PathBuf> {
    let (prog, _) = crate::modules::load(file).ok()?;
    let prelude = prog.files.iter().any(|f| ["json", "Json", "Default"].iter().any(|w| f.src.contains(w)));
    if prog.generated.is_empty() && !prelude {
        return None;
    }
    let mut text = String::new();
    for g in &prog.generated {
        let f = &prog.files[g.file];
        let module = if f.module.is_empty() { "-" } else { &f.module };
        text.push_str(&format!("@@derive {module} {}\n{}\n", g.label, f.src.trim_end()));
    }
    if prelude {
        text.push_str("@@prelude\n");
        text.push_str(crate::parser::PRELUDE);
    }
    let path = std::env::temp_dir().join(format!("ail-derived-{}.txt", std::process::id()));
    std::fs::write(&path, text).ok()?;
    Some(path)
}

/// Run `ail run compiler ARGS`; stdout on success. The compiler's own
/// warnings are dropped; its errors are about the user's program.
fn run_compiler(args: &[&str]) -> Result<String, ExitCode> {
    let derived = args.get(1).and_then(|f| derived_file(f));
    let out = run_compiler_with(args, derived.as_deref());
    if let Some(p) = derived {
        let _ = std::fs::remove_file(p);
    }
    out
}

fn run_compiler_with(args: &[&str], derived: Option<&Path>) -> Result<String, ExitCode> {
    let Some(dir) = compiler_dir() else {
        eprintln!("error[native.build.3]: can't find the native backend's `compiler/` directory; set AIL_COMPILER");
        return Err(ExitCode::from(2));
    };
    let exe = std::env::current_exe().map_err(|_| ExitCode::from(2))?;
    let mut cmd = Command::new(exe);
    cmd.arg("run").arg(&dir).args(args).env_remove("AIL_DERIVED");
    if let Some(p) = derived {
        cmd.env("AIL_DERIVED", p);
    }
    let out = cmd.output().map_err(|e| {
        eprintln!("error[native.build.3]: can't run the native backend: {e}");
        ExitCode::from(2)
    })?;
    let mut skipping = false;
    for line in String::from_utf8_lossy(&out.stderr).lines() {
        if line.contains(": warning[") {
            skipping = true;
        } else if !(line.starts_with(' ') || line.starts_with("help:") || line.is_empty()) {
            skipping = false;
        }
        if !skipping {
            eprintln!("{line}");
        }
    }
    if !out.status.success() {
        return Err(ExitCode::from(1));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// `ail api --mir FILE`: the program's MIR (ADR 0384).
pub fn mir(file: &str) -> ExitCode {
    match run_compiler(&["mir", file]) {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(code) => code,
    }
}

/// The target `ail build` makes by default: the host's.
fn host_target() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Some("x86_64-linux"),
        ("linux", "aarch64") => Some("aarch64-linux"),
        ("macos", "aarch64") => Some("aarch64-macos"),
        _ => None,
    }
}

/// `ail build FILE [-o OUT] [--target T]`: a native executable, by default
/// for the host and named after FILE in the current directory
/// (native.build.3).
pub fn command(args: &[&str]) -> ExitCode {
    let (mut file, mut out, mut target, mut no_cache) = (None, None, None, std::env::var_os("AIL_NO_CACHE").is_some());
    // run.limit.5: defaults for the run limits baked into the executable.
    let (mut timeout, mut memory) = (String::from("0"), String::from("0"));
    let mut i = 0;
    while i < args.len() {
        match (args[i], args.get(i + 1)) {
            ("-o", Some(o)) => (out, i) = (Some(*o), i + 1),
            ("--target", Some(t)) => (target, i) = (Some(*t), i + 1),
            ("--no-cache", _) => no_cache = true,
            ("--timeout", Some(v)) if positive(v) => (timeout, i) = (v.to_string(), i + 1),
            ("--max-memory", Some(v)) if positive(v) => (memory, i) = (v.to_string(), i + 1),
            (f, _) if file.is_none() && !f.starts_with('-') => file = Some(f),
            _ => {
                eprintln!("usage: ail build FILE [-o OUT] [--target x86_64-linux|aarch64-linux|aarch64-macos] [--no-cache] [--timeout MS] [--max-memory MB]");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }
    let Some(file) = file else {
        eprintln!("usage: ail build FILE [-o OUT] [--target x86_64-linux|aarch64-linux|aarch64-macos] [--no-cache] [--timeout MS] [--max-memory MB]");
        return ExitCode::from(2);
    };
    let Some(dir) = compiler_dir() else {
        eprintln!("error[native.build.3]: can't find the native backend's `compiler/` directory; set AIL_COMPILER");
        return ExitCode::from(2);
    };
    // The backend assumes a checked program.
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("ail"));
    // `--unsafe driver` (unsafe.hw.1) reaches the check this way.
    let mut check = Command::new(exe);
    check.arg("check");
    if crate::ast::UNSAFE_DRIVER.load(std::sync::atomic::Ordering::Relaxed) {
        check.args(["--unsafe", "driver"]);
    }
    match check.arg(file).status() {
        Ok(s) if s.success() => {}
        _ => return ExitCode::from(1),
    }
    let rt = dir.join("rt/native");
    // The program's subset problems are reported on every host; a host
    // without a target of its own gets the default target's.
    let target: Option<&str> = target.or_else(|| host_target());
    let out = match out {
        Some(o) => PathBuf::from(o),
        None => PathBuf::from(Path::new(file).file_stem().unwrap_or_default()),
    };
    // native.build.5: an unchanged program is copied from the cache.
    let key = if no_cache || target.is_none() { None } else { build_cache::key(file, &dir, &format!("{} {timeout} {memory}", target.unwrap_or_default())) };
    if let Some(bytes) = key.as_deref().and_then(build_cache::load) {
        return match write_executable(&out, &bytes) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error[native.build.3]: can't write {}: {e}", crate::shown::shown(&out));
                ExitCode::from(2)
            }
        };
    }
    let hex = match run_compiler(&["build", file, &rt.to_string_lossy(), target.unwrap_or("x86_64-linux"), &timeout, &memory, &dir.join("../bootstrap/src/std").to_string_lossy()]) {
        Ok(text) => text,
        Err(code) => return code,
    };
    if target.is_none() {
        eprintln!("error[native.build.3]: `ail build` has no target for this host yet; pass `--target`");
        return ExitCode::from(2);
    }
    let Some(bytes) = decode(&hex) else {
        eprintln!("error[native.build.3]: the native backend printed something other than hexadecimal");
        return ExitCode::from(2);
    };
    if let Some(key) = &key {
        build_cache::store(key, &bytes);
    }
    if let Err(e) = write_executable(&out, &bytes) {
        eprintln!("error[native.build.3]: can't write {}: {e}", crate::shown::shown(&out));
        return ExitCode::from(2);
    }
    ExitCode::SUCCESS
}

/// A positive whole number (run.limit.4).
fn positive(v: &str) -> bool {
    v.parse::<u64>().is_ok_and(|n| n > 0)
}

fn decode(hex: &str) -> Option<Vec<u8>> {
    let digits: Vec<u8> = hex.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if digits.len() % 2 != 0 {
        return None;
    }
    digits
        .chunks(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok())
        .collect()
}

fn write_executable(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    std::fs::write(path, bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}
