//! The bytecode tier against the tree walker (ADR 0359, ADR 0361, ADR 0362).
//!
//! Every example, test case, perfbench program, tokbench corpus program and
//! package runs twice through the real `ail` binary, once with
//! `--no-bytecode`: standard output, standard error and the exit code must
//! be the same bytes. Files with a `.test` expectation also run `ail test
//! --json` both ways. Programs that read the clock or unseeded random
//! numbers live in `tests/deterministic/`, which isn't included.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::Mutex;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn ail_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for e in entries {
        let p = e.unwrap().path();
        if p.extension().is_some_and(|x| x == "ail") {
            out.push(p);
        }
    }
}

/// The command lines to compare, without the tier flag: `run [ARGS] FILE`, and `test --json FILE`.
fn jobs() -> Vec<Vec<String>> {
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
        let path = f.display().to_string();
        let args = fs::read_to_string(f.with_extension("args")).unwrap_or_default();
        let mut run: Vec<String> = vec!["run".into()];
        run.extend(args.split_whitespace().map(String::from));
        run.push(path.clone());
        jobs.push(run);
        if f.with_extension("test").exists() {
            jobs.push(vec!["test".into(), "--json".into(), path]);
        }
    }
    let mut pkgs: Vec<PathBuf> = fs::read_dir(r.join("bootstrap/tests/cases/packages"))
        .map(|d| d.map(|e| e.unwrap().path()).filter(|p| p.is_dir()).collect())
        .unwrap_or_default();
    pkgs.sort();
    jobs.extend(pkgs.into_iter().map(|p| vec!["run".into(), p.display().to_string()]));
    jobs
}

fn run(job: &[String], walker: bool) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ail"));
    cmd.arg(&job[0]);
    if walker {
        cmd.arg("--no-bytecode");
    }
    cmd.args(&job[1..]).current_dir(root()).stdin(Stdio::null()).output().unwrap()
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
    assert!(jobs.len() > 400, "found only {} programs", jobs.len());
    let next = Mutex::new(0usize);
    let failures = Mutex::new(Vec::new());
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).min(8);
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| loop {
                let i = {
                    let mut n = next.lock().unwrap();
                    *n += 1;
                    *n - 1
                };
                let Some(job) = jobs.get(i) else { break };
                let (w, t) = (run(job, true), run(job, false));
                if let Some(d) = differ(job, &w, &t) {
                    failures.lock().unwrap().push(d);
                }
            });
        }
    });
    let failures = failures.into_inner().unwrap();
    assert!(failures.is_empty(), "{} run(s) differ between the tiers:\n\n{}", failures.len(), failures.join("\n\n"));
}
