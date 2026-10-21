//! Differential fuzzing of `ail build` output against `ail run` (fuzz.nat.*).
//! Each seed makes a program from the bytecode fuzzer's grammar (`fuzz/bcprogs.rs`,
//! with function values left out); programs the native backend declines are counted
//! as skips. The rest are built, run, and compared with the tree walker:
//!   fuzz.nat.1  standard output, the exit code and the first line of a trap match;
//!   fuzz.nat.2  the native program doesn't crash or hang.
//! A difference in the trap's column alone is counted, not failed (known: see
//! `tools/fuzz/found/native/`). `AIL_FUZZ_N` programs (default 24, the selftest
//! smoke) from `AIL_FUZZ_SEED` (default 1); `AIL_FUZZ_OUT` is where minimised
//! disagreements go. `tools/fuzz/native-wide.sh N [SEED]` runs it.

#[path = "fuzz/progs.rs"]
#[allow(dead_code)]
mod progs;
#[path = "fuzz/bcprogs.rs"]
mod bcprogs;
#[path = "fuzz/runner.rs"]
mod runner;

use runner::{exec_program_env, Out};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;

static TRAPPED: AtomicU64 = AtomicU64::new(0);
static COLUMN_ONLY: AtomicUsize = AtomicUsize::new(0);
static BUILT: AtomicUsize = AtomicUsize::new(0);
static DECLINED: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// What differs between the walker and the native build of `src`, if anything.
/// `None` also for a program the backend declines (counted in `DECLINED`).
fn disagreement(dir: &Path, src: &str, _pure: bool) -> Option<String> {
    let p = dir.join("p.ail");
    fs::write(&p, src).unwrap();
    let exe = dir.join("native");
    let _ = fs::remove_file(&exe);
    let ail = Path::new(env!("CARGO_BIN_EXE_ail"));
    let b = exec_program_env(ail, &["build", p.to_str().unwrap(), "-o", exe.to_str().unwrap()], &[], 60);
    if b.code != Some(0) || !exe.exists() {
        let why = b.stderr.lines().chain(b.stdout.lines()).find(|l| l.contains("error[")).unwrap_or("build failed");
        let rule = why.split('[').nth(1).and_then(|r| r.split(']').next()).unwrap_or("other");
        let msg = why.split("]: ").nth(1).unwrap_or(why).chars().take(70).collect::<String>();
        DECLINED.lock().unwrap().push(format!("{rule}: {msg}"));
        return None;
    }
    BUILT.fetch_add(1, Ordering::SeqCst);
    let walker = exec_program_env(ail, &["run", "--no-contracts", "--no-bytecode", p.to_str().unwrap()], &[], 30);
    let nat = exec_program_env(&exe, &[], &[], 30);
    if walker.code != Some(0) {
        TRAPPED.fetch_add(1, Ordering::SeqCst);
    }
    let first = |o: &Out| o.stderr.lines().next().unwrap_or("").to_string();
    if let Some(m) = nat.crashed() {
        return Some(format!("fuzz.nat.2 native: {m}"));
    }
    // Known (found/native/field_compound_assign_trap_column.ail): a column-only difference is
    // counted, not failed, so it doesn't hide other disagreements.
    let nocol = |s: String| -> String {
        let mut it = s.splitn(4, ':');
        match (it.next(), it.next(), it.next(), it.next()) {
            (Some(a), Some(l), Some(_), Some(rest)) => format!("{a}:{l}:{rest}"),
            _ => s,
        }
    };
    if nat.code == walker.code && nat.stdout == walker.stdout && first(&nat) != first(&walker) && nocol(first(&nat)) == nocol(first(&walker)) {
        COLUMN_ONLY.fetch_add(1, Ordering::SeqCst);
    } else if nat.code != walker.code || nat.stdout != walker.stdout || first(&nat) != first(&walker) {
        return Some(format!(
            "fuzz.nat.1 native differs\n--- walker (exit {:?})\n{}{}\n--- native (exit {:?})\n{}{}",
            walker.code, walker.stdout, first(&walker), nat.code, nat.stdout, first(&nat)
        ));
    }
    None
}

fn tag(msg: &str) -> String {
    msg.lines().next().unwrap_or("").split(' ').take(2).collect::<Vec<_>>().join(" ")
}

