//! Streaming processes (std.proc.4-7, ADR 0424): `proc.start` starts a
//! child whose standard streams are channels. Reader threads turn each
//! output stream into lines in a queue (`Ext`) that the scheduler moves into
//! a `Channel[str]` (`tasks.rs`), and a writer thread feeds standard input,
//! so a child can't block on a full pipe and no task blocks the others.
//! Plain Rust; `stdlib.rs` turns the results into values.

use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use crate::stdlib::error_kind;

/// The lines a reader thread has read, and whether the stream has ended.
#[derive(Default)]
pub struct Ext {
    state: Mutex<(VecDeque<String>, bool)>,
}

impl Ext {
    /// Take every queued line and whether the stream ended after them.
    pub fn take(&self) -> (VecDeque<String>, bool) {
        let mut s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        (std::mem::take(&mut s.0), s.1)
    }

    fn push(&self, line: String) {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).0.push_back(line);
        notify();
    }

    fn finish(&self) {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).1 = true;
        notify();
    }
}

/// A counter that every stream event bumps, so a scheduler with nothing to
/// run can sleep until a child writes, ends or exits.
static EVENT: (Mutex<u64>, Condvar) = (Mutex::new(0), Condvar::new());

fn notify() {
    *EVENT.0.lock().unwrap_or_else(|e| e.into_inner()) += 1;
    EVENT.1.notify_all();
}

/// The event counter now.
pub fn event_count() -> u64 {
    *EVENT.0.lock().unwrap_or_else(|e| e.into_inner())
}

/// Sleep until the counter moves past `seen`, or `ms` pass.
pub fn wait_event(seen: u64, ms: u64) {
    let g = EVENT.0.lock().unwrap_or_else(|e| e.into_inner());
    if *g == seen {
        let _ = EVENT.1.wait_timeout(g, Duration::from_millis(ms));
    }
}

struct Live {
    child: Child,
    cmd: String,
    timed_out: Arc<AtomicBool>,
}

/// Children started and not yet waited or killed, by process id.
static CHILDREN: Mutex<Option<HashMap<u32, Live>>> = Mutex::new(None);

fn children() -> std::sync::MutexGuard<'static, Option<HashMap<u32, Live>>> {
    let mut g = CHILDREN.lock().unwrap_or_else(|e| e.into_inner());
    g.get_or_insert_with(HashMap::new);
    g
}

/// A started child: its process id and the three streams.
pub struct Started {
    pub pid: u32,
    pub stdin: Sender<String>,
    pub out: Arc<Ext>,
    pub err: Arc<Ext>,
}

/// std.proc.4: start `cmd` with `args`; after `timeout_ms` (if any) it is
/// killed and `wait` reports `timed_out` (std.proc.6).
pub fn start(cmd: &str, args: &[String], timeout_ms: Option<u64>) -> Result<Started, &'static str> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| error_kind(&e))?;
    let pid = child.id();
    let mut stdin = child.stdin.take().expect("piped");
    let (tx, rx): (Sender<String>, Receiver<String>) = channel();
    std::thread::spawn(move || {
        // Ends when the sender is dropped (`close`); a child that exits
        // without reading its input is not a failure.
        for text in rx {
            if stdin.write_all(text.as_bytes()).is_err() {
                break;
            }
            let _ = stdin.flush();
        }
    });
    let out = Arc::new(Ext::default());
    let err = Arc::new(Ext::default());
    reader(Box::new(child.stdout.take().expect("piped")), Arc::clone(&out));
    reader(Box::new(child.stderr.take().expect("piped")), Arc::clone(&err));
    let timed_out = Arc::new(AtomicBool::new(false));
    children().as_mut().expect("set").insert(pid, Live { child, cmd: cmd.to_string(), timed_out: Arc::clone(&timed_out) });
    if let Some(ms) = timeout_ms {
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(ms));
            if let Some(l) = children().as_mut().expect("set").get_mut(&pid)
                && matches!(l.child.try_wait(), Ok(None))
            {
                timed_out.store(true, Ordering::SeqCst);
                let _ = l.child.kill();
            }
        });
    }
    Ok(Started { pid, stdin: tx, out, err })
}

/// Lines of `r`, without the line end, as text (bytes that aren't UTF-8 as U+FFFD).
fn reader(r: Box<dyn std::io::Read + Send>, ext: Arc<Ext>) {
    std::thread::spawn(move || {
        let mut r = BufReader::new(r);
        let mut buf = Vec::new();
        loop {
            buf.clear();
            match r.read_until(b'\n', &mut buf) {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    if buf.last() == Some(&b'\n') {
                        buf.pop();
                        if buf.last() == Some(&b'\r') {
                            buf.pop();
                        }
                    }
                    ext.push(String::from_utf8_lossy(&buf).into_owned());
                }
            }
        }
        ext.finish();
    });
}

/// How a child ended: its status, and whether the time limit killed it.
pub struct Ended {
    pub status: std::process::ExitStatus,
    pub cmd: String,
    pub timed_out: bool,
}

/// The child `pid` if it has exited (and then forget it); `None` while it runs.
pub fn try_wait(pid: u32) -> Option<Ended> {
    let mut g = children();
    let map = g.as_mut().expect("set");
    let l = map.get_mut(&pid)?;
    let status = l.child.try_wait().ok().flatten()?;
    let l = map.remove(&pid).expect("present");
    Some(Ended { status, cmd: l.cmd, timed_out: l.timed_out.load(Ordering::SeqCst) })
}

/// std.proc.7: kill `pid` and reap it.
pub fn kill(pid: u32) -> Option<Ended> {
    let mut l = children().as_mut().expect("set").remove(&pid)?;
    let _ = l.child.kill();
    let status = l.child.wait().ok()?;
    Some(Ended { status, cmd: l.cmd, timed_out: l.timed_out.load(Ordering::SeqCst) })
}

/// std.proc.7: the program is ending, so no child outlives it.
pub fn reap_all() {
    let mut g = CHILDREN.lock().unwrap_or_else(|e| e.into_inner());
    for (_, mut l) in g.take().unwrap_or_default() {
        let _ = l.child.kill();
        let _ = l.child.wait();
    }
}
