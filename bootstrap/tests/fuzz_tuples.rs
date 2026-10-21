//! Fuzzing tuples (type.tuple.*, fuzz.tup.*) with generated programs (`fuzz/bcprogs.rs`:
//! `tuples`): literals, destructuring, `t.0`, tuple patterns, tuple returns, tuples in
//! lists and as map keys. Some programs carry one deliberately invalid construct.
//!   fuzz.tup.1  the walker and the bytecode tier give the same stdout, exit code and stderr;
//!   fuzz.tup.2  `ail check` and both runs name the same rules (a rejected program is
//!               rejected by `ail run` on both tiers, with the same rules);
//!   fuzz.tup.3  nothing panics or hangs, and a program that checks never fails with
//!               a diagnostic other than a trap.
//! `AIL_FUZZ_N` programs (default 24) from `AIL_FUZZ_SEED`; `AIL_FUZZ_OUT` is where
//! minimised disagreements go. `tools/fuzz/tuple-diff.sh N [SEED]` runs it.

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

static REJECTED: AtomicUsize = AtomicUsize::new(0);
static TRAPPED: AtomicUsize = AtomicUsize::new(0);
static COMPARED: AtomicUsize = AtomicUsize::new(0);

fn ail(args: &[&str]) -> Out {
    exec_program_env(Path::new(env!("CARGO_BIN_EXE_ail")), args, &[], 30)
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

fn run(p: &Path, tier: bool) -> Out {
    let f = p.to_string_lossy().into_owned();
    let mut a = vec!["run", "--no-contracts"];
    if !tier {
        a.push("--no-bytecode");
    }
    a.push(&f);
    ail(&a)
}

fn disagreement(dir: &Path, src: &str, _seed: u64) -> Option<String> {
    let p = dir.join("p.ail");
    fs::write(&p, src).unwrap();
    let check = ail(&["check", p.to_str().unwrap()]);
    let (walker, tier) = (run(&p, false), run(&p, true));
    COMPARED.fetch_add(1, Ordering::SeqCst);
    for (name, o) in [("check", &check), ("walker", &walker), ("tier", &tier)] {
        if let Some(m) = o.crashed() {
            return Some(format!("fuzz.tup.3 {name}: {m}"));
        }
    }
    if walker.code != tier.code || walker.stdout != tier.stdout || walker.stderr != tier.stderr {
        return Some(format!("fuzz.tup.1 the tiers differ\n--- walker (exit {:?})\n{}{}\n--- tier (exit {:?})\n{}{}", walker.code, walker.stdout, walker.stderr, tier.code, tier.stdout, tier.stderr));
    }
    let accepted = check.code == Some(0);
    if !accepted {
        REJECTED.fetch_add(1, Ordering::SeqCst);
        if walker.code == Some(0) {
            return Some("fuzz.tup.2 ail check rejects a program ail run accepts".into());
        }
        if rules(&check) != rules(&walker) {
            return Some(format!("fuzz.tup.2 rules differ: check [{}] against run [{}]", rules(&check), rules(&walker)));
        }
    } else if rules(&walker).split(' ').any(|r| !r.is_empty() && !walker.stderr.contains(&format!("trap[{r}]"))) {
        return Some(format!("fuzz.tup.3 a checked program fails with a diagnostic: [{}]", rules(&walker)));
    }
    if walker.code != Some(0) && accepted {
        TRAPPED.fetch_add(1, Ordering::SeqCst);
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
fn fuzz_tuples() {
    let n: u64 = std::env::var("AIL_FUZZ_N").ok().and_then(|v| v.parse().ok()).unwrap_or(24);
    let first: u64 = std::env::var("AIL_FUZZ_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let out_dir = std::env::var("AIL_FUZZ_OUT").map(PathBuf::from).unwrap_or_else(|_| root.join("tools/fuzz/found/tuples"));
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("fuzz-tuples");
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
                    let src = bcprogs::tuples(seed, seed % 4 == 0);
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
    eprintln!("fuzz-tuples: {n} programs from seed {first}: {} compared, {} rejected by the checker, {} ended in a trap", COMPARED.load(Ordering::SeqCst), REJECTED.load(Ordering::SeqCst), TRAPPED.load(Ordering::SeqCst));
    let found = found.into_inner().unwrap();
    assert!(found.is_empty(), "{} disagreement(s):\n{}", found.len(), found.join("\n\n"));
}
