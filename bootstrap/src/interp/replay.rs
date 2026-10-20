//! Record and replay (replay.rec.1-5, replay.cap.1-2, replay.run.1-3, ADR 0398).
//!
//! `ail run --record R FILE` writes the result of every captured effect
//! (files, environment, standard input, processes, HTTP, the clock, time
//! zones, the clipboard) and the random seed to R, one JSON line each.
//! `ail run --replay R [FILE]` runs the program again with each captured
//! call answered from R instead of the world, so the run, its output and a
//! trap repeat exactly. `ail test --record R` keeps the effects of the tests
//! that failed; `ail test --replay R` runs those tests again from them.

use std::collections::VecDeque;
use std::io::Write as _;
use std::rc::Rc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};

use super::builtins::Table;
use super::{Exit, Failure, Flow, Interp, Trap, Value};
use crate::diag::{Pos, json_str};
use crate::num::Num;

/// Sized number kinds by their index in a recording.
const NUMS: [Num; 8] = [Num::I8, Num::I16, Num::I32, Num::U8, Num::U16, Num::U32, Num::U64, Num::F32];

/// The largest recording, in bytes of effect lines (replay.rec.4).
pub const LIMIT: usize = 64 << 20;

const OFF: u8 = 0;
const RECORD: u8 = 1;
const REPLAY: u8 = 2;
static MODE: AtomicU8 = AtomicU8::new(OFF);

struct State {
    path: String,
    /// The header line's fields.
    program: String,
    args: Vec<String>,
    seed: i64,
    /// Recording: the effect lines so far, and their size.
    lines: Vec<String>,
    bytes: usize,
    truncated: bool,
    /// Tests: the current test, each test's lines (recording) or each
    /// failed test's lines (replay).
    test: Option<String>,
    testing: bool,
    tests: Vec<(String, Vec<String>)>,
    /// Replaying: what is left, and whether the recording was cut.
    queue: VecDeque<J>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn state() -> std::sync::MutexGuard<'static, Option<State>> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

/// Whether a run records or replays (replay.rec.1, replay.run.1).
pub fn on() -> bool {
    MODE.load(Ordering::Relaxed) != OFF
}

/// replay.cap.1: the calls whose results are captured. Writes are captured
/// too: a replay doesn't repeat them. `io.eprint` and `io.exit` are output
/// and run in a replay as usual.
pub fn captured(module: &str, f: &str) -> bool {
    match module {
        "fs" | "env" | "time" | "tz" | "net" => true,
        "io" => !matches!(f, "eprint" | "exit"),
        "proc" => true,
        "ui" => f.starts_with("clipboard"),
        _ => false,
    }
}

/// Start recording into `path` (replay.rec.1). Returns the seed `std.random`
/// starts from: `seed` if given, else one from the clock.
pub fn start_record(path: &str, program: &str, args: &[String], seed: Option<i64>) -> i64 {
    let seed = seed.unwrap_or_else(|| {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| (d.as_nanos() % (1 << 62)) as i64)
    });
    *state() = Some(State {
        path: path.to_string(),
        program: program.to_string(),
        args: args.to_vec(),
        seed,
        lines: Vec::new(),
        bytes: 0,
        truncated: false,
        test: None,
        testing: false,
        tests: Vec::new(),
        queue: VecDeque::new(),
    });
    MODE.store(RECORD, Ordering::Relaxed);
    seed
}

/// What a recording's header says (replay.rec.2).
pub struct Header {
    pub program: String,
    pub args: Vec<String>,
    pub seed: i64,
}

