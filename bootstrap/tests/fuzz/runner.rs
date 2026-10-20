//! Subprocess runner shared by the module fuzzers (fuzz.mod.*): runs the real
//! `ail` binary with a timeout and captures both streams.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub struct Out {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl Out {
    /// A Rust panic, an interpreter-internal error, a signal or a timeout.
    pub fn crashed(&self) -> Option<String> {
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

/// Run `program args...` for at most `secs` seconds.
pub fn exec_program(program: &Path, args: &[&str], secs: u64) -> Out {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawning a process");
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

/// One scratch directory per worker, so files never collide.
pub struct Runner {
    pub dir: PathBuf,
}

impl Runner {
    pub fn exec(&self, args: &[&str], file: &Path, secs: u64) -> Out {
        let mut all: Vec<&str> = args.to_vec();
        let f = file.to_string_lossy().into_owned();
        all.push(&f);
        exec_program(Path::new(env!("CARGO_BIN_EXE_ail")), &all, secs)
    }

    pub fn write(&self, name: &str, src: &str) -> PathBuf {
        let p = self.dir.join(name);
        fs::write(&p, src).unwrap();
        p
    }
}
