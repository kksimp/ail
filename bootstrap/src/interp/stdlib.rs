//! The standard modules at run time (spec 17-std): files (std.fs.1-12), the
//! environment (std.env.1-5), standard input, standard error and exit
//! (std.io.1-4), processes (std.proc.1-3), HTTP (std.net.1-4) and the clock
//! (std.time.1-2).
//! Failures are `fs.IoError` and `proc.ProcError` values whose `kind` is
//! one of the short names of std.err.1.

use std::io::{BufRead, Read, Write};
use std::rc::Rc;
use std::sync::OnceLock;

use super::builtins::Table;
use super::{random, slot_mut, Env, Exit, Failure, Flow, Interp, Trap, Value};
use crate::ast::Arg;
use crate::diag::Pos;
use crate::stdlib::{StdCall, error_kind};

/// The program's path as given to `ail run` (std.env.4), and its
/// arguments, the words after it (std.env.1).
static PROGRAM: OnceLock<(String, Vec<String>)> = OnceLock::new();

/// Record the program's path and arguments before it runs (std.env.1, std.env.4).
pub fn set_program(path: &str, args: Vec<String>) {
    let _ = PROGRAM.set((path.to_string(), args));
}

/// The program's path as given to `ail run`, for positions (trace.run.2).
pub(super) fn program_path() -> String {
    PROGRAM.get().map_or_else(String::new, |(p, _)| p.clone())
}

fn text(s: impl Into<Rc<str>>) -> Value {
    Value::Str(s.into())
}

impl<'a, W: Write> Interp<'a, W> {
    /// A call the checker resolved to a standard module.
    pub(super) fn std_call(&mut self, call: StdCall, args: &'a [Arg], pos: Pos, env: &mut Env<'a>) -> Flow<Value> {
        let (module, f) = match call {
            StdCall::Type(s) => {
                let fields = self.labelled(&self.prog.structs[s].fields, args, env)?;
                return Ok(Value::Struct(s, Rc::new(fields)));
            }
            StdCall::Fn(module, f) => (module, f),
            StdCall::Method(_) => unreachable!("interp::method_call runs `ui.Window` methods"),
            StdCall::Const(_) => unreachable!("interp::eval reads module constants"),
            StdCall::Tensor(_) => unreachable!("interp::method_call runs tensor methods"),
            StdCall::Ptr(..) => unreachable!("interp::method_call runs pointer methods"),
        };
        if module == "tensor" {
            return self.tensor_call(f, args, pos, env);
        }
        if (module, f) == ("random", "shuffle") {
            let p = self.place(&args[0].value, env)?;
            let Value::List(items) = slot_mut(env, &p, false)? else { unreachable!("checked: a list") };
            let items = Rc::make_mut(items);
            // Fisher-Yates (std.random.4).
            for i in (1..items.len()).rev() {
                items.swap(i, random::below(i as u64 + 1) as usize);
            }
            return Ok(Value::Unit);
        }
        let mut values = Vec::with_capacity(args.len());
        for a in args {
            values.push(self.eval(&a.value, env)?);
        }
        if self.tracing & super::trace::EFFECTS != 0 {
            self.trace_effect(module, f, &values, pos);
        }
        if matches!(module, "hash" | "compress") {
            return Ok(self.codec_call(module, f, &values, pos));
        }
        // replay.cap.1: a recorded run's effects come from the recording.
        if super::replay::on() && super::replay::captured(module, f) {
            return self.captured_call(module, f, values, pos);
        }
        self.std_effect(module, f, values, pos)
    }