/// Load the recording at `path` and start replaying it (replay.run.1).
pub fn start_replay(path: &str) -> Result<Header, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("can't read the recording {path}: {e}"))?;
    let mut lines = text.lines();
    let head = parse(lines.next().unwrap_or("")).map_err(|e| format!("{path} isn't a recording: {e}"))?;
    if head.int("ail_record") != Some(1) {
        return Err(format!("{path} isn't a recording of this version (no `\"ail_record\":1` header)"));
    }
    let args = match head.get("args") {
        Some(J::Arr(xs)) => xs.iter().filter_map(|x| x.as_str().map(str::to_string)).collect(),
        _ => Vec::new(),
    };
    let mut queue = VecDeque::new();
    let mut tests: Vec<(String, Vec<String>)> = Vec::new();
    let mut truncated = false;
    for (n, line) in lines.enumerate() {
        let j = parse(line).map_err(|e| format!("{path}:{}: {e}", n + 2))?;
        if j.get("truncated").is_some() {
            truncated = true;
        } else if let Some(t) = j.str("test") {
            tests.push((t.to_string(), Vec::new()));
        } else if let Some((_, lines)) = tests.last_mut() {
            lines.push(line.to_string());
        } else {
            queue.push_back(j);
        }
    }
    let header = Header {
        program: head.str("program").unwrap_or_default().to_string(),
        args,
        seed: head.int("seed").unwrap_or(1),
    };
    *state() = Some(State {
        path: path.to_string(),
        program: header.program.clone(),
        args: header.args.clone(),
        seed: header.seed,
        lines: Vec::new(),
        bytes: 0,
        truncated,
        test: None,
        testing: false,
        tests,
        queue,
    });
    MODE.store(REPLAY, Ordering::Relaxed);
    Ok(header)
}

/// A test starts: its effects are kept apart (replay.rec.1), or, in a
/// replay, read from its own section (replay.run.1).
pub fn begin_test(name: &str) {
    let mut st = state();
    let Some(s) = st.as_mut() else { return };
    s.test = Some(name.to_string());
    s.testing = true;
    // replay.rec.5: each test draws from the recording's seed, so a test
    // replayed alone sees the numbers it saw among the others.
    super::random::seed(s.seed);
    match MODE.load(Ordering::Relaxed) {
        RECORD => s.lines.clear(),
        REPLAY => {
            let lines = s.tests.iter().find(|(t, _)| t == name).map(|(_, l)| l.clone()).unwrap_or_default();
            s.queue = lines.iter().filter_map(|l| parse(l).ok()).collect();
        }
        _ => {}
    }
}

/// Whether `ail test` runs this test: a replay runs only the tests its
/// recording holds (replay.run.1).
pub fn wants(name: &str) -> bool {
    MODE.load(Ordering::Relaxed) != REPLAY || state().as_ref().is_some_and(|s| s.tests.iter().any(|(t, _)| t == name))
}

/// A test ended: a failed test's effects are kept for the recording.
pub fn end_test(failed: bool) {
    let mut st = state();
    let Some(s) = st.as_mut() else { return };
    if MODE.load(Ordering::Relaxed) == RECORD
        && let Some(t) = s.test.take()
        && failed
    {
        let lines = std::mem::take(&mut s.lines);
        s.tests.push((t, lines));
    }
}

/// Write the recording (replay.rec.2). Called when the run ends, and by
/// `io.exit` before it leaves.
pub fn finish() {
    if MODE.load(Ordering::Relaxed) != RECORD {
        return;
    }
    let st = state();
    let Some(s) = st.as_ref() else { return };
    let mut out = String::new();
    let args: Vec<String> = s.args.iter().map(|a| json_str(a)).collect();
    out.push_str(&format!(
        "{{\"ail_record\":1,\"program\":{},\"args\":[{}],\"seed\":{},\"limit\":{LIMIT}}}\n",
        json_str(&s.program),
        args.join(","),
        s.seed
    ));
    if s.testing {
        for (t, lines) in &s.tests {
            out.push_str(&format!("{{\"test\":{}}}\n", json_str(t)));
            for l in lines {
                out.push_str(l);
                out.push('\n');
            }
        }
    } else {
        for l in &s.lines {
            out.push_str(l);
            out.push('\n');
        }
    }
    if s.truncated {
        out.push_str("{\"truncated\":true}\n");
    }
    if let Err(e) = std::fs::File::create(&s.path).and_then(|mut f| f.write_all(out.as_bytes())) {
        eprintln!("error: can't write the recording {}: {e}", s.path);
    }
}

