//! Module fuzzers (fuzz.mod.*, ADR 0460): adversarial inputs for the standard
//! modules, through the real `ail` binary.
//!
//! Each template `fuzz/modules/NAME.ail` is a small program with `{{TEXT}}`,
//! `{{TEXT2}}` (string literals) and `{{INT}}` placeholders. A header names
//! its input kind and the module it needs:
//!
//!     # fuzz-input: json|text|bytes|regex|date|http|compress
//!     # fuzz-needs: std.bytes        (blank when built in)
//!     # fuzz-native: yes             (optional: also build with `ail build`, fuzz.mod.6)
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
#[path = "fuzz/pure.rs"]
mod pure;

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
    t.body.replace("{{TEXT2}}", &literal(&i.text2)).replace("{{TEXT}}", &literal(&i.text)).replace("{{INT}}", &i.int.to_string())
}

/// Programs `ail build` accepted and whose output was compared (fuzz.mod.6); the rest are outside the pure subset.
static NATIVE_COMPARED: AtomicUsize = AtomicUsize::new(0);

fn smoke() -> bool {
    std::env::var("AIL_FUZZ_N").is_err()
}

fn env_num(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// The bootstrap binary's `build` command, if lane 120 has landed it: `ail build FILE -o OUT`.
fn native(r: &Runner, p: &Path) -> Option<PathBuf> {
    build_for(r, p, None)
}

/// `ail build --target T`; `None` is the host's.
fn build_for(r: &Runner, p: &Path, target: Option<&str>) -> Option<PathBuf> {
    let out = r.dir.join(format!("native{}", target.unwrap_or("")));
    let _ = fs::remove_file(&out);
    let mut args = vec!["build", p.to_str()?, "-o", out.to_str()?];
    if let Some(t) = target {
        args.extend(["--target", t]);
    }
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
    if with_native && src.contains("# fuzz-native: yes") {
        if let Some(exe) = native(r, &p) {
            NATIVE_COMPARED.fetch_add(1, Ordering::SeqCst);
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
    log.push(format!("native: {} programs built and compared", NATIVE_COMPARED.load(Ordering::SeqCst)));
    let log = log.join("\n");
    eprintln!("{log}");
    let _ = fs::write(out_dir.join("run.log"), format!("{log}\n"));
    assert!(failures.is_empty(), "{} module fuzz failures (fuzz.policy.1: each becomes a case)\n{}", failures.len(), failures.join("\n"));
}

/// The ARM64 Linux user-mode emulator, when installed (`apt install qemu-user`).
fn qemu_aarch64() -> Option<&'static str> {
    if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        return None;
    }
    ["qemu-aarch64-static", "qemu-aarch64"].into_iter().find(|q| std::process::Command::new(q).arg("--version").output().is_ok_and(|o| o.status.success()))
}

/// fuzz.nat.1 (ADR 0462): generated pure-subset programs run on the walker, the bytecode tier
/// and as `ail build` output, and print the same bytes, exit the same, and trap with the same
/// first line. Programs the backend declines are counted and skipped.
#[test]
fn fuzz_native_differential() {
    native_differential("pure", pure::program, true);
}

/// The same for programs that use `str` and `char` (native.str.1). Programs the backend
/// declines are reported, not failed, until the feature has merged everywhere.
#[test]
fn fuzz_native_strings() {
    native_differential("strings", pure::strings, false);
}

#[test]
fn fuzz_native_options() {
    native_differential("options", pure::options, false);
}

#[test]
fn fuzz_native_maps() {
    native_differential("maps", pure::maps, false);
}

fn native_differential(name: &str, make: fn(u64) -> String, need_half: bool) {
    if !has_native() {
        eprintln!("fuzz-native: `ail build` is not available; skipped");
        return;
    }
    let n = if smoke() { 6 } else { (env_num("AIL_FUZZ_N", 40) as usize / 5).max(6) };
    let first = env_num("AIL_FUZZ_SEED", 1);
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("fuzz-native-{name}"));
    let out_dir = std::env::var("AIL_FUZZ_OUT").map(PathBuf::from).unwrap_or_else(|_| root.join("failures"));
    fs::create_dir_all(&out_dir).unwrap();
    let workers = std::thread::available_parallelism().map_or(2, |p| p.get()).clamp(1, 4);
    let next = AtomicUsize::new(0);
    let (built, declined, unchecked, trapped) = (AtomicUsize::new(0), AtomicUsize::new(0), AtomicUsize::new(0), AtomicUsize::new(0));
    let failures: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let qemu = qemu_aarch64();
    let first_line = |o: &Out| o.stderr.lines().next().unwrap_or("").to_string();
    std::thread::scope(|s| {
        for w in 0..workers {
            let (next, built, declined, unchecked, trapped, failures, root, out_dir, qemu) = (&next, &built, &declined, &unchecked, &trapped, &failures, &root, &out_dir, &qemu);
            s.spawn(move || {
                let dir = root.join(format!("w{w}"));
                fs::create_dir_all(&dir).unwrap();
                let r = Runner { dir };
                loop {
                    let k = next.fetch_add(1, Ordering::SeqCst);
                    if k >= n {
                        break;
                    }
                    let seed = first + k as u64;
                    let src = make(seed);
                    let p = r.write("n.ail", &src);
                    if r.exec(&["check"], &p, 20).code != Some(0) {
                        unchecked.fetch_add(1, Ordering::SeqCst);
                        continue;
                    }
                    let Some(exe) = native(&r, &p) else {
                        declined.fetch_add(1, Ordering::SeqCst);
                        continue;
                    };
                    built.fetch_add(1, Ordering::SeqCst);
                    let walker = r.exec(&["run", "--no-contracts", "--deterministic"], &p, 10);
                    let tier = r.exec(&["run", "--no-contracts", "--deny", "clock,random,proc,net"], &p, 10);
                    let nat = exec_program(&exe, &[], 10);
                    let arm = qemu.and_then(|q| build_for(&r, &p, Some("aarch64-linux")).map(|e| exec_program(Path::new(q), &[e.to_str().unwrap()], 20)));
                    if walker.code == Some(1) {
                        trapped.fetch_add(1, Ordering::SeqCst);
                    }
                    let bad = if !same(&walker, &tier) {
                        Some("the bytecode tier differs from the walker".to_string())
                    } else if nat.code != walker.code || nat.stdout != walker.stdout || first_line(&nat) != first_line(&walker) {
                        Some(format!(
                            "native differs: exit {:?} against {:?}\n--- native\n{}{}--- interpreter\n{}{}",
                            nat.code, walker.code, nat.stdout, first_line(&nat), walker.stdout, first_line(&walker)
                        ))
                    } else {
                        None
                    };
                    let bad = bad.or_else(|| {
                        let a = arm.as_ref()?;
                        (a.code != walker.code || a.stdout != walker.stdout || first_line(a) != first_line(&walker))
                            .then(|| format!("aarch64 under qemu differs: exit {:?} against {:?}\n--- aarch64\n{}{}--- interpreter\n{}{}", a.code, walker.code, a.stdout, first_line(a), walker.stdout, first_line(&walker)))
                    });
                    if let Some(msg) = bad {
                        let file = out_dir.join(format!("native-{name}-seed-{seed}.ail"));
                        fs::write(&file, &src).unwrap();
                        failures.lock().unwrap().push(format!("seed {seed}: fuzz.mod.6 {msg}\n  program: {}", file.display()));
                    }
                }
            });
        }
    });
    let (built, declined, unchecked) = (built.into_inner(), declined.into_inner(), unchecked.into_inner());
    eprintln!("fuzz-native {name}: {n} programs from seed {first}: {built} built and compared ({} trapped; aarch64 under qemu: {}), {declined} declined by the backend, {unchecked} rejected by the checker", trapped.into_inner(), if qemu.is_some() { "yes" } else { "no emulator" });
    let failures = failures.into_inner().unwrap();
    assert!(failures.is_empty(), "{} native differential failures (fuzz.policy.1)\n{}", failures.len(), failures.join("\n"));
    assert!(!need_half || built * 2 >= n, "only {built} of {n} generated pure programs built natively ({declined} declined, {unchecked} rejected): the generator and the subset have drifted apart");
}

/// `AIL_FUZZ_PURE=SEED cargo test --test fuzz_modules dump_pure -- --nocapture` prints a pure program.
#[test]
fn dump_pure() {
    if let Ok(seed) = std::env::var("AIL_FUZZ_PURE") {
        print!("{}", pure::program(seed.parse().unwrap()));
    }
}

/// `AIL_FUZZ_PURE=NAME:SEED cargo test --test fuzz_modules dump_pure -- --nocapture` prints a generated program.
#[test]
fn dump_named() {
    if let Ok(v) = std::env::var("AIL_FUZZ_NAMED") {
        let (name, seed) = v.split_once(':').unwrap();
        let seed = seed.parse().unwrap();
        print!("{}", match name { "strings" => pure::strings(seed), "maps" => pure::maps(seed), "options" => pure::options(seed), _ => pure::program(seed) });
    }
}