/// Drop lines while the same kind of disagreement remains.
fn minimise(dir: &Path, src: &str, pure: bool, want: &str) -> String {
    let mut lines: Vec<String> = src.lines().map(String::from).collect();
    let mut chunk = lines.len().div_ceil(2).max(1);
    let mut budget = 150;
    loop {
        let mut i = 0;
        while i < lines.len() && budget > 0 {
            let end = (i + chunk).min(lines.len());
            let mut trial = lines[..i].to_vec();
            trial.extend_from_slice(&lines[end..]);
            budget -= 1;
            let text = trial.join("\n") + "\n";
            if disagreement(dir, &text, pure).is_some_and(|m| tag(&m) == want) {
                lines = trial;
            } else {
                i += chunk;
            }
        }
        if chunk == 1 || budget == 0 {
            break;
        }
        chunk = chunk.div_ceil(2);
    }
    lines.join("\n") + "\n"
}

#[test]
fn fuzz_native_wide() {
    let n: u64 = std::env::var("AIL_FUZZ_N").ok().and_then(|v| v.parse().ok()).unwrap_or(24);
    let first: u64 = std::env::var("AIL_FUZZ_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let out_dir = std::env::var("AIL_FUZZ_OUT").map(PathBuf::from).unwrap_or_else(|_| root.join("tools/fuzz/found/native"));
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("fuzz-native-wide");
    let _ = fs::remove_dir_all(&scratch);
    let next = AtomicUsize::new(0);
    let ran = AtomicUsize::new(0);
    let rejected = AtomicUsize::new(0);
    let found: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let workers = std::thread::available_parallelism().map_or(2, |p| p.get()).clamp(1, 8);
    std::thread::scope(|s| {
        for w in 0..workers {
            let (next, ran, rejected, found, scratch, out_dir) = (&next, &ran, &rejected, &found, &scratch, &out_dir);
            s.spawn(move || {
                let dir = scratch.join(format!("w{w}"));
                fs::create_dir_all(&dir).unwrap();
                loop {
                    let k = next.fetch_add(1, Ordering::SeqCst) as u64;
                    if k >= n {
                        break;
                    }
                    let seed = first + k;
                    let pure = seed % 3 == 0;
                    let src = if (seed / 3) % 2 == 1 { bcprogs::wide_for(seed, pure, false) } else { bcprogs::program(seed, pure) };
                    let p = dir.join("p.ail");
                    fs::write(&p, &src).unwrap();
                    // A program the checker rejects tests nothing.
                    let chk = runner::exec_program_env(Path::new(env!("CARGO_BIN_EXE_ail")), &["check", p.to_str().unwrap()], &[], 30);
                    if chk.code != Some(0) {
                        rejected.fetch_add(1, Ordering::SeqCst);
                        if std::env::var_os("AIL_FUZZ_SHOW_REJECTED").is_some() {
                            eprintln!("rejected seed {seed}: {}", chk.stdout.lines().chain(chk.stderr.lines()).next().unwrap_or(""));
                        }
                        continue;
                    }
                    ran.fetch_add(1, Ordering::SeqCst);
                    let Some(msg) = disagreement(&dir, &src, pure) else {
                        continue;
                    };
                    let small = minimise(&dir, &src, pure, &tag(&msg));
                    let _ = fs::create_dir_all(out_dir);
                    let file = out_dir.join(format!("seed-{seed}.ail"));
                    fs::write(&file, format!("# {}\n{small}", msg.lines().next().unwrap_or(""))).unwrap();
                    let detail = disagreement(&dir, &small, pure).unwrap_or(msg);
                    found.lock().unwrap().push(format!("seed {seed}: {detail}\n  minimised: {}", file.display()));
                }
            });
        }
    });
    eprintln!(
        "fuzz-native-wide: {n} programs from seed {first}: {} passed the checker, {} built natively and compared ({} ended in a trap), {} rejected by the checker",
        ran.load(Ordering::SeqCst),
        BUILT.load(Ordering::SeqCst),
        TRAPPED.load(Ordering::SeqCst),
        rejected.load(Ordering::SeqCst)
    );
    eprintln!("fuzz-native-wide: {} runs differed only in the trap column", COLUMN_ONLY.load(Ordering::SeqCst));
    let mut hist = std::collections::BTreeMap::new();
    for r in DECLINED.lock().unwrap().iter() {
        *hist.entry(r.clone()).or_insert(0usize) += 1;
    }
    let mut v: Vec<_> = hist.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1));
    for (why, c) in v.iter().take(12) {
        eprintln!("fuzz-native-wide: declined {c}x: {why}");
    }
    let found = found.into_inner().unwrap();
    assert!(found.is_empty(), "{} disagreement(s):\n{}", found.len(), found.join("\n\n"));
}
