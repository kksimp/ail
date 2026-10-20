//! `ail build FILE [-o OUT]` and `ail api --mir FILE`: drivers for the
//! native backend, which is written in AIL under `compiler/` (ADRs 0384,
//! 0385). This runs it on the interpreter, then writes the executable it
//! prints as hexadecimal (native.build.3); bytes go through text until the
//! standard library writes them.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// The repository's `compiler/` directory: `AIL_COMPILER`, or the one beside
/// the `bootstrap/target/PROFILE/ail` that is running.
fn compiler_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("AIL_COMPILER") {
        return Some(PathBuf::from(dir));
    }
    let exe = std::env::current_exe().ok()?;
    let dir = exe.ancestors().nth(4)?.join("compiler");
    dir.join("main.ail").exists().then_some(dir)
}

/// Run `ail run compiler ARGS`; stdout on success. The compiler's own
/// warnings are dropped; its errors are about the user's program.
fn run_compiler(args: &[&str]) -> Result<String, ExitCode> {
    let Some(dir) = compiler_dir() else {
        eprintln!("error[native.build.3]: can't find the native backend's `compiler/` directory; set AIL_COMPILER");
        return Err(ExitCode::from(2));
    };
    let exe = std::env::current_exe().map_err(|_| ExitCode::from(2))?;
    let out = Command::new(exe).arg("run").arg(&dir).args(args).output().map_err(|e| {
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
    let (mut file, mut out, mut target) = (None, None, None);
    let mut i = 0;
    while i < args.len() {
        match (args[i], args.get(i + 1)) {
            ("-o", Some(o)) => (out, i) = (Some(*o), i + 1),
            ("--target", Some(t)) => (target, i) = (Some(*t), i + 1),
            (f, _) if file.is_none() && !f.starts_with('-') => file = Some(f),
            _ => {
                eprintln!("usage: ail build FILE [-o OUT] [--target x86_64-linux|aarch64-linux|aarch64-macos]");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }
    let Some(file) = file else {
        eprintln!("usage: ail build FILE [-o OUT] [--target x86_64-linux|aarch64-linux|aarch64-macos]");
        return ExitCode::from(2);
    };
    let Some(dir) = compiler_dir() else {
        eprintln!("error[native.build.3]: can't find the native backend's `compiler/` directory; set AIL_COMPILER");
        return ExitCode::from(2);
    };
    // The backend assumes a checked program.
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("ail"));
    match Command::new(exe).args(["check", file]).status() {
        Ok(s) if s.success() => {}
        _ => return ExitCode::from(1),
    }
    let rt = dir.join("rt/native");
    // The program's subset problems are reported on every host; a host
    // without a target of its own gets the default target's.
    let target: Option<&str> = target.or_else(|| host_target());
    let hex = match run_compiler(&["build", file, &rt.to_string_lossy(), target.unwrap_or("x86_64-linux")]) {
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
    let out = match out {
        Some(o) => PathBuf::from(o),
        None => PathBuf::from(Path::new(file).file_stem().unwrap_or_default()),
    };
    if let Err(e) = write_executable(&out, &bytes) {
        eprintln!("error[native.build.3]: can't write {}: {e}", out.display());
        return ExitCode::from(2);
    }
    ExitCode::SUCCESS
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
