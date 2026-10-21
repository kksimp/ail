//! Fuzzing `ail fmt` with generated programs (fuzz.fmt.*). Each seed makes a program
//! from the bytecode fuzzer's grammar (`fuzz/bcprogs.rs`), messes up its layout
//! (trailing spaces, extra blank lines), and checks that
//!   fuzz.fmt.1  formatting twice gives the same text as formatting once;
//!   fuzz.fmt.2  the program means the same afterwards: `ail check` prints the same
//!               rules, and `ail run` gives the same stdout, exit code and trap rule,
//!               on the walker and on the bytecode tier;
//!   fuzz.fmt.3  the formatter doesn't crash and its output still checks.
//! `AIL_FUZZ_N` programs (default 24, the selftest smoke) from `AIL_FUZZ_SEED`;
//! `AIL_FUZZ_OUT` is where minimised disagreements go. `tools/fuzz/fmt-diff.sh N [SEED]`.

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
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

static CHANGED: AtomicUsize = AtomicUsize::new(0);
static COMPARED: AtomicUsize = AtomicUsize::new(0);
static RAN: AtomicUsize = AtomicUsize::new(0);

fn ail(args: &[&str]) -> Out {
    exec_program_env(Path::new(env!("CARGO_BIN_EXE_ail")), args, &[], 30)
}

/// Trailing spaces and extra blank lines: layout the formatter must undo.
fn mess(src: &str, seed: u64) -> String {
    let mut out = String::new();
    for (i, l) in src.lines().enumerate() {
        out.push_str(l);
        let h = (seed.wrapping_mul(6364136223846793005).wrapping_add(i as u64 * 1442695040888963407)) >> 33;
        if !l.trim().is_empty() && h % 5 == 0 {
            out.push_str("  ");
        }
        out.push('\n');
        if h % 7 == 0 {
            out.push_str("\n\n");
        }
    }
    out
}

/// The rules a program's diagnostics name, as sorted lines.
fn rules(o: &Out) -> String {
    // Only the diagnostic lines (`FILE:L:C: trap[rule]: ...`), not the source excerpts under them.
    let mut v: Vec<&str> = o
        .stderr
        .lines()
        .chain(o.stdout.lines())
        .filter_map(|l| {
            let at = ["trap[", "error[", "warning["].iter().filter_map(|k| l.find(k).map(|i| i + k.len())).min()?;
            let head = &l[..at];
            (head.contains(".ail:")).then(|| l[at..].split(']').next().unwrap_or(""))
        })
        .collect();
    v.sort();
    v.join(" ")
}

/// A run's stdout, exit code and trap rule (positions move when layout does).
fn meaning(p: &Path, tier: bool) -> (Option<i32>, String, String) {
    let f = p.to_string_lossy().into_owned();
    let mut a = vec!["run", "--no-contracts"];
    if !tier {
        a.push("--no-bytecode");
    }
    a.push(&f);
    let o = ail(&a);
    (o.code, o.stdout.clone(), rules(&o))
}

fn disagreement(dir: &Path, src: &str, seed: u64) -> Option<String> {
    let p = dir.join("p.ail");
    fs::write(&p, src).unwrap();
    let f = |q: &Path| ail(&["fmt", "--stdout", q.to_str().unwrap()]);
    let one = f(&p);
    if let Some(m) = one.crashed() {
        return Some(format!("fuzz.fmt.3 fmt: {m}"));
    }
    if one.code != Some(0) {
        return Some(format!("fuzz.fmt.3 fmt exit {:?}: {}", one.code, one.stderr.lines().next().unwrap_or("")));
    }
    let q = dir.join("q.ail");
    fs::write(&q, &one.stdout).unwrap();
    let two = f(&q);
    if two.stdout != one.stdout {
        return Some(format!("fuzz.fmt.1 not idempotent\n--- once\n{}--- twice\n{}", one.stdout, two.stdout));
    }
    let _ = seed;
    if one.stdout != src {
        CHANGED.fetch_add(1, Ordering::SeqCst);
    }
    COMPARED.fetch_add(1, Ordering::SeqCst);
    let (ca, cb) = (ail(&["check", p.to_str().unwrap()]), ail(&["check", q.to_str().unwrap()]));
    if ca.code != cb.code || rules(&ca) != rules(&cb) {
        return Some(format!("fuzz.fmt.2 check differs: {:?} {} against {:?} {}", ca.code, rules(&ca), cb.code, rules(&cb)));
    }
    if ca.code != Some(0) {
        return None;
    }
    RAN.fetch_add(1, Ordering::SeqCst);
    for tier in [false, true] {
        if meaning(&p, tier) != meaning(&q, tier) {
            return Some(format!("fuzz.fmt.2 `ail run{}` differs after formatting", if tier { "" } else { " --no-bytecode" }));
        }
    }
    None
}

fn tag(msg: &str) -> String {
    msg.lines().next().unwrap_or("").split(' ').take(2).collect::<Vec<_>>().join(" ")
}

/// Drop lines while the same kind of disagreement remains.
fn minimise(dir: &Path, src: &str, seed: u64, want: &str) -> String {
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
            if disagreement(dir, &text, seed).is_some_and(|m| tag(&m) == want) {
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
fn fuzz_fmt() {
    let n: u64 = std::env::var("AIL_FUZZ_N").ok().and_then(|v| v.parse().ok()).unwrap_or(24);
    let first: u64 = std::env::var("AIL_FUZZ_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let out_dir = std::env::var("AIL_FUZZ_OUT").map(PathBuf::from).unwrap_or_else(|_| root.join("tools/fuzz/found/fmt"));
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("fuzz-fmt");
    let _ = fs::remove_dir_all(&scratch);
    let next = AtomicUsize::new(0);
    let found: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let workers = std::thread::available_parallelism().map_or(2, |p| p.get()).clamp(1, 8);
    std::thread::scope(|s| {
        for w in 0..workers {
            let (next, found, scratch, out_dir) = (&next, &found, &scratch, &out_dir);
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
                    let base = if (seed / 3) % 2 == 1 { bcprogs::wide_for(seed, pure, true) } else { bcprogs::program(seed, pure) };
                    let src = mess(&base, seed);
                    let Some(msg) = disagreement(&dir, &src, seed) else {
                        continue;
                    };
                    let small = minimise(&dir, &src, seed, &tag(&msg));
                    let _ = fs::create_dir_all(out_dir);
                    let file = out_dir.join(format!("seed-{seed}.ail"));
                    fs::write(&file, format!("# {}\n{small}", msg.lines().next().unwrap_or(""))).unwrap();
                    let detail = disagreement(&dir, &small, seed).unwrap_or(msg);
                    found.lock().unwrap().push(format!("seed {seed}: {detail}\n  minimised: {}", file.display()));
                }
            });
        }
    });
    eprintln!("fuzz-fmt: {n} programs from seed {first}: {} compared, {} of them run before and after, {} changed by the formatter", COMPARED.load(Ordering::SeqCst), RAN.load(Ordering::SeqCst), CHANGED.load(Ordering::SeqCst));
    let found = found.into_inner().unwrap();
    assert!(found.is_empty(), "{} disagreement(s):\n{}", found.len(), found.join("\n\n"));
}
