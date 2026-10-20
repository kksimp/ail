//! Fuzz and differential harness (fuzz.*, ADR 0269-0271).
//!
//! Generates seeded, well-typed programs and checks, through the real `ail`
//! binary, that
//!   fuzz.prop.1  `check` is deterministic and never panics;
//!   fuzz.prop.2  `fmt` is idempotent and keeps the program's outline and output;
//!   fuzz.prop.3  `run` never panics and repeats under `--deterministic`;
//!   fuzz.prop.4  `api` (every form) is valid JSON, and a clean program raises
//!                no `bootstrap.*` diagnostic;
//!   fuzz.prop.5  `run` prints the same bytes and exits the same with and
//!                without the bytecode tier (ADR 0361);
//!   fuzz.mut.1   after one token-level mutation, `check` still diagnoses
//!                without panicking and `fix` leaves a parsable file.
//!
//! `AIL_FUZZ_N` programs (default 200, the smoke subset that `./ail selftest`
//! runs) starting at `AIL_FUZZ_SEED` (default 1). `tools/fuzz/run.sh` runs
//! 10,000. A failure is minimised (by lines) and saved under
//! `$AIL_FUZZ_OUT` (default: the test scratch directory) as `seed-N.ail`.

#[path = "fuzz/progs.rs"]
mod progs;
#[path = "fuzz/json.rs"]
mod json;

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Probes that checked clean and ran (the rest were rejected as ill-typed).
/// The default 200-program run that `./ail selftest` does skips the slower
/// repeats (second runs, the `api` variants, stacked mutants); `AIL_FUZZ_N` selects the full set.
fn smoke() -> bool {
    std::env::var("AIL_FUZZ_N").is_err()
}

static PROBES_RUN: AtomicUsize = AtomicUsize::new(0);

struct Out {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Out {
    /// fuzz.prop.1: a Rust panic, an interpreter-internal error, a signal or a timeout.
    fn crashed(&self) -> Option<String> {
        let both = format!("{}{}", self.stdout, self.stderr);
        if self.code.is_none() {
            Some("killed by a signal or timeout".into())
        } else if self.code == Some(101) || both.contains("panicked at") || both.contains("internal error") {
            let line = both.lines().find(|l| l.contains("panicked at") || l.contains("internal error")).unwrap_or("");
            Some(format!("panic: {}", line.rsplit(": ").next().unwrap_or(line)))
        } else {
            None
        }
    }
}

/// One scratch directory per worker, so files never collide.
struct Runner {
    dir: PathBuf,
}

impl Runner {
    fn exec(&self, args: &[&str], file: &Path) -> Out {
        self.exec_for(args, file, 10)
    }

    fn exec_for(&self, args: &[&str], file: &Path, secs: u64) -> Out {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ail"))
            .args(args)
            .arg(file)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawning ail");
        let (mut so, mut se) = (child.stdout.take().unwrap(), child.stderr.take().unwrap());
        let t_out = std::thread::spawn(move || {
            let mut b = Vec::new();
            let _ = so.read_to_end(&mut b);
            b
        });
        let t_err = std::thread::spawn(move || {
            let mut b = Vec::new();
            let _ = se.read_to_end(&mut b);
            b
        });
        let deadline = Instant::now() + Duration::from_secs(secs);
        let code = loop {
            match child.try_wait().unwrap() {
                Some(status) => break status.code(),
                None if Instant::now() > deadline => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break None;
                }
                None => std::thread::sleep(Duration::from_millis(1)),
            }
        };
        let text = |b: Vec<u8>| String::from_utf8_lossy(&b).into_owned();
        Out { code, stdout: text(t_out.join().unwrap()), stderr: text(t_err.join().unwrap()) }
    }

    fn write(&self, name: &str, src: &str) -> PathBuf {
        let p = self.dir.join(name);
        fs::write(&p, src).unwrap();
        p
    }
}

/// Is the first of two identical runs equal to the second?
fn same(a: &Out, b: &Out) -> bool {
    a.code == b.code && a.stdout == b.stdout && a.stderr == b.stderr
}

