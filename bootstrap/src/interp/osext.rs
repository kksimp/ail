//! The parts of `std.fs`, `std.env` and `std.proc` that agents reach for
//! when operating a computer (ADRs 0281-0282): walking and matching
//! directories (std.fs.8-9), metadata (std.fs.10), renaming and copying
//! (std.fs.11-12), the environment as a map (std.env.5), and processes with
//! input and a time limit (std.proc.2), found on `PATH` (std.proc.3).
//! Each function here is plain Rust; `stdlib.rs` turns its result into
//! values. Failures are the short kinds of std.err.1.

use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::stdlib::{error_kind, is_sep_in, path_join};

/// A failure: the path it concerns, and its kind (std.err.1).
pub type FsFail = (String, &'static str);

/// std.fs.8: every file under `dir`, at any depth, as `dir` joined with its
/// path below it, sorted by bytes. Directories aren't listed and symbolic
/// links to directories aren't followed.
pub fn walk(dir: &str) -> Result<Vec<String>, FsFail> {
    let mut out = Vec::new();
    walk_into(dir, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk_into(dir: &str, out: &mut Vec<String>) -> Result<(), FsFail> {
    let entries = std::fs::read_dir(dir).map_err(|e| (dir.to_string(), error_kind(&e)))?;
    for entry in entries {
        let entry = entry.map_err(|e| (dir.to_string(), error_kind(&e)))?;
        let path = path_join(dir, &entry.file_name().to_string_lossy());
        match entry.file_type() {
            Ok(t) if t.is_dir() => walk_into(&path, out)?,
            Ok(_) => out.push(path),
            Err(e) => return Err((path, error_kind(&e))),
        }
    }
    Ok(())
}

/// std.fs.9: the paths that match `pattern`, sorted by bytes. In a
/// component, `*` matches any run of characters and `?` one character, and
/// neither matches a leading `.`; a whole component `**` matches any number
/// of directories. Directories that can't be read are skipped.
pub fn glob(pattern: &str) -> Vec<String> {
    glob_in_platform(pattern, cfg!(windows))
}

/// std.fs.9 on a platform the caller picks: the root of `pattern` (its
/// leading separator, or a Windows drive root `C:\`) and the components.
fn glob_in_platform(pattern: &str, windows: bool) -> Vec<String> {
    let (root, rest) = glob_root(pattern, windows);
    let parts: Vec<&str> = rest.split(|c| is_sep_in(c, windows)).filter(|p| !p.is_empty()).collect();
    let mut out = Vec::new();
    glob_in(root, &parts, &mut out);
    out.sort();
    out.dedup();
    out
}

/// std.fs.9: the absolute root of a pattern and what follows it. The root
/// keeps the pattern's own separator; a drive root is `C:` and its separator.
fn glob_root(pattern: &str, windows: bool) -> (&str, &str) {
    if windows && matches!(pattern.as_bytes(), [d, b':', s, ..] if d.is_ascii_alphabetic() && is_sep_in(*s as char, true)) {
        return (&pattern[..3], &pattern[3..]);
    }
    match pattern.chars().next() {
        Some(c) if is_sep_in(c, windows) => (&pattern[..1], &pattern[1..]),
        _ => ("", pattern),
    }
}

fn glob_in(prefix: &str, parts: &[&str], out: &mut Vec<String>) {
    let Some((&part, rest)) = parts.split_first() else { return };
    let join = |name: &str| if prefix.is_empty() { name.to_string() } else { path_join(prefix, name) };
    if !part.contains(['*', '?']) {
        // A literal component: no directory read needed.
        let path = join(part);
        if rest.is_empty() {
            if std::fs::symlink_metadata(&path).is_ok() {
                out.push(path);
            }
        } else if Path::new(&path).is_dir() {
            glob_in(&path, rest, out);
        }
        return;
    }
    let Ok(entries) = std::fs::read_dir(if prefix.is_empty() { "." } else { prefix }) else { return };
    let mut names: Vec<(String, bool)> = entries
        .flatten()
        .map(|e| (e.file_name().to_string_lossy().into_owned(), e.file_type().is_ok_and(|t| t.is_dir())))
        .collect();
    names.sort();
    if part == "**" {
        if rest.is_empty() {
            // A trailing `**`: everything below.
            for (name, is_dir) in names.iter().filter(|(n, _)| !n.starts_with('.')) {
                out.push(join(name));
                if *is_dir {
                    glob_in(&join(name), parts, out);
                }
            }
            return;
        }
        glob_in(prefix, rest, out);
        for (name, _) in names.iter().filter(|(n, d)| *d && !n.starts_with('.')) {
            glob_in(&join(name), parts, out);
        }
        return;
    }
    for (name, is_dir) in &names {
        if name.starts_with('.') && !part.starts_with('.') || !wildcard(part.as_bytes(), name.as_bytes()) {
            continue;
        }
        if rest.is_empty() {
            out.push(join(name));
        } else if *is_dir {
            glob_in(&join(name), rest, out);
        }
    }
}

/// Whether `name` matches the component pattern `pat` (`*` and `?`).
fn wildcard(pat: &[u8], name: &[u8]) -> bool {
    let (mut p, mut n) = (0, 0);
    let mut star: Option<(usize, usize)> = None;
    while n < name.len() {
        if p < pat.len() && (pat[p] == b'?' || pat[p] == name[n]) {
            // `?` takes one whole UTF-8 character.
            if pat[p] == b'?' {
                n += utf8_len(name[n]);
            } else {
                n += 1;
            }
            p += 1;
        } else if p < pat.len() && pat[p] == b'*' {
            star = Some((p, n));
            p += 1;
        } else if let Some((sp, sn)) = star {
            p = sp + 1;
            n = sn + utf8_len(name[sn]);
            star = Some((sp, n));
        } else {
            return false;
        }
    }
    pat[p..].iter().all(|&c| c == b'*')
}

fn utf8_len(b: u8) -> usize {
    match b {
        0xF0.. => 4,
        0xE0.. => 3,
        0xC0.. => 2,
        _ => 1,
    }
}

/// std.fs.10: size in bytes (0 for a directory), modification time in
/// milliseconds since 1970, and whether it is a directory.
pub fn metadata(path: &str) -> Result<(i64, i64, bool), &'static str> {
    let m = std::fs::metadata(path).map_err(|e| error_kind(&e))?;
    let modified = m
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX));
    let size = if m.is_dir() { 0 } else { i64::try_from(m.len()).unwrap_or(i64::MAX) };
    Ok((size, modified, m.is_dir()))
}

