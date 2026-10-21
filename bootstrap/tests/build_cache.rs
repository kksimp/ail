//! The `ail build` cache (ADR 0655, native.build.5): a cached build, an
//! uncached build and a first build produce the same bytes; a changed
//! program is a new entry; `--no-cache` neither reads nor writes.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn entries(dir: &Path) -> usize {
    fs::read_dir(dir).map(|d| d.flatten().filter(|e| e.file_name().len() == 64).count()).unwrap_or(0)
}

fn build(cache: &Path, src: &Path, out: &Path, extra: &[&str]) {
    let status = Command::new(env!("CARGO_BIN_EXE_ail"))
        .env("AIL_CACHE_DIR", cache)
        .env_remove("AIL_NO_CACHE")
        .args(["build", &src.to_string_lossy(), "-o", &out.to_string_lossy()])
        .args(extra)
        .status()
        .unwrap();
    assert!(status.success(), "ail build failed");
}

// native.build.5
#[test]
fn cached_and_uncached_builds_are_identical() {
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("build_cache");
    let _ = fs::remove_dir_all(&tmp);
    let (cache, src) = (tmp.join("cache"), tmp.join("prog.ail"));
    fs::create_dir_all(&tmp).unwrap();
    fs::write(&src, "fn main()\n    print(\"one\")\n").unwrap();

    let (first, second, plain) = (tmp.join("first"), tmp.join("second"), tmp.join("plain"));
    build(&cache, &src, &first, &[]);
    assert_eq!(entries(&cache), 1, "a build is stored");
    build(&cache, &src, &second, &[]);
    assert_eq!(entries(&cache), 1, "an unchanged program is a hit");
    assert_eq!(fs::read(&first).unwrap(), fs::read(&second).unwrap(), "a hit returns the stored bytes");

    let (nocache, fresh) = (tmp.join("nocache_cache"), tmp.join("fresh"));
    build(&nocache, &src, &plain, &["--no-cache"]);
    assert_eq!(entries(&nocache), 0, "--no-cache stores nothing");
    assert_eq!(fs::read(&first).unwrap(), fs::read(&plain).unwrap(), "cached and uncached builds are the same bytes");

    // A changed program, and a changed file beside it, are new entries.
    fs::write(&src, "fn main()\n    print(\"two\")\n").unwrap();
    build(&cache, &src, &fresh, &[]);
    assert_eq!(entries(&cache), 2);
    assert_ne!(fs::read(&first).unwrap(), fs::read(&fresh).unwrap());
    fs::write(tmp.join("other.ail"), "fn unused() int\n    1\n").unwrap();
    build(&cache, &src, &fresh, &[]);
    assert_eq!(entries(&cache), 3, "any .ail file of the package is part of the key");
}
