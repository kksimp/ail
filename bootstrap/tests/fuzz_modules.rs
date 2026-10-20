//! Module fuzzers (fuzz.mod.*, ADR 0460): adversarial inputs for the standard
//! modules, through the real `ail` binary.
//!
//! Each template `fuzz/modules/NAME.ail` is a small program with `{{TEXT}}`,
//! `{{TEXT2}}` (string literals) and `{{INT}}` placeholders. A header names
//! its input kind and the module it needs:
//!
//!     # fuzz-input: json|text|bytes|regex|date|http|compress
//!     # fuzz-needs: std.bytes        (blank when built in)
//!
//! A template whose module isn't merged yet is skipped and reported. A lane
//! adds a module's fuzzer by adding one file, with no Rust. The properties:
//!   fuzz.mod.1  no crash: no panic, internal error or signal;
//!   fuzz.mod.2  no hang: a run ends in 10 s, so a regex engine must be linear;
//!   fuzz.mod.3  the program prints no line that starts with `FAIL` (its own oracle);
//!   fuzz.mod.4  a trap names a spec rule, never `bootstrap.*`;
//!   fuzz.mod.5  the bytecode tier prints the same bytes and exits the same as the walker;
//!   fuzz.mod.6  `ail build` output, when the command exists, does too (ADR 0460);
//!   fuzz.mod.7  a run repeats (full runs only).
//!
//! `AIL_FUZZ_N` inputs per template (default 40, the selftest smoke), from
//! `AIL_FUZZ_SEED` (default 1); `AIL_FUZZ_MODULE=NAME` picks one template. A
//! failing input is minimised and saved with its program as `NAME-seed-N.ail`
//! under `$AIL_FUZZ_OUT`; `run.log` there records the seeds of every run.
//! Each finding becomes a case in `bootstrap/tests/cases/` (fuzz.policy.1).

#[path = "fuzz/progs.rs"]
#[allow(dead_code)]
mod progs;
#[path = "fuzz/inputs.rs"]
mod inputs;
#[path = "fuzz/runner.rs"]
mod runner;

use inputs::{generate, literal, Input};
use runner::{exec_program, Out, Runner};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

struct Template {
    name: String,
    kind: String,
    needs: String,
    body: String,
}

fn templates(dir: &Path) -> Vec<Template> {
    let mut out = Vec::new();
    let mut files: Vec<PathBuf> = fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|x| x == "ail")).collect();
    files.sort();
    for f in files {
        let body = fs::read_to_string(&f).unwrap();
        let header = |key: &str| body.lines().find_map(|l| l.strip_prefix(&format!("# fuzz-{key}:"))).map(|v| v.trim().to_string());
        let kind = header("input").unwrap_or_else(|| panic!("{}: missing `# fuzz-input:`", f.display()));
        assert!(inputs::KINDS.contains(&kind.as_str()), "{}: unknown input kind {kind}", f.display());
        out.push(Template { name: f.file_stem().unwrap().to_string_lossy().into_owned(), kind, needs: header("needs").unwrap_or_default(), body });
    }
    out
}

fn fill(t: &Template, i: &Input) -> String {
    // `{{BYTES}}` is the hex secondary input as a list of ints.
    let bytes = |h: &str| format!("[{}]", (0..h.len() / 2).map(|k| i64::from_str_radix(&h[2 * k..2 * k + 2], 16).unwrap_or(0).to_string()).collect::<Vec<_>>().join(", "));
    let body = if t.body.contains("{{BYTES}}") { t.body.replace("{{BYTES}}", &bytes(&i.text2)) } else { t.body.clone() };
    body.replace("{{TEXT2}}", &literal(&i.text2)).replace("{{TEXT}}", &literal(&i.text)).replace("{{INT}}", &i.int.to_string())
}

fn smoke() -> bool {
    std::env::var("AIL_FUZZ_N").is_err()
}