    /// A standard call that reaches the world (or the program's own state).
    pub(super) fn std_effect(&mut self, module: &str, f: &str, values: Vec<Value>, pos: Pos) -> Flow<Value> {
        let str_arg = |i: usize| match &values[i] {
            Value::Str(s) => s.clone(),
            other => unreachable!("checked: str, found {other:?}"),
        };
        Ok(match (module, f) {
            ("fs", "read_text") => {
                let path = str_arg(0);
                match read_text(&path) {
                    Ok(s) => text(s),
                    Err(kind) => self.io_error(&path, kind, pos),
                }
            }
            ("fs", "write_text" | "append_text") => {
                let (path, contents) = (str_arg(0), str_arg(1));
                let result = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(f == "append_text")
                    .truncate(f == "write_text")
                    .open(&*path)
                    .and_then(|mut file| file.write_all(contents.as_bytes()));
                match result {
                    Ok(()) => Value::Unit,
                    Err(e) => self.io_error(&path, dir_kind(&path, &e), pos),
                }
            }
            // std.fs.13
            ("fs", "read_bytes") => {
                let path = str_arg(0);
                let result = if std::fs::metadata(&*path).is_ok_and(|m| m.is_dir()) {
                    Err("is_dir")
                } else {
                    std::fs::read(&*path).map_err(|e| error_kind(&e))
                };
                match result {
                    Ok(b) => super::bytes::bytes_value(b),
                    Err(kind) => self.io_error(&path, kind, pos),
                }
            }
            // std.fs.14
            ("fs", "write_bytes" | "append_bytes") => {
                let path = str_arg(0);
                let Value::Bytes(contents) = &values[1] else { unreachable!("checked: bytes") };
                let result = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(f == "append_bytes")
                    .truncate(f == "write_bytes")
                    .open(&*path)
                    .and_then(|mut file| file.write_all(contents));
                match result {
                    Ok(()) => Value::Unit,
                    Err(e) => self.io_error(&path, dir_kind(&path, &e), pos),
                }
            }
            ("fs", "exists") => Value::Bool(std::path::Path::new(&*str_arg(0)).exists()),
            ("fs", "list_dir") => {
                let path = str_arg(0);
                match list_dir(&path) {
                    Ok(names) => Value::List(Rc::new(names.into_iter().map(text).collect())),
                    Err(kind) => self.io_error(&path, kind, pos),
                }
            }
            ("fs", "remove") => {
                let path = str_arg(0);
                // std.fs.6: a file, or an empty directory, never a tree.
                let result = match std::fs::symlink_metadata(&*path) {
                    Ok(m) if m.is_dir() => std::fs::remove_dir(&*path),
                    Ok(_) => std::fs::remove_file(&*path),
                    Err(e) => Err(e),
                };
                match result {
                    Ok(()) => Value::Unit,
                    Err(e) => self.io_error(&path, error_kind(&e), pos),
                }
            }
            ("fs", "make_dir") => {
                let path = str_arg(0);
                // std.fs.7: with any missing parents; an existing directory is fine.
                match std::fs::create_dir_all(&*path) {
                    Ok(()) => Value::Unit,
                    Err(e) => self.io_error(&path, error_kind(&e), pos),
                }
            }
            ("fs", "walk") => {
                let path = str_arg(0);
                match super::osext::walk(&path) {
                    Ok(files) => Value::List(Rc::new(files.into_iter().map(text).collect())),
                    Err((at, kind)) => self.io_error(&at.into(), kind, pos),
                }
            }
            ("fs", "glob") => Value::List(Rc::new(super::osext::glob(&str_arg(0)).into_iter().map(text).collect())),
            ("fs", "metadata") => {
                let path = str_arg(0);
                match super::osext::metadata(&path) {
                    Ok((size, modified, is_dir)) => {
                        let s = self.structs["fs.Metadata"];
                        Value::Struct(s, Rc::new(vec![Value::Int(size), Value::Int(modified), Value::Bool(is_dir)]))
                    }
                    Err(kind) => self.io_error(&path, kind, pos),
                }
            }
            ("fs", "rename" | "copy") => match super::osext::move_or_copy(&str_arg(0), &str_arg(1), f == "copy") {
                Ok(()) => Value::Unit,
                Err((at, kind)) => self.io_error(&at.into(), kind, pos),
            },
            ("path", "dir") => text(crate::stdlib::path_dir(&str_arg(0))),
            ("path", "base") => text(crate::stdlib::path_split(&str_arg(0)).1),
            ("path", "ext") => text(crate::stdlib::path_ext(&str_arg(0))),
            ("path", "join") => text(crate::stdlib::path_join(&str_arg(0), &str_arg(1))),
            ("env", "args") => {
                let args = PROGRAM.get().map(|(_, a)| a.as_slice()).unwrap_or_default();
                Value::List(Rc::new(args.iter().map(|a| text(a.as_str())).collect()))
            }
            ("env", "get") => match std::env::var(&*str_arg(0)) {
                Ok(v) => text(v),
                Err(_) => Value::None,
            },
            ("env", "vars") => {
                let mut t = Table::default();
                for (k, v) in super::osext::vars() {
                    t.insert(text(k), Some(text(v)));
                }
                Value::Map(Rc::new(t))
            }
            // std.env.6: HOME, else USERPROFILE (Windows).
            ("env", "home") => match std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
                Ok(v) if !v.is_empty() => text(v),
                _ => Value::None,
            },
            ("env", "program") => text(PROGRAM.get().map_or("", |(p, _)| p.as_str())),
            ("env", "cwd") => match std::env::current_dir() {
                Ok(d) => text(d.to_string_lossy().into_owned()),
                Err(e) => return stop("std.env.3", pos, format!("can't read the working directory: {e}")),
            },
            ("io", "read_line") => {
                self.flush(pos)?;
                let mut line = String::new();
                let mut bytes = Vec::new();
                match std::io::stdin().lock().read_until(b'\n', &mut bytes) {
                    Ok(0) => Value::None,
                    Ok(_) => {
                        line.push_str(&String::from_utf8_lossy(&bytes));
                        let end = line.strip_suffix('\n').map_or(line.as_str(), |l| l.strip_suffix('\r').unwrap_or(l));
                        text(end)
                    }
                    Err(e) => return stop("std.io.1", pos, format!("reading standard input failed: {e}")),
                }
            }
            ("io", "read_all") => {
                self.flush(pos)?;
                let mut bytes = Vec::new();
                if let Err(e) = std::io::stdin().lock().read_to_end(&mut bytes) {
                    return stop("std.io.2", pos, format!("reading standard input failed: {e}"));
                }
                text(String::from_utf8_lossy(&bytes).into_owned())
            }
            // std.io.5: the rest of standard input, undecoded.
            ("io", "read_bytes") => {
                self.flush(pos)?;
                let mut bytes = Vec::new();
                if let Err(e) = std::io::stdin().lock().read_to_end(&mut bytes) {
                    return stop("std.io.2", pos, format!("reading standard input failed: {e}"));
                }
                super::bytes::bytes_value(bytes)
            }
            ("io", "eprint") => {
                let line = self.text(&values[0], pos)?;
                // Keep standard output and standard error in the order written.
                self.flush(pos)?;
                let mut err = std::io::stderr().lock();
                let _ = writeln!(err, "{line}");
                Value::Unit
            }
            ("io", "exit") => {
                let code = values[0].int();
                if !(0..=255).contains(&code) {
                    return stop("std.io.4", pos, format!("exit status {code} is outside 0 to 255"));
                }
                self.flush(pos)?;
                super::spawn::reap_all();
                super::replay::finish();
                std::process::exit(code as i32);
            }
            ("proc", "run" | "run_bytes") => {
                // std.proc.8: `run_bytes` takes `bytes` input and gives `bytes` output.
                let raw = f == "run_bytes";
                let cmd = str_arg(0);
                let Value::List(list) = &values[1] else { unreachable!("checked: List[str]") };
                let argv: Vec<String> = list
                    .iter()
                    .map(|v| match v {
                        Value::Str(s) => s.to_string(),
                        other => unreachable!("checked: str, found {other:?}"),
                    })
                    .collect();
                // std.proc.2: optional input and time limit.
                let input: Option<Vec<u8>> = match values.get(2) {
                    Some(Value::Str(s)) => Some(s.as_bytes().to_vec()),
                    Some(Value::Bytes(b)) => Some(b.to_vec()),
                    _ => None,
                };
                let timeout = match values.get(3) {
                    Some(Value::Int(ms)) if *ms < 0 => {
                        return stop("std.proc.2", pos, format!("can't time out after {ms} milliseconds"));
                    }
                    Some(Value::Int(ms)) => Some(*ms as u64),
                    _ => None,
                };
                self.flush(pos)?;
                // conc.cancel.2: a cancelled task doesn't start a process.
                if self.cancelled() {
                    let s = self.structs["proc.ProcError"];
                    let error = Value::Struct(s, Rc::new(vec![Value::Str(cmd), text("cancelled")]));
                    return Ok(fail(error, pos));
                }
                match super::osext::run(&cmd, &argv, input.as_deref(), timeout) {
                    Ok((status, out, err)) if raw => {
                        let s = self.structs["proc.RawOutput"];
                        let fields = vec![
                            Value::Int(exit_status(&status)),
                            super::bytes::bytes_value(out),
                            super::bytes::bytes_value(err),
                        ];
                        Value::Struct(s, Rc::new(fields))
                    }
                    Ok((status, out, err)) => {
                        let s = self.structs["proc.Output"];
                        let fields = vec![
                            Value::Int(exit_status(&status)),
                            text(String::from_utf8_lossy(&out).into_owned()),
                            text(String::from_utf8_lossy(&err).into_owned()),
                        ];
                        Value::Struct(s, Rc::new(fields))
                    }
                    Err(kind) => {
                        let s = self.structs["proc.ProcError"];
                        let error = Value::Struct(s, Rc::new(vec![Value::Str(cmd), text(kind)]));
                        fail(error, pos)
                    }
                }
            }
            ("proc", "start") => {
                let cmd = str_arg(0);
                let Value::List(list) = &values[1] else { unreachable!("checked: List[str]") };
                let argv: Vec<String> = list
                    .iter()
                    .map(|v| match v {
                        Value::Str(s) => s.to_string(),
                        other => unreachable!("checked: str, found {other:?}"),
                    })
                    .collect();
                let timeout = match values.get(2) {
                    Some(Value::Int(ms)) if *ms < 0 => {
                        return stop("std.proc.4", pos, format!("can't time out after {ms} milliseconds"));
                    }
                    Some(Value::Int(ms)) => Some(*ms as u64),
                    _ => None,
                };
                self.flush(pos)?;
                let failed = |this: &Self, kind: &str| {
                    let s = this.structs["proc.ProcError"];
                    fail(Value::Struct(s, Rc::new(vec![Value::Str(cmd.clone()), text(kind)])), pos)
                };
                if self.cancelled() {
                    return Ok(failed(self, "cancelled"));
                }
                match super::spawn::start(&cmd, &argv, timeout) {
                    Ok(c) => {
                        let stdin = self.child_chan(None, Some(c.stdin));
                        let out = self.child_chan(Some(c.out), None);
                        let err = self.child_chan(Some(c.err), None);
                        let s = self.structs["proc.Child"];
                        Value::Struct(s, Rc::new(vec![Value::Int(i64::from(c.pid)), stdin, out, err]))
                    }
                    Err(kind) => failed(self, kind),
                }
            }
            ("proc", "wait") => {
                let Value::Struct(_, fields) = &values[0] else { unreachable!("checked: a Child") };
                let pid = fields[0].int() as u32;
                // std.proc.6: other tasks run while the child does.
                let ended = loop {
                    if let Some(e) = super::spawn::try_wait(pid) {
                        break e;
                    }
                    if self.cancelled() {
                        let e = super::spawn::kill(pid);
                        let cmd = e.map_or_else(String::new, |e| e.cmd);
                        let s = self.structs["proc.ProcError"];
                        return Ok(fail(Value::Struct(s, Rc::new(vec![Value::Str(cmd.into()), text("cancelled")])), pos));
                    }
                    self.yield_turn(pos)?;
                };
                if ended.timed_out {
                    let s = self.structs["proc.ProcError"];
                    return Ok(fail(Value::Struct(s, Rc::new(vec![Value::Str(ended.cmd.into()), text("timed_out")])), pos));
                }
                Value::Int(exit_status(&ended.status))
            }
            ("proc", "kill") => {
                let Value::Struct(_, fields) = &values[0] else { unreachable!("checked: a Child") };
                // Already ended and waited: killing it again is not an error.
                Value::Int(super::spawn::kill(fields[0].int() as u32).map_or(-1, |e| exit_status(&e.status)))
            }
            ("task", "select") => {
                let Value::List(list) = &values[0] else { unreachable!("checked: List[Channel]") };
                let chans: Vec<usize> = list
                    .iter()
                    .map(|v| match v {
                        Value::Handle(super::tasks::Handle::Channel(c)) => *c,
                        other => unreachable!("checked: a channel, found {other:?}"),
                    })
                    .collect();
                return self.select(&chans, pos);
            }
            ("proc", "which") => super::osext::which(&str_arg(0)).map_or(Value::None, text),
            ("net", "serve") => self.net_serve(&str_arg(0), pos),
            ("net", "listen") => self.net_listen(&str_arg(0), pos),
            ("net", "connect" | "connect_tls") => self.net_connect(&str_arg(0), values[1].int(), f == "connect_tls", pos),
            ("net", "options") => {
                let s = self.structs["net.Options"];
                let empty = || Value::Map(Rc::new(Table::default()));
                let fields = vec![
                    empty(),
                    empty(),
                    text(""),
                    Value::Int(super::net::TIMEOUT.as_millis() as i64),
                    Value::Int(super::net::MAX_REDIRECTS as i64),
                    super::bytes::bytes_value(Vec::new()),
                ];
                Value::Struct(s, Rc::new(fields))
            }
            ("net", "get" | "post" | "request" | "get_bytes" | "post_bytes" | "request_bytes") => {
                // std.net.16: the `_bytes` forms give the body undecoded.
                let raw = f.ends_with("_bytes");
                let method = f.trim_end_matches("_bytes");
                let url = str_arg(if method == "request" { 1 } else { 0 });
                super::netsock::set_yielder(self.yielder());
                let result = match method {
                    "get" => super::net::get(&url),
                    "post" => {
                        let body = match &values[1] {
                            Value::Bytes(b) => b.to_vec(),
                            Value::Str(s) => s.as_bytes().to_vec(),
                            other => unreachable!("checked: str or bytes, found {other:?}"),
                        };
                        super::net::post(&url, &body, &str_arg(2))
                    }
                    _ => {
                        let Value::Struct(_, o) = &values[2] else { unreachable!("checked: net.Options") };
                        let Value::Str(body) = &o[2] else { unreachable!("checked: str") };
                        let Value::Bytes(data) = &o[5] else { unreachable!("checked: bytes") };
                        let (timeout_ms, redirects) = (o[3].int(), o[4].int());
                        if timeout_ms <= 0 || redirects < 0 {
                            return stop(
                                "std.net.6",
                                pos,
                                format!("`timeout_ms` is at least 1 and `max_redirects` at least 0, not {timeout_ms} and {redirects}"),
                            );
                        }
                        let opts = super::net::Opts {
                            headers: super::netsock::options_pairs(&o[0]),
                            query: super::netsock::options_pairs(&o[1]),
                            // std.net.16: `data`, when it isn't empty, is the body sent.
                            body: if data.is_empty() { body.as_bytes().to_vec() } else { data.to_vec() },
                            content_type: None,
                            timeout: std::time::Duration::from_millis(timeout_ms as u64),
                            max_redirects: redirects as usize,
                        };
                        super::net::request(&str_arg(0), &url, &opts)
                    }
                };
                match result {
                    Ok(r) => {
                        let mut headers = Table::default();
                        for (k, v) in r.headers {
                            headers.insert(text(k), Some(text(v)));
                        }
                        if raw {
                            let s = self.structs["net.RawResponse"];
                            let fields = vec![Value::Int(r.status), Value::Map(Rc::new(headers)), super::bytes::bytes_value(r.body)];
                            Value::Struct(s, Rc::new(fields))
                        } else {
                            let s = self.structs["net.Response"];
                            let body = text(String::from_utf8_lossy(&r.body).into_owned());
                            Value::Struct(s, Rc::new(vec![Value::Int(r.status), Value::Map(Rc::new(headers)), body]))
                        }
                    }
                    Err(e) => {
                        let s = self.structs["net.NetError"];
                        fail(Value::Struct(s, Rc::new(vec![text(e.url), text(e.kind), text(e.message)])), pos)
                    }
                }
            }
            ("mem", _) => return self.mem_call(f, &values, pos),
            ("regex", _) => return self.regex_call(f, &values, pos),
            ("random", "int") => {
                let (lo, hi) = (values[0].int(), values[1].int());
                if lo > hi {
                    return stop("std.random.2", pos, format!("`int` needs lo <= hi, but lo is {lo} and hi is {hi}"));
                }
                Value::Int(random::range(lo, hi))
            }
            ("random", "float") => Value::Float(random::unit()),
            ("random", "seed") => {
                random::seed(values[0].int());
                Value::Unit
            }
            ("time", "now_ms") => {
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
                Value::Int(now.map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX)))
            }
            ("tz", "offset") => super::tz::offset(&str_arg(0), values[1].int()).map_or(Value::None, Value::Int),
            ("tz", "system") => text(super::tz::system()),
            ("time", "sleep_ms") => {
                let ms = values[0].int();
                if ms < 0 {
                    return stop("std.time.2", pos, format!("can't sleep for {ms} milliseconds"));
                }
                self.flush(pos)?;
                // conc.cancel.2: a cancelled task doesn't sleep.
                if !self.cancelled() {
                    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
                }
                Value::Unit
            }
            ("task", "cancelled") => {
                self.switch_point(pos)?;
                Value::Bool(self.cancelled())
            }
            ("task", "check") => {
                self.switch_point(pos)?;
                if self.cancelled() {
                    let s = self.structs["Cancelled"];
                    return Ok(fail(Value::Struct(s, Rc::new(Vec::new())), pos));
                }
                Value::Unit
            }
            ("ui", "clipboard_get") => crate::ui::clipboard_get().map_or(Value::None, text),
            ("ui", "clipboard_set") => {
                crate::ui::clipboard_set(&str_arg(0));
                Value::Unit
            }
            ("ui", "open") => {
                let (title, cols, rows) = (str_arg(0), values[1].int(), values[2].int());
                self.ui_open(&title, cols, rows, pos)
            }
            _ => unreachable!("checked: `{module}.{f}` is a standard function"),
        })
    }

    /// The failure `fs.IoError(path, kind)` (std.err.1).
    fn io_error(&self, path: &Rc<str>, kind: &str, pos: Pos) -> Value {
        let s = self.structs["fs.IoError"];
        fail(Value::Struct(s, Rc::new(vec![Value::Str(path.clone()), text(kind)])), pos)
    }
}