impl<'a, W: std::io::Write> Interp<'a, W> {
    /// A captured standard call (replay.cap.1): run and record it, or answer it
    /// from the recording.
    pub(super) fn captured_call(&mut self, module: &str, f: &str, values: Vec<Value>, pos: Pos) -> Flow<Value> {
        let call = format!("{module}.{f}");
        if MODE.load(Ordering::Relaxed) == RECORD {
            let result = self.std_effect(module, f, values, pos);
            let line = match &result {
                // replay.rec.3: a trap in the call repeats in the replay.
                Err(Exit::Trap(t)) => format!(
                    "{{\"call\":{},\"line\":{},\"col\":{},\"trap\":{},\"message\":{}}}",
                    json_str(&call),
                    pos.line,
                    pos.col,
                    json_str(t.rule),
                    json_str(&t.message)
                ),
                Err(_) => return result,
                Ok(v) => match self.encode(v) {
                    Some(v) => format!("{{\"call\":{},\"line\":{},\"col\":{},\"result\":{v}}}", json_str(&call), pos.line, pos.col),
                    // replay.cap.2: a handle (a process, a socket, a window) can't be written down.
                    None => format!("{{\"call\":{},\"line\":{},\"col\":{},\"uncaptured\":true}}", json_str(&call), pos.line, pos.col),
                },
            };
            let mut st = state();
            if let Some(s) = st.as_mut()
                && !s.truncated
            {
                if s.bytes + line.len() > LIMIT {
                    s.truncated = true;
                } else {
                    s.bytes += line.len() + 1;
                    s.lines.push(line);
                }
            }
            return result;
        }
        let next = state().as_mut().and_then(|s| s.queue.pop_front());
        let Some(entry) = next else {
            let cut = state().as_ref().is_some_and(|s| s.truncated);
            let why = if cut {
                format!("the recording was cut at its size limit ({} MB) before this `{call}` (replay.rec.4)", LIMIT >> 20)
            } else {
                format!("the recording has no more effects, but the program calls `{call}` here: the program or its recording changed")
            };
            return trap(pos, why);
        };
        let got = entry.str("call").unwrap_or_default();
        if got != call {
            return trap(
                pos,
                format!(
                    "the program calls `{call}` here, but the recording's next effect is `{got}` (recorded at {}:{}): the program changed since it was recorded",
                    entry.int("line").unwrap_or(0),
                    entry.int("col").unwrap_or(0)
                ),
            );
        }
        if let Some(rule) = entry.str("trap") {
            let rule: &'static str = Box::leak(rule.to_string().into_boxed_str());
            let message = entry.str("message").unwrap_or_default().to_string();
            return Err(Exit::Trap(Box::new(Trap { rule, pos, message })));
        }
        if entry.get("uncaptured").is_some() {
            return Err(Exit::Trap(Box::new(Trap {
                rule: "replay.cap.2",
                pos,
                message: format!("`{call}` returns a handle, which a recording can't hold, so the replay stops here"),
            })));
        }
        match entry.get("result").and_then(|v| self.decode(v)) {
            Some(v) => Ok(v),
            None => trap(pos, format!("the recorded result of `{call}` doesn't fit this program")),
        }
    }

