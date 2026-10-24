//! Run limits and coverage (run.limit.1-4, test.cover.1-3). Both hang off one
//! flag the interpreter tests once per statement, so a run without them pays
//! one predictable branch and nothing else.

use super::*;
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering as AtomicOrdering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// What `--timeout`, `--max-memory` and `--coverage` ask for.
#[derive(Default, Clone, Copy)]
pub struct Config {
    pub timeout_ms: Option<u64>,
    pub memory_mb: Option<u64>,
    pub coverage: bool,
}

static CONFIG: OnceLock<Config> = OnceLock::new();
/// Set by the watchdog thread: 0 running, 1 timed out, 2 over memory.
static STOP: AtomicU8 = AtomicU8::new(0);
/// The lines of statements that ran, across every interpreter thread.
static HITS: Mutex<BTreeSet<usize>> = Mutex::new(BTreeSet::new());

/// ct.eval.3: the most statements a constant may run (`AIL_CT_MAX_STEPS` lowers it, for tests).
fn ct_max_steps() -> u64 {
    static MAX: OnceLock<u64> = OnceLock::new();
    *MAX.get_or_init(|| {
        std::env::var("AIL_CT_MAX_STEPS").ok().and_then(|v| v.parse().ok()).unwrap_or(100_000_000)
    })
}

pub fn configure(c: Config) {
    let _ = CONFIG.set(c);
}

fn config() -> Config {
    CONFIG.get().copied().unwrap_or_default()
}

/// Whether any limit or coverage is on (the interpreter's `hook`).
pub(super) fn active() -> bool {
    let c = config();
    c.timeout_ms.is_some() || c.memory_mb.is_some() || c.coverage
}

/// The resident memory of this process in MB, where the platform tells us.
fn resident_mb() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        let kb: u64 = status.lines().find_map(|l| l.strip_prefix("VmRSS:"))?.trim().trim_end_matches("kB").trim().parse().ok()?;
        Some(kb / 1024)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let out = std::process::Command::new("ps").args(["-o", "rss=", "-p", &std::process::id().to_string()]).output().ok()?;
        let kb: u64 = String::from_utf8_lossy(&out.stdout).trim().parse().ok()?;
        Some(kb / 1024)
    }
}

/// Whether `--max-memory` can be honoured here (run.limit.3).
pub fn memory_supported() -> bool {
    !cfg!(windows)
}

/// A running watchdog; dropping it stops the thread (run.limit.1-2).
pub struct Watch(Arc<AtomicBool>);

impl Drop for Watch {
    fn drop(&mut self) {
        self.0.store(true, AtomicOrdering::Relaxed);
    }
}

