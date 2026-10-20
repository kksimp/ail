//! The bytecode tier against the tree walker (ADR 0359, ADR 0361, ADR 0362).
//!
//! Every example, test case, perfbench program, tokbench corpus program and
//! package runs twice through the real `ail` binary, once with
//! `--no-bytecode`: standard output, standard error and the exit code must
//! be the same bytes. Files with a `.test` expectation also run `ail test
//! --json` both ways. Programs that read the clock or unseeded random
//! numbers live in `tests/deterministic/`, which isn't included; programs
//! that open a window (`ui.open(`) wait for it to close, so they're skipped.
//! A run that takes longer than `LIMIT` is killed and fails the test.
//!
//! Programs without tasks also run with `--max-steps 300`, which stops most
//! of them part way with a trap and its snapshot, and with `--trace=all`;
//! `.test` files also run with `--coverage` (ADR 0364). Programs with tasks
//! interleave their steps and trace lines differently from run to run, on
//! either tier, so they're left out of those.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// The longest a run may take; every program here finishes in about a second.
const LIMIT: Duration = Duration::from_secs(60);

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn ail_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for e in entries {
        let p = e.unwrap().path();
        if p.extension().is_some_and(|x| x == "ail")
            && !fs::read_to_string(&p).is_ok_and(|src| src.contains("ui.open("))
        {
            out.push(p);
        }
    }
}

/// Words that mark a program whose tasks or threads make its step count and
/// trace order vary between runs.
const CONCURRENT: [&str; 6] = ["spawn", "par ", "TaskGroup", "Atomic", "Channel", "Mutex"];

/// The command lines to compare, without the tier flag: `run [ARGS] FILE`,
/// the same with a step limit and with a trace, and `test --json FILE`.
/// One job is one program's runs, made in turn, since some programs write
/// files at fixed paths.
fn jobs() -> Vec<Vec<Vec<String>>> {
    let r = root();
    let mut files = Vec::new();
    for dir in ["examples", "bootstrap/tests/cases", "tools/perfbench/programs"] {
        ail_files(&r.join(dir), &mut files);
    }
    if let Ok(entries) = fs::read_dir(r.join("tools/tokbench/corpus/programs")) {
        for e in entries {
            ail_files(&e.unwrap().path(), &mut files);
        }
    }
    files.sort();
    let mut jobs = Vec::new();
    for f in &files {
        let mut job = Vec::new();
        let path = f.display().to_string();
        let args = fs::read_to_string(f.with_extension("args")).unwrap_or_default();
        let src = fs::read_to_string(f).unwrap_or_default();
        let steady = !CONCURRENT.iter().any(|w| src.contains(w));
        let extra: &[&[&str]] = if steady { &[&[], &["--max-steps", "300"], &["--trace=all"]] } else { &[&[]] };
        for flags in extra {
            let mut run: Vec<String> = vec!["run".into()];
            run.extend(flags.iter().map(|x| x.to_string()));
            run.extend(args.split_whitespace().map(String::from));
            run.push(path.clone());
            job.push(run);
        }
        if f.with_extension("test").exists() {
            job.push(vec!["test".into(), "--json".into(), path.clone()]);
            if steady {
                job.push(vec!["test".into(), "--coverage".into(), path]);
            }
        }
        jobs.push(job);
    }
    let mut pkgs: Vec<PathBuf> = fs::read_dir(r.join("bootstrap/tests/cases/packages"))
        .map(|d| d.map(|e| e.unwrap().path()).filter(|p| p.is_dir()).collect())
        .unwrap_or_default();
    pkgs.sort();
    jobs.extend(pkgs.into_iter().map(|p| vec![vec!["run".into(), p.display().to_string()]]));
    jobs
}

/// Run one side of a command line, or say that it ran past `LIMIT`.
fn run(job: &[String], walker: bool) -> Result<Output, String> {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ail"));
    cmd.arg(&job[0]);
    if walker {
        cmd.arg("--no-bytecode");
    }
    cmd.args(&job[1..]).current_dir(root()).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().unwrap();
    // The pipes are drained while the program runs, so a large output can't block it.
    let drain = |child: &mut Child, out: bool| {
        let mut pipe: Box<dyn Read + Send> =
            if out { Box::new(child.stdout.take().unwrap()) } else { Box::new(child.stderr.take().unwrap()) };
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            pipe.read_to_end(&mut buf).unwrap();
            buf
        })
    };
    let (stdout, stderr) = (drain(&mut child, true), drain(&mut child, false));
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if start.elapsed() > LIMIT {
            child.kill().unwrap();
            child.wait().unwrap();
            let side = if walker { "walker" } else { "tier" };
            return Err(format!("ail {} ({side}): still running after {}s, killed", job.join(" "), LIMIT.as_secs()));
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    Ok(Output { status, stdout: stdout.join().unwrap(), stderr: stderr.join().unwrap() })
}

/// A short report of what differs between the walker's run and the tier's.
fn differ(job: &[String], w: &Output, t: &Output) -> Option<String> {
    if w.status.code() == t.status.code() && w.stdout == t.stdout && w.stderr == t.stderr {
        return None;
    }
    let text = |b: &[u8]| String::from_utf8_lossy(b).into_owned();
    Some(format!(
        "ail {}\n--- walker (exit {:?})\n{}{}--- tier (exit {:?})\n{}{}",
        job.join(" "),
        w.status.code(),
        text(&w.stdout),
        text(&w.stderr),
        t.status.code(),
        text(&t.stdout),
        text(&t.stderr)
    ))
}

#[test]
fn tier_matches_the_walker() {
    let jobs = jobs();
    let runs: usize = jobs.iter().map(Vec::len).sum();
    assert!(jobs.len() > 400 && runs > 1200, "found only {} programs, {runs} runs", jobs.len());
    let next = Mutex::new(0usize);
    let failures = Mutex::new(Vec::new());
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).min(8);
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                loop {
                    let i = {
                        let mut n = next.lock().unwrap();
                        *n += 1;
                        *n - 1
                    };
                    let Some(job) = jobs.get(i) else { break };
                    for cmd in job {
                        let d = match (run(cmd, true), run(cmd, false)) {
                            (Ok(w), Ok(t)) => differ(cmd, &w, &t),
                            (Err(e), _) | (_, Err(e)) => Some(e),
                        };
                        if let Some(d) = d {
                            failures.lock().unwrap().push(d);
                        }
                    }
                }
            });
        }
    });
    let failures = failures.into_inner().unwrap();
    assert!(failures.is_empty(), "{} run(s) differ between the tiers:\n\n{}", failures.len(), failures.join("\n\n"));
}
