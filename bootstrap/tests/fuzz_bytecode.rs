//! Differential fuzzing of the bytecode tier against the tree walker (fuzz.bc.*,
//! ADR 0756). Each seed makes a program that favours what the tier compiles
//! (`fuzz/bcprogs.rs`), runs it with the tier and with `--no-bytecode`, and
//! checks that
//!   fuzz.bc.1  standard output, the exit code and standard error (the trap
//!              text and its trace) are the same bytes;
//!   fuzz.bc.2  neither run panics, hangs or reports an internal error;
//!   fuzz.bc.3  in a `pure` program (no hand-off construct) the tier hands
//!              off at most `PURE_HANDOFFS` times however long the loops run.
//! `AIL_FUZZ_N` programs (default 60, the selftest smoke) from `AIL_FUZZ_SEED`
//! (default 1); `AIL_FUZZ_OUT` is where the minimised disagreements go
//! (default `tools/fuzz/found/bytecode`). `tools/fuzz/bytecode-diff.sh N` runs it.

#[path = "fuzz/bcprogs.rs"]
mod bcprogs;
#[path = "fuzz/progs.rs"]
#[allow(dead_code)]
mod progs;
#[path = "fuzz/runner.rs"]
mod runner;

use runner::{Out, exec_program_env};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// The most hand-offs a `pure` program makes: its prints and the call into `work`.
const PURE_HANDOFFS: u64 = 40;

/// Totals for the report: programs that trapped, tier instructions, hand-offs.
static TRAPPED: AtomicU64 = AtomicU64::new(0);
static INS: AtomicU64 = AtomicU64::new(0);
static HAND: AtomicU64 = AtomicU64::new(0);
static TRAPS: Mutex<Vec<String>> = Mutex::new(Vec::new());
/// fuzz.bc.3 misses: not failures, since a missed fast path costs speed only.
static MISSED: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn run(file: &Path, tier: bool) -> (Out, Option<u64>) {
    let mut args = vec!["run", "--no-contracts"];
    if !tier {
        args.push("--no-bytecode");
    }
    let f = file.to_string_lossy().into_owned();
    args.push(&f);
    let o = exec_program_env(Path::new(env!("CARGO_BIN_EXE_ail")), &args, &[("AIL_BYTECODE_STATS", "1")], 30);
    // The stats line goes to standard error; take it out before comparing.
    let mut handoffs = None;
    let kept: Vec<&str> = o
        .stderr
        .lines()
        .filter(|l| {
            if let Some(rest) = l.strip_prefix("bytecode: ") {
                handoffs = rest
                    .split(" hand-offs")
                    .next()
                    .and_then(|h| h.rsplit(", ").next())
                    .and_then(|n| n.trim().parse().ok());
                if let Some(ins) = rest
                    .split(" instructions")
                    .next()
                    .and_then(|h| h.rsplit("; ").next())
                    .and_then(|n| n.trim().parse::<u64>().ok())
                {
                    INS.fetch_add(ins, Ordering::SeqCst);
                }
                HAND.fetch_add(handoffs.unwrap_or(0), Ordering::SeqCst);
                false
            } else {
                true
            }
        })
        .collect();
    let stderr = kept.join("\n");
    (Out { code: o.code, stdout: o.stdout, stderr }, handoffs)
}

/// What differs between the tiers for `src`, if anything.
fn disagreement(dir: &Path, src: &str, pure: bool) -> Option<String> {
    let p = dir.join("p.ail");
    fs::write(&p, src).unwrap();
    let (walker, _) = run(&p, false);
    let (tier, handoffs) = run(&p, true);
    if tier.code != Some(0) {
        TRAPPED.fetch_add(1, Ordering::SeqCst);
        let rule = tier.stderr.split('[').nth(1).and_then(|r| r.split(']').next()).unwrap_or("none").to_string();
        TRAPS.lock().unwrap().push(rule);
    }
    for (name, o) in [("walker", &walker), ("tier", &tier)] {
        if let Some(m) = o.crashed() {
            return Some(format!("fuzz.bc.2 {name}: {m}"));
        }
    }
    if walker.stderr.contains("bootstrap.") {
        return Some("fuzz.bc.2 bootstrap.* diagnostic".into());
    }
    if walker.code != tier.code || walker.stdout != tier.stdout || walker.stderr != tier.stderr {
        return Some(format!(
            "fuzz.bc.1 the tiers differ\n--- walker (exit {:?})\n{}{}\n--- tier (exit {:?})\n{}{}",
            walker.code, walker.stdout, walker.stderr, tier.code, tier.stdout, tier.stderr
        ));
    }
    if pure && walker.code == Some(0) {
        if let Some(h) = handoffs {
            if h > PURE_HANDOFFS {
                MISSED.lock().unwrap().push(format!("{h}"));
                let _ = fs::write(dir.join("missed.ail"), src);
            }
        }
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
fn fuzz_bytecode_differential() {
    let n: u64 = std::env::var("AIL_FUZZ_N").ok().and_then(|v| v.parse().ok()).unwrap_or(60);
    let first: u64 = std::env::var("AIL_FUZZ_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let out_dir =
        std::env::var("AIL_FUZZ_OUT").map(PathBuf::from).unwrap_or_else(|_| root.join("tools/fuzz/found/bytecode"));
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("fuzz-bytecode");
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
                    let src =
                        if (seed / 3) % 2 == 1 { bcprogs::wide(seed, pure) } else { bcprogs::program(seed, pure) };
                    let p = dir.join("p.ail");
                    fs::write(&p, &src).unwrap();
                    // A program the checker rejects tests nothing.
                    let chk = runner::exec_program_env(
                        Path::new(env!("CARGO_BIN_EXE_ail")),
                        &["check", p.to_str().unwrap()],
                        &[],
                        30,
                    );
                    if chk.code != Some(0) {
                        rejected.fetch_add(1, Ordering::SeqCst);
                        if std::env::var_os("AIL_FUZZ_SHOW_REJECTED").is_some() {
                            eprintln!(
                                "rejected seed {seed}: {}",
                                chk.stdout.lines().chain(chk.stderr.lines()).next().unwrap_or("")
                            );
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
        "fuzz-bytecode: {} programs from seed {first}: {} compared ({} ended in a trap), {} rejected by the checker; the tier ran {} instructions with {} hand-offs",
        n,
        ran.load(Ordering::SeqCst),
        TRAPPED.load(Ordering::SeqCst),
        rejected.load(Ordering::SeqCst),
        INS.load(Ordering::SeqCst),
        HAND.load(Ordering::SeqCst)
    );
    let mut hist = std::collections::BTreeMap::new();
    for r in TRAPS.lock().unwrap().iter() {
        *hist.entry(r.clone()).or_insert(0usize) += 1;
    }
    eprintln!("fuzz-bytecode: traps by rule: {hist:?}");
    eprintln!(
        "fuzz-bytecode: fuzz.bc.3: {} pure programs made more than {PURE_HANDOFFS} hand-offs (a missed fast path, not a failure)",
        MISSED.lock().unwrap().len()
    );
    let found = found.into_inner().unwrap();
    assert!(found.is_empty(), "{} disagreement(s):\n{}", found.len(), found.join("\n\n"));
}