fn env_num(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// The bootstrap binary's `build` command, if lane 120 has landed it: `ail build FILE -o OUT`.
fn native(r: &Runner, p: &Path) -> Option<PathBuf> {
    let out = r.dir.join(if cfg!(windows) { "native.exe" } else { "native" });
    let _ = fs::remove_file(&out);
    let args = ["build", p.to_str()?, "-o", out.to_str()?];
    let b = exec_program(Path::new(env!("CARGO_BIN_EXE_ail")), &args, 60);
    (b.code == Some(0) && out.exists()).then_some(out)
}

fn has_native() -> bool {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("fuzz-modules-probe");
    fs::create_dir_all(&dir).unwrap();
    let r = Runner { dir };
    let p = r.write("p.ail", "fn main()\n    print(1)\n");
    native(&r, &p).is_some()
}

fn same(a: &Out, b: &Out) -> bool {
    a.code == b.code && a.stdout == b.stdout && a.stderr == b.stderr
}

/// The first property the program breaks on this input, if any.
fn check(r: &Runner, src: &str, with_native: bool) -> Option<String> {
    let p = r.write("m.ail", src);
    let c = r.exec(&["check", "--json"], &p, 20);
    if let Some(m) = c.crashed() {
        return Some(format!("fuzz.mod.1 check: {m}"));
    }
    if c.code != Some(0) {
        return Some(format!("fuzz.mod.0 template does not check: {}", c.stdout.lines().chain(c.stderr.lines()).next().unwrap_or("")));
    }
    let walker = r.exec(&["run", "--no-contracts", "--deterministic"], &p, 10);
    if walker.code.is_none() {
        return Some("fuzz.mod.2 hang: no result in 10 s".into());
    }
    if let Some(m) = walker.crashed() {
        return Some(format!("fuzz.mod.1 run: {m}"));
    }
    if let Some(l) = walker.stdout.lines().find(|l| l.starts_with("FAIL")) {
        return Some(format!("fuzz.mod.3 oracle: {l}"));
    }
    if walker.stderr.contains("bootstrap.") {
        return Some("fuzz.mod.4 bootstrap.* error while running".into());
    }
    if walker.code != Some(0) && !walker.stderr.contains('[') {
        return Some(format!("fuzz.mod.4 exit {:?} without a rule id on stderr", walker.code));
    }
    // `--deterministic` keeps a run on the walker; denying the clock gives the tier the same sandbox.
    let tier = r.exec(&["run", "--no-contracts", "--deny", "clock,random,proc,net"], &p, 10);
    if !same(&walker, &tier) {
        return Some(format!(
            "fuzz.mod.5 the bytecode tier changed the run\n--- walker (exit {:?})\n{}{}--- tier (exit {:?})\n{}{}",
            walker.code, walker.stdout, walker.stderr, tier.code, tier.stdout, tier.stderr
        ));
    }
    if with_native {
        if let Some(exe) = native(r, &p) {
            let n = exec_program(&exe, &[], 10);
            if n.code != walker.code || n.stdout != walker.stdout {
                return Some(format!("fuzz.mod.6 native differs (exit {:?} against {:?})\n{}", n.code, walker.code, n.stdout));
            }
        }
    }
    if !smoke() {
        let again = r.exec(&["run", "--no-contracts", "--deterministic"], &p, 10);
        if !same(&walker, &again) {
            return Some("fuzz.mod.7 run is not deterministic".into());
        }
    }
    None
}

/// Shrink one text by removing chunks while the same property still breaks.
fn shrink(text: &str, fails: &dyn Fn(&str) -> bool) -> String {
    let mut cs: Vec<char> = text.chars().collect();
    let mut chunk = cs.len().div_ceil(2).max(1);
    let mut budget = 120;
    loop {
        let mut i = 0;
        while i < cs.len() && budget > 0 {
            let end = (i + chunk).min(cs.len());
            let mut trial = cs[..i].to_vec();
            trial.extend_from_slice(&cs[end..]);
            budget -= 1;
            if fails(&trial.iter().collect::<String>()) {
                cs = trial;
            } else {
                i += chunk;
            }
        }
        if chunk == 1 || budget == 0 {
            break;
        }
        chunk = chunk.div_ceil(2);
    }
    cs.into_iter().collect()
}

/// What `kind` of failure: the property id and its first words, so shrinking can't drift to another bug.
fn tag(msg: &str) -> String {
    msg.lines().next().unwrap_or("").split(':').next().unwrap_or("").to_string()
}

#[test]
fn fuzz_module_inputs() {
    let n = env_num("AIL_FUZZ_N", 40) as usize;
    let first = env_num("AIL_FUZZ_SEED", 1);
    let only = std::env::var("AIL_FUZZ_MODULE").ok();
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("fuzz-modules");
    let out_dir = std::env::var("AIL_FUZZ_OUT").map(PathBuf::from).unwrap_or_else(|_| root.join("failures"));
    fs::create_dir_all(&out_dir).unwrap();
    let with_native = has_native();
    let workers = std::thread::available_parallelism().map_or(2, |p| p.get()).clamp(1, 8);

    // Which templates can run: their module must exist.
    let probe = Runner { dir: root.join("probe") };
    fs::create_dir_all(&probe.dir).unwrap();
    let mut live: Vec<Template> = Vec::new();
    let mut template_errors: Vec<String> = Vec::new();
    let mut log = vec![format!("fuzz-modules: n={n} first-seed={first} native={with_native}")];
    for t in templates(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fuzz/modules")) {
        if only.as_deref().is_some_and(|o| o != t.name) {
            continue;
        }
        if !t.needs.is_empty() {
            let p = probe.write("needs.ail", &format!("use {}\n\nfn main()\n    print(1)\n", t.needs));
            if probe.exec(&["check"], &p, 20).code != Some(0) {
                log.push(format!("{}: skipped, {} is not merged yet", t.name, t.needs));
                continue;
            }
        }
        // A template that doesn't check is one finding, not one per input.
        let pre = check(&probe, &fill(&t, &generate(&t.kind, first)), false);
        if let Some(m) = pre.filter(|m| m.starts_with("fuzz.mod.0")) {
            template_errors.push(format!("{}: {m}", t.name));
            continue;
        }
        live.push(t);
    }

    let tasks: Vec<(usize, u64)> = (0..live.len()).flat_map(|t| (0..n).map(move |i| (t, first + i as u64))).collect();
    let next = AtomicUsize::new(0);
    let failures: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let per_module: Vec<AtomicUsize> = live.iter().map(|_| AtomicUsize::new(0)).collect();
    std::thread::scope(|s| {
        for w in 0..workers {
            let (next, failures, tasks, live, per_module, root, out_dir) = (&next, &failures, &tasks, &live, &per_module, &root, &out_dir);
            s.spawn(move || {
                let dir = root.join(format!("w{w}"));
                fs::create_dir_all(&dir).unwrap();
                let r = Runner { dir };
                loop {
                    let k = next.fetch_add(1, Ordering::SeqCst);
                    let Some(&(ti, seed)) = tasks.get(k) else { break };
                    let t = &live[ti];
                    let mut input = generate(&t.kind, seed);
                    let Some(msg) = check(&r, &fill(t, &input), with_native) else { continue };
                    per_module[ti].fetch_add(1, Ordering::SeqCst);
                    let want = tag(&msg);
                    let broken = |i: &Input| check(&r, &fill(t, i), with_native).is_some_and(|m| tag(&m) == want);
                    input.text = shrink(&input.text, &|s| broken(&Input { text: s.into(), text2: input.text2.clone(), int: input.int }));
                    input.text2 = shrink(&input.text2, &|s| broken(&Input { text: input.text.clone(), text2: s.into(), int: input.int }));
                    let file = out_dir.join(format!("{}-seed-{seed}.ail", t.name));
                    fs::write(&file, fill(t, &input)).unwrap();
                    failures.lock().unwrap().push(format!(
                        "{} seed {seed}: {msg}\n  minimised program: {}\n  reproduce: AIL_FUZZ_MODULE={} AIL_FUZZ_SEED={seed} AIL_FUZZ_N=1 ./ail selftest --test fuzz_modules -- --nocapture",
                        t.name, file.display(), t.name
                    ));
                }
            });
        }
    });
    for (t, bad) in live.iter().zip(&per_module) {
        log.push(format!("{}: {n} inputs from seed {first} ({}), {} failures", t.name, t.kind, bad.load(Ordering::SeqCst)));
    }
    let mut failures = failures.into_inner().unwrap();
    failures.extend(template_errors);
    let log = log.join("\n");
    eprintln!("{log}");
    let _ = fs::write(out_dir.join("run.log"), format!("{log}\n"));
    assert!(failures.is_empty(), "{} module fuzz failures (fuzz.policy.1: each becomes a case)\n{}", failures.len(), failures.join("\n"));
}