/// std.fs.11-12: rename or copy `from` to `to`. A failure names `from` when
/// it is what's missing (or, for `copy`, a directory), else `to`.
pub fn move_or_copy(from: &str, to: &str, copy: bool) -> Result<(), FsFail> {
    let source = std::fs::metadata(from);
    match &source {
        Err(e) => return Err((from.to_string(), error_kind(e))),
        // std.fs.12: copying a directory copies the tree.
        Ok(m) if copy && m.is_dir() => return copy_tree(from, to),
        Ok(_) => {}
    }
    let result = if copy { std::fs::copy(from, to).map(|_| ()) } else { std::fs::rename(from, to) };
    result.map_err(|e| {
        let kind = if Path::new(to).is_dir() && !source.as_ref().is_ok_and(|m| m.is_dir()) { "is_dir" } else { error_kind(&e) };
        (to.to_string(), kind)
    })
}

/// std.fs.12: copies the directory `from` and everything under it to `to`,
/// which is made (with its parents) when missing; files already there are
/// replaced. A symbolic link is copied as what it points to, a link to a
/// directory is not followed (kind `other`), and so is copying a directory
/// into itself.
fn copy_tree(from: &str, to: &str) -> Result<(), FsFail> {
    if let (Ok(src), Some(dst)) = (std::fs::canonicalize(from), nearest_canonical(Path::new(to)))
        && dst.starts_with(&src)
    {
        return Err((to.to_string(), "other"));
    }
    copy_dir(Path::new(from), Path::new(to))
}

/// The canonical form of `p`, or of its nearest existing ancestor.
fn nearest_canonical(p: &Path) -> Option<std::path::PathBuf> {
    let mut cur = p;
    loop {
        if let Ok(c) = std::fs::canonicalize(cur) {
            return Some(c);
        }
        cur = cur.parent().filter(|q| !q.as_os_str().is_empty())?;
    }
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), FsFail> {
    let fail = |p: &Path, e: &std::io::Error| (crate::shown::shown(&p), error_kind(e));
    std::fs::create_dir_all(to).map_err(|e| fail(to, &e))?;
    let mut entries: Vec<_> = std::fs::read_dir(from)
        .map_err(|e| fail(from, &e))?
        .collect::<Result<_, _>>()
        .map_err(|e| fail(from, &e))?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        let kind = entry.file_type().map_err(|e| fail(&src, &e))?;
        if kind.is_dir() {
            copy_dir(&src, &dst)?;
        } else if kind.is_symlink() && std::fs::metadata(&src).is_ok_and(|m| m.is_dir()) {
            return Err((crate::shown::shown(&src), "other"));
        } else {
            std::fs::copy(&src, &dst).map_err(|e| {
                let k = if dst.is_dir() { "is_dir" } else { error_kind(&e) };
                (crate::shown::shown(&dst), k)
            })?;
        }
    }
    Ok(())
}

/// std.env.5: the environment's UTF-8 variables, sorted by name.
pub fn vars() -> Vec<(String, String)> {
    let mut vars: Vec<(String, String)> =
        std::env::vars_os().filter_map(|(k, v)| Some((k.into_string().ok()?, v.into_string().ok()?))).collect();
    vars.sort();
    vars
}

