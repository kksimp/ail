//! The `ail build` cache (ADR 0655): a finished executable stored under a
//! hash of everything that decides its bytes, so building an unchanged
//! program again copies it instead of running the native backend. The key
//! covers the program's package, the backend's sources, the bootstrap
//! binary, the target and the path as given; a build the backend rejects is
//! never stored. `--no-cache` and `AIL_NO_CACHE` skip it (native.build.5).

use crate::net::crypto::sha2::{Hash, Sha256};
use std::path::{Path, PathBuf};

/// Entries kept; the least recently used are deleted past this.
const KEEP: usize = 64;
/// A program package larger than this is not cached (the key would cost more
/// than it saves).
const MAX_FILES: usize = 4000;
const MAX_BYTES: u64 = 32 << 20;

/// The cache directory: `AIL_CACHE_DIR`, `$XDG_CACHE_HOME/ail/build`, or
/// `%LOCALAPPDATA%\\ail\\build` (Windows) or `~/.cache/ail/build`.
fn dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("AIL_CACHE_DIR") {
        return Some(PathBuf::from(d));
    }
    if let Some(d) = std::env::var_os("XDG_CACHE_HOME").filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(d).join("ail/build"));
    }
    if let Some(d) = std::env::var_os("LOCALAPPDATA").filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(d).join("ail/build"));
    }
    std::env::var_os("HOME").filter(|d| !d.is_empty()).map(|h| PathBuf::from(h).join(".cache/ail/build"))
}

/// Every `.ail` file under `root`, sorted by path; `None` past the limits.
/// Symbolic links and hidden directories are not followed.
fn sources(root: &Path, files: &mut Vec<PathBuf>, bytes: &mut u64) -> Option<()> {
    let mut entries: Vec<_> = std::fs::read_dir(root).ok()?.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let kind = e.file_type().ok()?;
        let path = e.path();
        if kind.is_dir() && !e.file_name().to_string_lossy().starts_with('.') {
            sources(&path, files, bytes)?;
        } else if kind.is_file() && path.extension().is_some_and(|x| x == "ail") {
            *bytes += e.metadata().ok()?.len();
            files.push(path);
            if files.len() > MAX_FILES || *bytes > MAX_BYTES {
                return None;
            }
        }
    }
    Some(())
}

fn add(h: &mut Sha256, tag: &str, data: &[u8]) {
    h.update(&(tag.len() as u64).to_le_bytes());
    h.update(tag.as_bytes());
    h.update(&(data.len() as u64).to_le_bytes());
    h.update(data);
}

/// The key of building `file` for `target` with the backend in `compiler`,
/// or `None` when the build can't be cached.
pub fn key(file: &str, compiler: &Path, target: &str) -> Option<String> {
    let mut h = Sha256::new();
    add(&mut h, "ail-build-cache-1", env!("CARGO_PKG_VERSION").as_bytes());
    let exe = std::fs::metadata(std::env::current_exe().ok()?).ok()?;
    let mtime = exe.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?;
    add(&mut h, "exe", format!("{} {}", exe.len(), mtime.as_nanos()).as_bytes());
    add(&mut h, "target", target.as_bytes());
    add(&mut h, "file", file.as_bytes());
    let (mut paths, mut bytes) = (Vec::new(), 0);
    sources(compiler, &mut paths, &mut bytes)?;
    let split = paths.len();
    let root = Path::new(file).parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    sources(root, &mut paths, &mut bytes)?;
    for (i, p) in paths.iter().enumerate() {
        let (base, tag) = if i < split { (compiler, "backend") } else { (root, "program") };
        let rel = p.strip_prefix(base).unwrap_or(p).to_string_lossy();
        add(&mut h, tag, rel.as_bytes());
        add(&mut h, "text", &std::fs::read(p).ok()?);
    }
    // The standard modules written in AIL that only `ail build` reads (ADR 0819)
    // aren't in the executable, so their text is part of the key.
    let std_dir = compiler.join("../bootstrap/src/std");
    let (mut std_paths, mut std_bytes) = (Vec::new(), 0);
    if sources(&std_dir, &mut std_paths, &mut std_bytes).is_some() {
        for p in &std_paths {
            add(&mut h, "std", p.strip_prefix(&std_dir).unwrap_or(p).to_string_lossy().as_bytes());
            add(&mut h, "text", &std::fs::read(p).ok()?);
        }
    }
    // The program file itself, in case the walk above skipped it (a hidden directory).
    add(&mut h, "root", &std::fs::read(file).ok()?);
    Some(h.finish().iter().map(|b| format!("{b:02x}")).collect())
}

/// The cached executable for `key`, marking it recently used.
pub fn load(key: &str) -> Option<Vec<u8>> {
    let path = dir()?.join(key);
    let bytes = std::fs::read(&path).ok()?;
    if let Ok(f) = std::fs::File::options().append(true).open(&path) {
        let _ = f.set_modified(std::time::SystemTime::now());
    }
    Some(bytes)
}

/// Store `bytes` under `key` (best effort), then delete the oldest entries
/// past `KEEP`.
pub fn store(key: &str, bytes: &[u8]) {
    let Some(dir) = dir() else { return };
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let tmp = dir.join(format!("{key}.{}.tmp", std::process::id()));
    if std::fs::write(&tmp, bytes).is_err() || std::fs::rename(&tmp, dir.join(key)).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return;
    }
    let Ok(read) = std::fs::read_dir(&dir) else { return };
    let mut entries: Vec<_> = read
        .flatten()
        .filter(|e| e.file_name().len() == 64)
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
        .collect();
    if entries.len() > KEEP {
        entries.sort();
        for (_, p) in &entries[..entries.len() - KEEP] {
            let _ = std::fs::remove_file(p);
        }
    }
}