/// Properties of a generated program; `Err` names the property that broke.
/// `Ok(true)` when the program checked clean, `Ok(false)` when it was rejected
/// (a generator or checker bug, counted separately).
fn generated(r: &Runner, src: &str) -> Result<bool, String> {
    let p = r.write("p.ail", src);
    let c1 = r.exec(&["check", "--json", "--deterministic"], &p);
    if let Some(m) = c1.crashed() {
        return Err(format!("fuzz.prop.1 check: {m}"));
    }
    if !smoke() && !same(&c1, &r.exec(&["check", "--json", "--deterministic"], &p)) {
        return Err("fuzz.prop.1 check is not deterministic".into());
    }
    for line in c1.stdout.lines().chain(c1.stderr.lines()) {
        json::normalise(line).map_err(|e| format!("fuzz.prop.4 diagnostic is not JSON: {e}"))?;
    }
    if c1.stdout.contains("\"rule\":\"bootstrap.") || c1.stderr.contains("\"rule\":\"bootstrap.") {
        return Err("fuzz.prop.4 bootstrap.* diagnostic on a generated program".into());
    }
    let clean = c1.code == Some(0);

    if !clean {
        return Ok(false);
    }
    // fuzz.prop.2: fmt.
    let f1 = r.exec(&["fmt", "--stdout"], &p);
    if let Some(m) = f1.crashed() {
        return Err(format!("fuzz.prop.2 fmt: {m}"));
    }
    if f1.code != Some(0) {
        return Err(format!("fuzz.prop.2 fmt rejected a program: {}", f1.stderr.lines().next().unwrap_or("")));
    }
    let q = r.write("q.ail", &f1.stdout);
    let f2 = r.exec(&["fmt", "--stdout"], &q);
    if f2.stdout != f1.stdout {
        return Err("fuzz.prop.2 fmt is not idempotent".into());
    }
    if clean {
        let (a1, a2) = (r.exec(&["api"], &p), r.exec(&["api"], &q));
        for (out, form) in [(&a1, "api")] {
            if let Some(m) = out.crashed() {
                return Err(format!("fuzz.prop.4 {form}: {m}"));
            }
        }
        let (n1, n2) = (
            json::normalise(&a1.stdout).map_err(|e| format!("fuzz.prop.4 api is not JSON: {e}"))?,
            json::normalise(&a2.stdout).map_err(|e| format!("fuzz.prop.4 api of the formatted program is not JSON: {e}"))?,
        );
        if n1 != n2 {
            return Err("fuzz.prop.2 fmt changed the program's outline".into());
        }
        let variants: &[&[&str]] = if smoke() { &[&["api", "--symbols"]] } else { &[&["api", "--pretty"], &["api", "--text"], &["api", "--symbols"]] };
        for extra in variants.iter().copied() {
            let o = r.exec(extra, &p);
            if let Some(m) = o.crashed() {
                return Err(format!("fuzz.prop.4 {}: {m}", extra.join(" ")));
            }
            if extra.last() != Some(&"--text") && o.code == Some(0) {
                json::normalise(&o.stdout).map_err(|e| format!("fuzz.prop.4 {} is not JSON: {e}", extra.join(" ")))?;
            }
        }

        // fuzz.prop.3: run.
        let r1 = r.exec(&["run", "--deterministic"], &p);
        if let Some(m) = r1.crashed() {
            return Err(format!("fuzz.prop.3 run: {m}"));
        }
        if r1.stderr.contains("bootstrap.") {
            return Err("fuzz.prop.4 bootstrap.* error while running a clean program".into());
        }
        // The smoke run compares against the no-contracts run, which doubles as the repeat.
        // contract.run.3: the generated contracts always hold, so skipping them changes nothing.
        let r4 = r.exec(&["run", "--deterministic", "--no-contracts"], &p);
        if r4.code != r1.code || r4.stdout != r1.stdout {
            return Err("fuzz.prop.3 --no-contracts changed what the program prints".into());
        }
        // fuzz.prop.5: the bytecode tier against the walker. `--deterministic` and checked contracts
        // keep a run on the walker, so the tier's run denies the same capabilities instead.
        let r5 = r.exec(&["run", "--deny", "clock,random,proc,net", "--no-contracts"], &p);
        if !same(&r4, &r5) {
            return Err(format!(
                "fuzz.prop.5 the bytecode tier changed the run\n--- walker (exit {:?})\n{}{}--- tier (exit {:?})\n{}{}",
                r4.code, r4.stdout, r4.stderr, r5.code, r5.stdout, r5.stderr
            ));
        }
        if !smoke() && !same(&r1, &r.exec(&["run", "--deterministic"], &p)) {
            return Err("fuzz.prop.3 run is not deterministic".into());
        }
        let r3 = r.exec(&["run", "--deterministic"], &q);
        if r3.code != r1.code || r3.stdout != r1.stdout {
            return Err("fuzz.prop.2 fmt changed what the program prints".into());
        }
    }
    Ok(clean)
}