/// What a finished process left: exit status, standard output, standard
/// error (std.proc.1).
pub type Ran = (std::process::ExitStatus, Vec<u8>, Vec<u8>);

/// std.proc.1-2: run `cmd` with `args`, writing `input` (if any) to its
/// standard input, and killing it after `timeout_ms` (if any), which is
/// then the failure `timed_out`.
pub fn run(cmd: &str, args: &[String], input: Option<&[u8]>, timeout_ms: Option<u64>) -> Result<Ran, &'static str> {
    // std.proc.1: on Windows a bare name is found with the extensions in
    // PATHEXT first, so `npm` starts `npm.cmd`, which Rust's own lookup
    // doesn't try. A name that isn't found is started as given and fails.
    let program = if cfg!(windows) && !cmd.contains(['/', '\\']) {
        which(cmd).unwrap_or_else(|| cmd.to_string())
    } else {
        cmd.to_string()
    };
    let mut child = Command::new(program)
        .args(args)
        .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| error_kind(&e))?;
    // Feed and drain on threads so a process that fills one pipe while we
    // wait on another can't deadlock.
    let feeder = input.map(|text| {
        let mut stdin = child.stdin.take().expect("piped");
        let text = text.to_vec();
        std::thread::spawn(move || {
            // A process that exits without reading all its input is not a failure.
            let _ = stdin.write_all(&text);
        })
    });
    let drain = |mut r: Box<dyn Read + Send>| {
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = r.read_to_end(&mut bytes);
            bytes
        })
    };
    let out = drain(Box::new(child.stdout.take().expect("piped")));
    let err = drain(Box::new(child.stderr.take().expect("piped")));
    let status = match timeout_ms {
        None => child.wait().map_err(|e| error_kind(&e))?,
        Some(ms) => {
            let deadline = Instant::now() + Duration::from_millis(ms);
            let mut nap = Duration::from_micros(200);
            loop {
                if let Some(status) = child.try_wait().map_err(|e| error_kind(&e))? {
                    break status;
                }
                let now = Instant::now();
                if now >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    // The readers end when the pipes close; a grandchild
                    // that keeps them open must not hold us, so they are
                    // left to finish on their own.
                    return Err("timed_out");
                }
                std::thread::sleep(nap.min(deadline - now));
                nap = (nap * 2).min(Duration::from_millis(10));
            }
        }
    };
    if let Some(f) = feeder {
        let _ = f.join();
    }
    Ok((status, out.join().unwrap_or_default(), err.join().unwrap_or_default()))
}

/// std.proc.3: the path of the program `run(name, ...)` would start, or
/// `None`. A name with a separator is checked as it is.
pub fn which(name: &str) -> Option<String> {
    if name.is_empty() {
        return None;
    }
    if name.contains('/') || (cfg!(windows) && name.contains('\\')) {
        return executable(Path::new(name)).then(|| name.to_string());
    }
    let path = std::env::var_os("PATH")?;
    let exts: Vec<String> = if cfg!(windows) {
        let pathext = std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.CMD;.BAT;.COM".into());
        std::iter::once(String::new()).chain(pathext.split(';').map(str::to_string)).collect()
    } else {
        vec![String::new()]
    };
    for dir in std::env::split_paths(&path) {
        for ext in &exts {
            let candidate = dir.join(format!("{name}{ext}"));
            if executable(&candidate) {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
    }
    None
}

fn executable(p: &Path) -> bool {
    let Ok(m) = std::fs::metadata(p) else { return false };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        m.is_file() && m.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        m.is_file()
    }
}

#[cfg(test)]
mod tests {
    use super::wildcard;

    #[test]
    fn glob_roots_on_both_platforms() {
        use super::glob_root;
        assert_eq!(glob_root("/tmp/*.ail", false), ("/", "tmp/*.ail"));
        assert_eq!(glob_root("a\\*.ail", false), ("", "a\\*.ail"));
        assert_eq!(glob_root("C:\\src\\*.ail", true), ("C:\\", "src\\*.ail"));
        assert_eq!(glob_root("\\src\\*.ail", true), ("\\", "src\\*.ail"));
        assert_eq!(glob_root("C:src", true), ("", "C:src"));
        assert_eq!(glob_root("C:\\src", false), ("", "C:\\src"));
    }

    #[test]
    fn wildcards() {
        assert!(wildcard(b"*.ail", b"hello.ail"));
        assert!(!wildcard(b"*.ail", b"hello.ail.out"));
        assert!(wildcard(b"h?llo*", b"hello.ail"));
        assert!(wildcard(b"*", b""));
        assert!(wildcard(b"a*b*c", b"axxbyyc"));
        assert!(!wildcard(b"a*b*c", b"axxbyy"));
        assert!(wildcard("?.txt".as_bytes(), "é.txt".as_bytes()));
    }
}