/// An error value leaving a standard function: its failure, with the call
/// as the first entry of the return trace (err.ctx.1).
pub(super) fn fail(error: Value, pos: Pos) -> Value {
    Value::Fail(Rc::new(Failure { error, context: Vec::new(), trace: vec![pos] }))
}

/// std.fs.1: a directory is `is_dir` on every platform.
fn read_text(path: &str) -> Result<String, &'static str> {
    if std::fs::metadata(path).is_ok_and(|m| m.is_dir()) {
        return Err("is_dir");
    }
    std::fs::read_to_string(path).map_err(|e| error_kind(&e))
}

/// std.fs.5: the names in a directory, sorted by their bytes.
fn list_dir(path: &str) -> Result<Vec<String>, &'static str> {
    let mut names = Vec::new();
    for entry in std::fs::read_dir(path).map_err(|e| error_kind(&e))? {
        let entry = entry.map_err(|e| error_kind(&e))?;
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    names.sort();
    Ok(names)
}

/// The kind of a failure to write `path`, reporting a directory as `is_dir`
/// on every platform (std.err.1).
fn dir_kind(path: &str, e: &std::io::Error) -> &'static str {
    if std::fs::metadata(path).is_ok_and(|m| m.is_dir()) { "is_dir" } else { error_kind(e) }
}

/// std.proc.1: the exit status, or 128 plus the signal that ended the
/// process, as shells report it.
fn exit_status(status: &std::process::ExitStatus) -> i64 {
    if let Some(code) = status.code() {
        return i64::from(code);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(sig) = status.signal() {
            return 128 + i64::from(sig);
        }
    }
    -1
}

fn stop<T>(rule: &'static str, pos: Pos, message: String) -> Flow<T> {
    Err(Exit::Trap(Box::new(Trap { rule, pos, message })))
}