/// fuzz.prop.3 on a probe: it may be rejected or trap, but not crash, and it repeats.
fn probed(r: &Runner, src: &str) -> Result<(), String> {
    let p = r.write("r.ail", src);
    let c = r.exec(&["check", "--json"], &p);
    if let Some(m) = c.crashed() {
        return Err(format!("fuzz.prop.1 check of a probe: {m}"));
    }
    if c.code != Some(0) {
        return Ok(());
    }
    PROBES_RUN.fetch_add(1, Ordering::SeqCst);
    let r1 = r.exec_for(&["run", "--deterministic"], &p, 5);
    if let Some(m) = r1.crashed() {
        return Err(format!("fuzz.prop.3 run of a probe: {m}"));
    }
    if r1.stderr.contains("bootstrap.") {
        return Err("fuzz.prop.4 bootstrap.* error while running a probe".into());
    }
    if !smoke() && !same(&r1, &r.exec_for(&["run", "--deterministic"], &p, 5)) {
        return Err("fuzz.prop.3 run of a probe is not deterministic".into());
    }
    Ok(())
}

/// fuzz.mut.1 on a mutated program.
fn mutated(r: &Runner, src: &str) -> Result<(), String> {
    let p = r.write("m.ail", src);
    let c1 = r.exec(&["check", "--json"], &p);
    if let Some(m) = c1.crashed() {
        return Err(format!("fuzz.mut.1 check: {m}"));
    }
    if !smoke() && !same(&c1, &r.exec(&["check", "--json"], &p)) {
        return Err("fuzz.mut.1 check is not deterministic".into());
    }
    for line in c1.stdout.lines().chain(c1.stderr.lines()) {
        json::normalise(line).map_err(|e| format!("fuzz.mut.1 diagnostic is not JSON: {e}"))?;
    }
    if c1.code == Some(2) && !c1.stdout.contains("\"severity\":\"error\"") && !c1.stderr.contains("\"severity\":\"error\"") {
        return Err("fuzz.mut.1 rejected with no error diagnostic".into());
    }
    if c1.code == Some(0) {
        // A mutant that still checks clean must run without a panic; a hang
        // (a changed loop bound) is not a finding.
        let run = r.exec_for(&["run", "--deterministic"], &p, 3);
        if run.code.is_some() {
            if let Some(m) = run.crashed() {
                return Err(format!("fuzz.mut.1 run: {m}"));
            }
        }
    }
    let parsed_before = r.exec(&["fmt", "--stdout"], &p);
    if let Some(m) = parsed_before.crashed() {
        return Err(format!("fuzz.mut.1 fmt: {m}"));
    }
    let fixed = r.write("x.ail", src);
    let fx = r.exec(&["fix"], &fixed);
    if let Some(m) = fx.crashed() {
        return Err(format!("fuzz.mut.1 fix: {m}"));
    }
    if parsed_before.code == Some(0) {
        let after = r.exec(&["fmt", "--stdout"], &fixed);
        if after.code != Some(0) {
            return Err("fuzz.mut.1 fix output no longer parses".into());
        }
    }
    Ok(())
}

/// The failing property's name without its detail, for minimising.
fn kind(msg: &str) -> &str {
    // The detail after the property name matters: a hang is not a panic.
    msg
}