    /// A value as JSON (replay.rec.3), or `None` for one a recording can't hold.
    fn encode(&self, v: &Value) -> Option<String> {
        let list = |xs: &mut dyn Iterator<Item = &Value>| -> Option<String> {
            let parts: Option<Vec<String>> = xs.map(|x| self.encode(x)).collect();
            Some(format!("[{}]", parts?.join(",")))
        };
        Some(match v {
            Value::Unit => "null".into(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => format!("{{\"f\":\"{f:?}\"}}"),
            Value::Num(k, bits) => format!("{{\"num\":{},\"bits\":{bits}}}", NUMS.iter().position(|n| n == k)?),
            Value::Bool(b) => b.to_string(),
            Value::Str(s) => json_str(s),
            Value::Bytes(b) => format!("{{\"bytes\":\"{}\"}}", b.iter().map(|x| format!("{x:02x}")).collect::<String>()),
            Value::Char(c) => format!("{{\"char\":{}}}", json_str(&c.to_string())),
            Value::Struct(s, fields) => {
                format!("{{\"struct\":{},\"fields\":{}}}", json_str(&self.prog.structs[*s].name), list(&mut fields.iter())?)
            }
            Value::Enum(e, var, fields) => format!(
                "{{\"enum\":{},\"variant\":{var},\"fields\":{}}}",
                json_str(&self.prog.enums[*e].name),
                list(&mut fields.iter())?
            ),
            Value::List(xs) => list(&mut xs.iter())?,
            Value::Map(t) => {
                let parts: Option<Vec<String>> =
                    t.entries().map(|(k, v)| Some(format!("[{},{}]", self.encode(k)?, self.encode(v)?))).collect();
                format!("{{\"map\":[{}]}}", parts?.join(","))
            }
            Value::Set(t) => format!("{{\"set\":{}}}", list(&mut t.keys())?),
            Value::None => "{\"none\":true}".into(),
            Value::Nested(n) => format!("{{\"nested\":{n}}}"),
            Value::Fail(f) => {
                let ctx: Vec<String> = f.context.iter().map(|c| json_str(c)).collect();
                let at: Vec<String> = f.trace.iter().map(|p| format!("[{},{}]", p.line, p.col)).collect();
                format!(
                    "{{\"fail\":{},\"context\":[{}],\"trace\":[{}]}}",
                    self.encode(&f.error)?,
                    ctx.join(","),
                    at.join(",")
                )
            }
            // Function values have no text form (type.fn.1), so a recording can't hold them.
            Value::Ref(_) | Value::Handle(_) | Value::Tensor(_) | Value::Atomic(_) | Value::Fn(_) => return None,
        })
    }

    /// A recorded value back (replay.run.2); `None` when it names a type
    /// this program doesn't have.
    fn decode(&self, j: &J) -> Option<Value> {
        let all = |j: &J| -> Option<Vec<Value>> {
            match j {
                J::Arr(xs) => xs.iter().map(|x| self.decode(x)).collect(),
                _ => None,
            }
        };
        Some(match j {
            J::Null => Value::Unit,
            J::Bool(b) => Value::Bool(*b),
            J::Int(i) => Value::Int(*i),
            J::Num(_) => return None,
            J::Str(s) => Value::Str(Rc::from(s.as_str())),
            J::Arr(_) => Value::List(Rc::new(all(j)?)),
            J::Obj(_) => {
                if let Some(f) = j.str("f") {
                    Value::Float(f.parse().ok()?)
                } else if let Some(k) = j.int("num") {
                    Value::Num(*NUMS.get(usize::try_from(k).ok()?)?, j.int("bits")?)
                } else if let Some(h) = j.str("bytes") {
                    let b: Option<Vec<u8>> =
                        (0..h.len()).step_by(2).map(|i| u8::from_str_radix(h.get(i..i + 2)?, 16).ok()).collect();
                    Value::Bytes(Rc::new(b?))
                } else if let Some(c) = j.str("char") {
                    Value::Char(c.chars().next()?)
                } else if let Some(name) = j.str("struct") {
                    let s = self.prog.structs.iter().position(|s| s.name == name)?;
                    Value::Struct(s, Rc::new(all(j.get("fields")?)?))
                } else if let Some(name) = j.str("enum") {
                    let e = self.prog.enums.iter().position(|e| e.name == name)?;
                    Value::Enum(e, j.int("variant")? as usize, Rc::new(all(j.get("fields")?)?))
                } else if let Some(J::Arr(pairs)) = j.get("map") {
                    let mut t = Table::default();
                    for p in pairs {
                        let kv = all(p)?;
                        let [k, v] = <[Value; 2]>::try_from(kv).ok()?;
                        t.insert(k, Some(v));
                    }
                    Value::Map(Rc::new(t))
                } else if let Some(xs) = j.get("set") {
                    let mut t = Table::default();
                    for k in all(xs)? {
                        t.insert(k, None);
                    }
                    Value::Set(Rc::new(t))
                } else if j.get("none").is_some() {
                    Value::None
                } else if let Some(n) = j.int("nested") {
                    Value::Nested(n as u32)
                } else if let Some(e) = j.get("fail") {
                    let context = match j.get("context") {
                        Some(J::Arr(xs)) => xs.iter().filter_map(|x| x.as_str().map(str::to_string)).collect(),
                        _ => Vec::new(),
                    };
                    let trace = match j.get("trace") {
                        Some(J::Arr(xs)) => xs
                            .iter()
                            .filter_map(|p| match p {
                                J::Arr(lc) => match lc.as_slice() {
                                    [J::Int(l), J::Int(c)] => Some(Pos { line: *l as usize, col: *c as usize }),
                                    _ => None,
                                },
                                _ => None,
                            })
                            .collect(),
                        _ => Vec::new(),
                    };
                    Value::Fail(Rc::new(Failure { error: self.decode(e)?, context, trace }))
                } else {
                    return None;
                }
            }
        })
    }
}

fn trap<T>(pos: Pos, message: String) -> Flow<T> {
    Err(Exit::Trap(Box::new(Trap { rule: "replay.run.3", pos, message })))
}

/// A JSON value read back from a recording.
#[derive(Debug, Clone, PartialEq)]
pub enum J {
    Null,
    Bool(bool),
    Int(i64),
    Num(f64),
    Str(String),
    Arr(Vec<J>),
    Obj(Vec<(String, J)>),
}

impl J {
    fn get(&self, key: &str) -> Option<&J> {
        match self {
            J::Obj(kv) => kv.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    fn str(&self, key: &str) -> Option<&str> {
        self.get(key)?.as_str()
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            J::Str(s) => Some(s),
            _ => None,
        }
    }

    fn int(&self, key: &str) -> Option<i64> {
        match self.get(key)? {
            J::Int(i) => Some(*i),
            _ => None,
        }
    }
}

/// Read one line of JSON, as the recording writes it.
pub fn parse(s: &str) -> Result<J, String> {
    let b = s.as_bytes();
    let mut i = 0;
    let v = value(s, b, &mut i)?;
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    if i != b.len() {
        return Err(format!("unexpected text at byte {i}"));
    }
    Ok(v)
}

fn value(s: &str, b: &[u8], i: &mut usize) -> Result<J, String> {
    while *i < b.len() && b[*i].is_ascii_whitespace() {
        *i += 1;
    }
    let at = *i;
    let close = |b: &[u8], i: &mut usize, c: u8| -> Result<bool, String> {
        while *i < b.len() && b[*i].is_ascii_whitespace() {
            *i += 1;
        }
        match b.get(*i) {
            Some(&x) if x == c => {
                *i += 1;
                Ok(true)
            }
            Some(b',') => {
                *i += 1;
                Ok(false)
            }
            _ => Err(format!("expected `,` or `{}` at byte {}", c as char, *i)),
        }
    };
    match b.get(at) {
        None => Err("the line ends early".into()),
        Some(b'{') => {
            *i += 1;
            let mut kv = Vec::new();
            if close(b, i, b'}').unwrap_or(false) {
                return Ok(J::Obj(kv));
            }
            loop {
                let J::Str(k) = value(s, b, i)? else { return Err(format!("expected a key at byte {}", *i)) };
                while *i < b.len() && b[*i].is_ascii_whitespace() {
                    *i += 1;
                }
                if b.get(*i) != Some(&b':') {
                    return Err(format!("expected `:` at byte {}", *i));
                }
                *i += 1;
                kv.push((k, value(s, b, i)?));
                if close(b, i, b'}')? {
                    return Ok(J::Obj(kv));
                }
            }
        }
        Some(b'[') => {
            *i += 1;
            let mut xs = Vec::new();
            let save = *i;
            if close(b, i, b']').unwrap_or(false) {
                return Ok(J::Arr(xs));
            }
            *i = save;
            loop {
                xs.push(value(s, b, i)?);
                if close(b, i, b']')? {
                    return Ok(J::Arr(xs));
                }
            }
        }
        Some(b'"') => {
            *i += 1;
            let mut out = String::new();
            loop {
                let start = *i;
                while *i < b.len() && b[*i] != b'"' && b[*i] != b'\\' {
                    *i += 1;
                }
                out.push_str(&s[start..*i]);
                match b.get(*i) {
                    Some(b'"') => {
                        *i += 1;
                        return Ok(J::Str(out));
                    }
                    Some(b'\\') => {
                        let e = *b.get(*i + 1).ok_or("the line ends inside a string")?;
                        *i += 2;
                        match e {
                            b'n' => out.push('\n'),
                            b't' => out.push('\t'),
                            b'r' => out.push('\r'),
                            b'b' => out.push('\u{8}'),
                            b'f' => out.push('\u{c}'),
                            b'u' => {
                                let hex = s.get(*i..*i + 4).ok_or("bad \\u escape")?;
                                *i += 4;
                                let mut n = u32::from_str_radix(hex, 16).map_err(|_| "bad \\u escape")?;
                                // A surrogate pair.
                                if (0xD800..0xDC00).contains(&n) && s.get(*i..*i + 2) == Some("\\u") {
                                    let lo = u32::from_str_radix(s.get(*i + 2..*i + 6).unwrap_or("0"), 16).unwrap_or(0);
                                    *i += 6;
                                    n = 0x10000 + ((n - 0xD800) << 10) + (lo.wrapping_sub(0xDC00) & 0x3FF);
                                }
                                out.push(char::from_u32(n).unwrap_or('\u{fffd}'));
                            }
                            c => out.push(c as char),
                        }
                    }
                    _ => return Err("the line ends inside a string".into()),
                }
            }
        }
        Some(c) if *c == b'-' || c.is_ascii_digit() => {
            while *i < b.len() && (b[*i].is_ascii_digit() || b"+-.eE".contains(&b[*i])) {
                *i += 1;
            }
            let t = &s[at..*i];
            match t.parse::<i64>() {
                Ok(n) => Ok(J::Int(n)),
                Err(_) => t.parse().map(J::Num).map_err(|_| format!("bad number `{t}`")),
            }
        }
        _ => {
            for (word, v) in [("true", J::Bool(true)), ("false", J::Bool(false)), ("null", J::Null)] {
                if s[at..].starts_with(word) {
                    *i += word.len();
                    return Ok(v);
                }
            }
            Err(format!("unexpected text at byte {at}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_recordings() {
        let j = parse(r#"{"call":"fs.read_text","result":{"f":"1.5"},"xs":[1,-2,"a\né😀"],"n":null}"#).unwrap();
        assert_eq!(j.str("call"), Some("fs.read_text"));
        assert_eq!(j.get("xs"), Some(&J::Arr(vec![J::Int(1), J::Int(-2), J::Str("a\né😀".into())])));
        assert_eq!(j.get("n"), Some(&J::Null));
        assert!(parse("{\"a\":").is_err());
    }

    #[test]
    fn captured_calls() {
        assert!(captured("fs", "write_text"));
        assert!(captured("io", "read_line"));
        assert!(!captured("io", "eprint"));
        assert!(!captured("random", "int"));
        assert!(!captured("path", "join"));
    }
}