/// Start the clock for one run, or one test: clears an earlier stop and, when
/// a time or memory limit is set, watches from a thread of its own.
pub fn watch() -> Option<Watch> {
    STOP.store(0, AtomicOrdering::Relaxed);
    let c = config();
    if c.timeout_ms.is_none() && c.memory_mb.is_none() {
        return None;
    }
    let done = Arc::new(AtomicBool::new(false));
    let flag = done.clone();
    let start = Instant::now();
    std::thread::spawn(move || {
        while !flag.load(AtomicOrdering::Relaxed) {
            if c.timeout_ms.is_some_and(|ms| start.elapsed() >= Duration::from_millis(ms)) {
                STOP.store(1, AtomicOrdering::Relaxed);
                return;
            }
            if c.memory_mb.is_some_and(|mb| resident_mb().is_some_and(|now| now > mb)) {
                STOP.store(2, AtomicOrdering::Relaxed);
                return;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    });
    Some(Watch(done))
}

impl<'a, W: Write> Interp<'a, W> {
    /// Called before each statement, and each turn of a `while`, when `hook`
    /// is on: enforce the limits (run.limit.1-3) and note the line (test.cover.1).
    #[inline(never)]
    pub(super) fn hit(&mut self, pos: Pos) -> Result<(), Exit> {
        let c = config();
        let stop = STOP.load(AtomicOrdering::Relaxed);
        if stop != 0 {
            return Err(if stop == 1 {
                Trap {
                    rule: "run.limit.1",
                    pos,
                    message: format!("run took longer than --timeout {} ms (raise it, or find the loop)", c.timeout_ms.unwrap_or(0)),
                }
            } else {
                Trap {
                    rule: "run.limit.2",
                    pos,
                    message: format!("run used more than --max-memory {} MB (raise it, or find what grows)", c.memory_mb.unwrap_or(0)),
                }
            }
            .into());
        }
        // ct.eval.3: a constant may run at most `CT_MAX_STEPS` statements.
        if let Some((name, n)) = self.ct_eval.as_mut() {
            *n += 1;
            let max = ct_max_steps();
            if *n > max {
                return Err(Trap {
                    rule: "ct.eval.3",
                    pos,
                    message: format!("computing the constant `{name}` took more than {max} evaluation steps (a step is one statement run); simplify it or compute it at run time"),
                }
                .into());
            }
        }
        if c.coverage && pos.line != self.cov_last {
            self.cov_last = pos.line;
            HITS.lock().unwrap().insert(pos.line);
        }
        Ok(())
    }

    /// Take back the step a bytecode `Hit` took for a statement it then
    /// hands to the walker's `stmt`, which takes it again (ADR 0364).
    pub(super) fn unhit(&mut self) {
    }

    /// Reset the step count for the next test (run.limit.3).
    pub(super) fn reset_limits(&mut self) {
        self.steps = 0;
    }
}

/// One file's coverage (test.cover.1).
pub struct FileCover {
    pub file: String,
    pub lines: usize,
    pub lines_hit: usize,
    pub fns: usize,
    pub fns_hit: usize,
    /// Unexecuted lines as inclusive ranges, in file order.
    pub missed: Vec<(usize, usize)>,
}

fn lines_of(b: &Block, out: &mut Vec<usize>) {
    for s in b {
        out.push(s.pos().line);
        if let Stmt::Defer(inner, _) = s {
            out.push(inner.pos().line);
        }
        crate::check::parallel::each_block(s, &mut |inner| lines_of(inner, out));
    }
}

/// Coverage of every non-test function in the program's own files, from the
/// lines that ran. `line_map` maps doc-test-augmented lines back (test.doc.3).
pub fn report(prog: &Program, line_map: &[usize]) -> Vec<FileCover> {
    let hits = HITS.lock().unwrap().clone();
    let orig = |line: usize| -> Option<usize> {
        let rel = line.checked_sub(prog.file_at(Pos { line, col: 1 })?.base)?;
        if line_map.is_empty() { return Some(rel) }
        line_map.get(rel.wrapping_sub(1)).copied().filter(|l| *l != 0)
    };
    let mut files: Vec<FileCover> = Vec::new();
    for f in prog.fns.iter().filter(|f| f.top_level() && !f.is_test() && f.body.is_some()) {
        // Generated code (ct.gen.1) isn't the program's own lines.
        let Some(sf) = prog.file_at(f.pos).filter(|sf| !sf.path.starts_with("gen:")) else { continue };
        let mut lines = Vec::new();
        lines_of(f.body.as_ref().unwrap(), &mut lines);
        let all: BTreeSet<usize> = lines.iter().copied().collect();
        let at = match files.iter().position(|c| c.file == sf.path) {
            Some(i) => i,
            None => {
                files.push(FileCover { file: sf.path.clone(), lines: 0, lines_hit: 0, fns: 0, fns_hit: 0, missed: Vec::new() });
                files.len() - 1
            }
        };
        let c = &mut files[at];
        c.fns += 1;
        if lines.first().is_some_and(|l| hits.contains(l)) {
            c.fns_hit += 1;
        }
        for l in all {
            let Some(shown) = orig(l) else { continue };
            c.lines += 1;
            if hits.contains(&l) {
                c.lines_hit += 1;
            } else if let Some(last) = c.missed.last_mut().filter(|(_, e)| *e + 1 == shown) {
                last.1 = shown;
            } else {
                c.missed.push((shown, shown));
            }
        }
    }
    for c in &mut files {
        c.missed.sort_unstable();
    }
    files
}