/// Remove chunks of lines while `fails` still reports the same kind.
fn minimise(src: &str, want: &str, fails: &dyn Fn(&str) -> Option<String>) -> String {
    let mut lines: Vec<&str> = src.lines().collect();
    let mut chunk = lines.len().div_ceil(2).max(1);
    let mut budget = 150;
    loop {
        let mut i = 0;
        while i < lines.len() && budget > 0 {
            let end = (i + chunk).min(lines.len());
            let mut trial: Vec<&str> = lines[..i].to_vec();
            trial.extend_from_slice(&lines[end..]);
            let text = trial.join("\n") + "\n";
            budget -= 1;
            if fails(&text).is_some_and(|m| kind(&m) == want) {
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

fn env_num(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

#[test]
fn fuzz_generated_and_mutated_programs() {
    let n = env_num("AIL_FUZZ_N", 200) as usize;
    let first = env_num("AIL_FUZZ_SEED", 1);
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("fuzz");
    let out_dir = std::env::var("AIL_FUZZ_OUT").map(PathBuf::from).unwrap_or_else(|_| root.join("failures"));
    let workers = std::thread::available_parallelism().map_or(2, |p| p.get()).clamp(1, 8);
    let next = AtomicUsize::new(0);
    let failures: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let (rejected, clean) = (AtomicUsize::new(0), AtomicUsize::new(0));
    let rejects: Mutex<Vec<u64>> = Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for w in 0..workers {
            let (next, failures, rejected, clean, rejects, root, out_dir) = (&next, &failures, &rejected, &clean, &rejects, &root, &out_dir);
            s.spawn(move || {
                let dir = root.join(format!("w{w}"));
                fs::create_dir_all(&dir).unwrap();
                fs::create_dir_all(out_dir).unwrap();
                let r = Runner { dir };
                loop {
                    let i = next.fetch_add(1, Ordering::SeqCst);
                    if i >= n {
                        break;
                    }
                    let seed = first + i as u64;
                    let src = progs::program(seed);
                    let record = |tag: &str, msg: String, text: &str, f: &dyn Fn(&str) -> Option<String>| {
                        let small = minimise(text, kind(&msg), f);
                        let file = out_dir.join(format!("{tag}-{seed}.ail"));
                        fs::write(&file, &small).unwrap();
                        failures.lock().unwrap().push(format!("seed {seed} ({tag}): {msg}\n  minimised to {} lines: {}", small.lines().count(), file.display()));
                    };
                    match generated(&r, &src) {
                        Ok(true) => {
                            clean.fetch_add(1, Ordering::SeqCst);
                        }
                        Ok(false) => {
                            rejected.fetch_add(1, Ordering::SeqCst);
                            rejects.lock().unwrap().push(seed);
                        }
                        Err(msg) => record("gen", msg, &src, &|t| generated(&r, t).err()),
                    }
                    let pr = progs::probe(seed);
                    if let Err(msg) = probed(&r, &pr) {
                        record("probe", msg, &pr, &|t| probed(&r, t).err());
                    }
                    // Up to three edits stacked, so the damage is not always one token.
                    let mut m = src.clone();
                    for k in 0..if smoke() { 1 } else { 3 } {
                        let (next, what) = progs::mutate(&m, seed.wrapping_mul(31).wrapping_add(k));
                        m = next;
                        if let Err(msg) = mutated(&r, &m) {
                            record("mut", format!("{msg} (mutation {k}: {what})"), &m, &|t| mutated(&r, t).err());
                            break;
                        }
                    }
                }
            });
        }
    });
    let failures = failures.into_inner().unwrap();
    let (rejected, clean) = (rejected.into_inner(), clean.into_inner());
    let mut rejects = rejects.into_inner().unwrap();
    rejects.sort();
    eprintln!(
        "fuzz: {n} programs from seed {first}: {clean} clean, {rejected} rejected {:?}, {} probes run, {} failures",
        &rejects[..rejects.len().min(10)],
        PROBES_RUN.load(Ordering::SeqCst),
        failures.len()
    );
    assert!(failures.is_empty(), "{} fuzz failures (fuzz.policy.1: each becomes a case)\n{}", failures.len(), failures.join("\n"));
    // The generator is well-typed by construction (fuzz.gen.1): a rejection is a bug in it or the checker.
    assert!(rejected * 50 <= n, "{rejected} of {n} generated programs were rejected, e.g. seed {:?}", rejects.first());
}

/// `AIL_FUZZ_DUMP=SEED cargo test --test fuzz dump -- --nocapture` prints a program.
#[test]
fn dump() {
    if let Ok(seed) = std::env::var("AIL_FUZZ_DUMP") {
        let seed: u64 = seed.parse().unwrap();
        let mut src = progs::program(seed);
        // AIL_FUZZ_MUT=K also applies the first K edits the fuzz test would.
        for k in 0..env_num("AIL_FUZZ_MUT", 0) {
            src = progs::mutate(&src, seed.wrapping_mul(31).wrapping_add(k)).0;
        }
        print!("{src}");
    }
}
