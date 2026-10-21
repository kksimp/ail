//! Runtime traces, watches and trap snapshots for agents (spec 21-trace,
//! ADRs 0265-0268): `ail run --trace[=calls|effects|all]` (trace.run.1-4),
//! `--watch NAME` (trace.watch.1-2) and the snapshot a trap carries
//! (trace.snap.1-4). Off, the interpreter tests one byte per call and per
//! assignment (trace.run.5). Values are shown in their structural text form
//! without running user code (trace.val.1), cut to a budget (trace.val.2).

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering as AtomicOrdering};
use std::sync::{Mutex, OnceLock};

use super::*;
use crate::ast::{SourceFile, is_prelude};
use crate::diag::json_str;

/// `self.tracing` bits.
pub(super) const CALLS: u8 = 1;
pub(super) const EFFECTS: u8 = 2;
pub(super) const WATCH: u8 = 4;

/// What `ail run` traces; read once per interpreter by `build`.
static MODE: AtomicU8 = AtomicU8::new(0);

struct Config {
    limit: u64,
    json: bool,
    watch: String,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

/// Events shown and dropped, by kind (trace.run.4). Tasks run one at a
/// time (conc.task.2), so the lock is never contended.
static STATE: Mutex<State> = Mutex::new(State { shown: 0, dropped: 0, kinds: [0; KINDS.len()] });

struct State {
    shown: u64,
    dropped: u64,
    kinds: [u64; KINDS.len()],
}

const KINDS: [&str; 7] = ["call", "return", "effect", "spawn", "wait", "contract", "watch"];

/// The default event budget (trace.run.4).
const DEFAULT_LIMIT: u64 = 1000;
/// A value in a trace line or a default trap report (trace.val.2).
const SHORT: usize = 60;
/// A value in a snapshot as captured, and in `--verbose` (trace.val.2).
const LONG: usize = 1000;
/// Frames a snapshot keeps, innermost first (trace.snap.2).
const FRAME_LIMIT: usize = 64;
/// Lines of a default human trap report (trace.snap.3).
const REPORT_LINES: usize = 10;

/// One function being run while `--watch` is on: its name, and the names
/// in it that stand for the watched variable (trace.watch.3).
struct WatchFrame {
    name: String,
    aliases: Vec<String>,
}

/// Events kept for a test (trace.test.1).
struct Capture {
    lines: Vec<String>,
    dropped: u64,
}

thread_local! {
    static FRAMES: std::cell::RefCell<Vec<WatchFrame>> = const { std::cell::RefCell::new(Vec::new()) };
    /// Aliases found at a call site, until the callee's body starts (trace.watch.3).
    static PENDING: std::cell::RefCell<Vec<(usize, Vec<String>)>> = const { std::cell::RefCell::new(Vec::new()) };
    static CAPTURE: std::cell::RefCell<Option<Capture>> = const { std::cell::RefCell::new(None) };
    /// The snapshot of the trap that ended `run`, for `main.rs` to render.
    static LAST: std::cell::RefCell<Option<Snapshot>> = const { std::cell::RefCell::new(None) };
}

/// `--verbose`: the human trap report shows every frame and value (trace.snap.3).
static VERBOSE: AtomicBool = AtomicBool::new(false);

/// The options `ail run` takes for tracing (trace.run.1, trace.run.4,
/// trace.watch.1, trace.snap.3), removed from `args`. Returns whether any
/// was given, so other commands can reject them.
pub fn take_options(args: &mut Vec<&str>, json: bool) -> Result<bool, String> {
    let mut mode = 0;
    let mut limit = DEFAULT_LIMIT;
    let mut watch = String::new();
    let mut given = false;
    let mut i = 0;
    while i < args.len() {
        let a = args[i];
        let take = match a {
            "--trace" | "--trace=all" => {
                mode |= CALLS | EFFECTS;
                1
            }
            "--trace=calls" => {
                mode |= CALLS;
                1
            }
            "--trace=effects" => {
                mode |= EFFECTS;
                1
            }
            "--verbose" => {
                VERBOSE.store(true, AtomicOrdering::Relaxed);
                1
            }
            "--trace-limit" => {
                let n = args.get(i + 1).and_then(|s| s.parse::<u64>().ok());
                limit = n.ok_or("`--trace-limit` needs a non-negative number of events, e.g. `--trace-limit 200` (trace.run.4)")?;
                2
            }
            "--watch" => {
                let name = args.get(i + 1).filter(|s| !s.starts_with('-'));
                watch = name.ok_or("`--watch` needs a variable name, e.g. `--watch total` (trace.watch.1)")?.to_string();
                mode |= WATCH;
                2
            }
            t if t.starts_with("--trace=") => {
                return Err(format!("`{t}`: trace `calls`, `effects` or `all` (trace.run.1)"));
            }
            _ => 0,
        };
        if take == 0 {
            i += 1;
        } else {
            given = true;
            args.drain(i..i + take);
        }
    }
    if mode != 0 {
        MODE.store(mode, AtomicOrdering::Relaxed);
        let _ = CONFIG.set(Config { limit, json, watch });
    }
    Ok(given)
}

/// Whether `arg` is one of `take_options`' options, and how many words it takes.
pub fn option_words(arg: &str) -> Option<usize> {
    match arg {
        "--trace-limit" | "--watch" => Some(2),
        "--verbose" => Some(1),
        a if a == "--trace" || a.starts_with("--trace=") => Some(1),
        _ => None,
    }
}

pub(super) fn mode() -> u8 {
    MODE.load(AtomicOrdering::Relaxed)
}

/// Write one event, or count it as dropped past the budget (trace.run.4).
/// `line` builds the text only when it is shown.
fn emit(kind: usize, line: impl FnOnce(bool) -> String) {
    let Some(cfg) = CONFIG.get() else { return };
    // trace.test.1: inside a test the events are kept, not written.
    let mut line = Some(line);
    let kept = CAPTURE.with(|c| {
        let mut c = c.borrow_mut();
        let cap = c.as_mut()?;
        if (cap.lines.len() as u64) < cfg.limit {
            cap.lines.push(line.take()?(cfg.json));
        } else {
            cap.dropped += 1;
        }
        Some(())
    });
    if kept.is_some() {
        return;
    }
    let Some(line) = line else { return };
    let mut st = STATE.lock().unwrap_or_else(|e| e.into_inner());
    st.kinds[kind] += 1;
    if st.shown >= cfg.limit {
        st.dropped += 1;
        return;
    }
    st.shown += 1;
    eprintln!("{}", line(cfg.json));
}

/// The final line of a traced run (trace.run.4): events shown, dropped,
/// and counted by kind.
pub(super) fn finish() {
    let Some(cfg) = CONFIG.get() else { return };
    let st = STATE.lock().unwrap_or_else(|e| e.into_inner());
    let kinds: Vec<(&str, u64)> = KINDS.iter().copied().zip(st.kinds).filter(|(_, n)| *n > 0).collect();
    if cfg.json {
        let mut s = format!("{{\"api\":1,\"trace\":\"summary\",\"shown\":{},\"dropped\":{}", st.shown, st.dropped);
        for (k, n) in kinds {
            let _ = write!(s, ",\"{k}\":{n}");
        }
        eprintln!("{s}}}");
    } else {
        let counts: Vec<String> = kinds.iter().map(|(k, n)| format!("{k} {n}")).collect();
        let list = if counts.is_empty() { "none".into() } else { counts.join(", ") };
        eprintln!("trace: summary: {} events shown, {} dropped ({list})", st.shown, st.dropped);
    }
}

/// `FILE:LINE:COL` parts of `pos` (mod.unit.5: a package has several files).
fn place_of(prog: &Program, pos: Pos) -> (String, usize, usize) {
    match prog.file_at(pos) {
        Some(f) => (f.path.clone(), pos.line - f.base, pos.col),
        None if is_prelude(pos) => ("<prelude>".into(), pos.line, pos.col),
        None => (super::stdlib::program_path(), pos.line, pos.col),
    }
}

/// Whether `pos` is in the program's first file, which trace lines leave
/// unnamed (trace.run.2).
fn in_root(prog: &Program, pos: Pos) -> bool {
    !is_prelude(pos) && prog.file_at(pos).is_none_or(|f| f.base == 0)
}

/// `LINE:COL`, or `FILE:LINE:COL` outside the first file.
fn at_text(prog: &Program, pos: Pos) -> String {
    let (f, l, c) = place_of(prog, pos);
    if in_root(prog, pos) { format!("{l}:{c}") } else { format!("{f}:{l}:{c}") }
}

/// `"line":L,"col":C`, after `"file":F` outside the first file.
fn at_json(prog: &Program, pos: Pos) -> String {
    let (f, l, c) = place_of(prog, pos);
    if in_root(prog, pos) {
        format!("\"line\":{l},\"col\":{c}")
    } else {
        format!("\"file\":{},\"line\":{l},\"col\":{c}", json_str(&f))
    }
}

fn indent(depth: usize) -> String {
    "  ".repeat(depth.saturating_sub(1).min(20))
}

/// `name = value, ...` or a JSON object of the same.
fn pairs(pairs: &[(String, String)], json: bool) -> String {
    if json {
        let parts: Vec<String> = pairs.iter().map(|(n, v)| format!("{}:{}", json_str(n), json_str(v))).collect();
        format!("{{{}}}", parts.join(","))
    } else {
        pairs.iter().map(|(n, v)| format!("{n} = {v}")).collect::<Vec<_>>().join(", ")
    }
}

/// trace.val.1: the structural text form of `v` (trait.auto.2) without
/// running any `to_str`, so showing a value has no effects and can't trap.
/// trace.val.2: cut at about `budget` characters, ending in `…`.
pub(super) fn show(v: &Value, budget: usize) -> String {
    let mut s = String::new();
    write_value(&mut s, v, budget);
    if s.len() > budget {
        let mut end = budget;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        s.truncate(end);
        s.push('…');
    }
    s
}

fn write_value(s: &mut String, v: &Value, budget: usize) {
    if s.len() > budget {
        return;
    }
    let fields = |s: &mut String, names: &[String], values: &[Value]| {
        if values.is_empty() {
            return;
        }
        s.push('(');
        for (i, (n, x)) in names.iter().zip(values).enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            let _ = write!(s, "{n}: ");
            write_value(s, x, budget);
        }
        s.push(')');
    };
    match v {
        Value::Unit => s.push_str("()"),
        Value::Str(_) | Value::Bytes(_) | Value::Char(_) | Value::Int(_) | Value::Num(..) | Value::Bool(_) | Value::None => {
            s.push_str(&key_text(v))
        }
        Value::Nested(_) => s.push_str(&key_text(v)),
        Value::Float(x) => s.push_str(&float_text(*x)),
        // A `ref` cell shows its current value (own.ref.1); the cell may be
        // borrowed by the write in progress, so don't block on it.
        Value::Ref(c) => match c.try_borrow() {
            Ok(inner) => {
                s.push_str("ref ");
                write_value(s, &inner, budget);
            }
            Err(_) => s.push_str("ref …"),
        },
        Value::Tuple(items) => {
            s.push('(');
            for (i, x) in items.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }
                write_value(s, x, budget);
            }
            s.push(')');
        }
        Value::List(items) => {
            s.push('[');
            for (i, x) in items.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }
                if s.len() > budget {
                    let _ = write!(s, "… {} more", items.len() - i);
                    break;
                }
                write_value(s, &x, budget);
            }
            s.push(']');
        }
        Value::Struct(st, values) => NAMES.with(|n| {
            let n = n.borrow();
            let (name, names) = &n.structs[*st];
            s.push_str(name);
            // std.proc.4: a process id differs from run to run, so a child
            // shows as its name only and traces replay (trace.run.1).
            if name != "proc.Child" {
                fields(s, names, values);
            }
        }),
        Value::Enum(e, i, values) => NAMES.with(|n| {
            let n = n.borrow();
            let (enum_name, variants) = &n.enums[*e];
            let (name, names) = &variants[*i];
            let _ = write!(s, "{enum_name}.{name}");
            fields(s, names, values);
        }),
        Value::Map(t) | Value::Set(t) => {
            s.push('{');
            let is_map = matches!(v, Value::Map(_));
            // A set has no values, so `entries` would pair nothing.
            let rows: Vec<(&Value, &Value)> =
                if is_map { t.entries().collect() } else { t.keys().map(|k| (k, k)).collect() };
            for (i, (k, x)) in rows.into_iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }
                if s.len() > budget {
                    let _ = write!(s, "… {} more", t.len() - i);
                    break;
                }
                write_value(s, k, budget);
                if is_map {
                    s.push_str(": ");
                    write_value(s, x, budget);
                }
            }
            s.push('}');
        }
        Value::Fail(f) => {
            s.push_str("error ");
            write_value(s, &f.error, budget);
        }
        Value::Handle(h) => s.push_str(&h.text()),
        Value::Tensor(t) => s.push_str(&format!("tensor(shape: {:?}, {} elements)", t.shape, t.len())),
        Value::Atomic(c) => s.push_str(&c.text()),
        Value::Fn(f) => s.push_str(&format!("fn#{f}")),
    }
}

/// The locals of a frame, each name once (the innermost binding), shown.
fn locals(env: &Env<'_>, budget: usize) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::with_capacity(env.len());
    for (name, v) in env.iter().rev() {
        if !out.iter().any(|(n, _)| n == name) {
            out.push((name.to_string(), show(v, budget)));
        }
    }
    out.reverse();
    out
}

/// What a trap's snapshot holds while the stack unwinds (trace.snap.1-2).
#[derive(Default)]
pub(super) struct Snap {
    frames: Vec<Frame>,
    omitted: usize,
    /// The innermost block's locals, taken before the blocks drop them.
    pending: Option<Vec<(String, String)>>,
    condition: Vec<(String, String)>,
}

struct Frame {
    func: String,
    /// Where this function was called; the next frame out is there.
    called_at: Pos,
    /// How many of `locals` are the receiver and parameters.
    params: usize,
    locals: Vec<(String, String)>,
}

/// A trap's snapshot, ready to render (trace.snap.1).
pub struct Snapshot {
    /// Each frame's function, position (`FILE`, line, column, and as
    /// `at_text` writes it), parameters and other locals, innermost first.
    frames: Vec<(String, (String, usize, usize, String), Vec<(String, String)>, Vec<(String, String)>)>,
    omitted: usize,
    condition: Vec<(String, String)>,
    source: Option<String>,
}

/// The snapshot of the trap that ended the last `run` on this thread.
pub fn take_snapshot() -> Option<Snapshot> {
    LAST.with(|l| l.borrow_mut().take())
}

/// Turn the captured frames into a snapshot of `trap` (trace.snap.1).
pub(super) fn keep(prog: &Program, snap: Snap, trap: &Trap) {
    let mut at = trap.pos;
    let mut frames = Vec::with_capacity(snap.frames.len());
    for mut f in snap.frames {
        let rest = f.locals.split_off(f.params.min(f.locals.len()));
        let (file, line, col) = place_of(prog, at);
        frames.push((f.func, (file, line, col, at_text(prog, at)), f.locals, rest));
        at = f.called_at;
    }
    let source = source_line(prog, trap.pos);
    LAST.with(|l| *l.borrow_mut() = Some(Snapshot { frames, omitted: snap.omitted, condition: snap.condition, source }));
}

/// The text of the line `pos` is on (trace.snap.1).
fn source_line(prog: &Program, pos: Pos) -> Option<String> {
    if is_prelude(pos) {
        return None;
    }
    let (line, src) = match prog.file_at(pos) {
        Some(SourceFile { base, src, .. }) => (pos.line - base, src.clone()),
        None => (pos.line, std::fs::read_to_string(super::stdlib::program_path()).ok()?),
    };
    src.lines().nth(line.checked_sub(1)?).map(str::to_string)
}

/// A trap as one JSON object (trace.snap.4), or as the short human report
/// (trace.snap.3): the trap line, the source line, the contract's values and
/// as many frames as fit in ten lines; `--verbose` shows them all.
pub fn render_trap(prog: &Program, trap: &Trap, snap: Option<Snapshot>, json: bool) -> String {
    let (file, line, col) = place_of(prog, trap.pos);
    let snap = snap.unwrap_or(Snapshot { frames: Vec::new(), omitted: 0, condition: Vec::new(), source: None });
    if json {
        let mut s = format!("{{\"api\":{},\"severity\":\"trap\"", crate::diag::API_VERSION);
        let _ = write!(s, ",\"rule\":{},\"file\":{},\"line\":{line},\"col\":{col}", json_str(trap.rule), json_str(&file));
        let _ = write!(s, ",\"message\":{}", json_str(&trap.message));
        if let Some(src) = &snap.source {
            let _ = write!(s, ",\"source\":{}", json_str(src));
        }
        if !snap.condition.is_empty() {
            let _ = write!(s, ",\"condition\":{}", pairs(&snap.condition, true));
        }
        s.push_str(",\"stack\":[");
        for (i, (func, (f, l, c, _), params, rest)) in snap.frames.iter().enumerate() {
            let sep = if i > 0 { "," } else { "" };
            let _ = write!(
                s,
                "{sep}{{\"fn\":{},\"file\":{},\"line\":{l},\"col\":{c},\"args\":{},\"locals\":{}}}",
                json_str(func),
                json_str(f),
                pairs(params, true),
                pairs(rest, true)
            );
        }
        let _ = write!(s, "],\"omitted_frames\":{}}}", snap.omitted);
        return s;
    }
    let verbose = VERBOSE.load(AtomicOrdering::Relaxed);
    let cut = |pairs: &[(String, String)]| -> Vec<(String, String)> {
        if verbose {
            return pairs.to_vec();
        }
        pairs.iter().map(|(n, v)| (n.clone(), cut_text(v, SHORT))).collect()
    };
    let mut lines = vec![format!("{file}:{line}:{col}: trap[{}]: {}", trap.rule, trap.message)];
    if let Some(src) = &snap.source {
        let gutter = line.to_string();
        lines.push(format!("  {gutter} | {src}"));
        let pad: String = src.chars().take(col.saturating_sub(1)).map(|c| if c == '\t' { '\t' } else { ' ' }).collect();
        lines.push(format!("  {} | {pad}^", " ".repeat(gutter.len())));
    }
    if !snap.condition.is_empty() {
        lines.push(format!("  values: {}", pairs(&cut(&snap.condition), false)));
    }
    let total = snap.frames.len() + snap.omitted;
    for (i, (func, (.., at), params, rest)) in snap.frames.iter().enumerate() {
        if !verbose && lines.len() + 1 >= REPORT_LINES && i + 1 < snap.frames.len() {
            lines.push(format!("  … {} more frames (--verbose shows every frame and value)", total - i));
            return lines.join("\n");
        }
        let head = format!("  at {at} in {func}({})", pairs(&cut(params), false));
        if verbose {
            lines.push(head);
            for (n, v) in rest {
                lines.push(format!("      {n} = {v}"));
            }
        } else if rest.is_empty() {
            lines.push(cut_text(&head, 160));
        } else {
            lines.push(cut_text(&format!("{head}; {}", pairs(&cut(rest), false)), 160));
        }
    }
    if snap.omitted > 0 {
        lines.push(format!("  … {} more frames not captured", snap.omitted));
    }
    lines.join("\n")
}

fn cut_text(s: &str, budget: usize) -> String {
    if s.chars().count() <= budget {
        return s.to_string();
    }
    let mut out: String = s.chars().take(budget).collect();
    out.push('…');
    out
}

/// The names a contract condition reads (trace.snap.2).
fn names_in<'e>(e: &'e Expr, out: &mut Vec<&'e str>) {
    match e {
        Expr::Name(n, _) => {
            if !out.contains(&n.as_str()) {
                out.push(n);
            }
        }
        Expr::Binary { lhs, rhs, .. } => {
            names_in(lhs, out);
            names_in(rhs, out);
        }
        Expr::Unary { operand, .. } => names_in(operand, out),
        Expr::Call { args, .. } => args.iter().for_each(|a| names_in(&a.value, out)),
        Expr::Method { recv, args, .. } => {
            names_in(recv, out);
            args.iter().for_each(|a| names_in(&a.value, out));
        }
        Expr::Field { recv, .. } => names_in(recv, out),
        Expr::Index { recv, index, .. } => {
            names_in(recv, out);
            names_in(index, out);
        }
        Expr::List(items, _) => items.iter().for_each(|x| names_in(x, out)),
        Expr::Interp(parts, _) => {
            for p in parts {
                if let InterpPart::Expr(x) | InterpPart::Fmt(x, _) = p {
                    names_in(x, out);
                }
            }
        }
        _ => {}
    }
}

impl<'a, W: Write> Interp<'a, W> {
    /// trace.run.2: `f` is entered with these arguments, called at `pos`.
    #[inline(never)]
    pub(super) fn trace_call(&self, f: &FnDecl, env: &Env<'_>, pos: Pos) {
        if is_prelude(f.pos) {
            return;
        }
        let depth = self.depth;
        let prog = self.prog;
        emit(0, |json| {
            let args = locals(env, SHORT);
            if json {
                format!(
                    "{{\"api\":1,\"trace\":\"call\",\"fn\":{},\"depth\":{depth},\"args\":{},{}}}",
                    json_str(&f.full_name()),
                    pairs(&args, true),
                    at_json(prog, pos)
                )
            } else {
                let indent = indent(depth);
                format!("trace: {indent}call {}({}) at {}", f.full_name(), pairs(&args, false), at_text(prog, pos))
            }
        });
    }

    /// trace.run.2: `f` returned `result`.
    #[inline(never)]
    pub(super) fn trace_return(&self, f: &FnDecl, result: &Value) {
        if is_prelude(f.pos) {
            return;
        }
        let depth = self.depth;
        emit(1, |json| {
            let value = show(result, SHORT);
            if json {
                format!(
                    "{{\"api\":1,\"trace\":\"return\",\"fn\":{},\"depth\":{depth},\"value\":{}}}",
                    json_str(&f.full_name()),
                    json_str(&value)
                )
            } else {
                let indent = indent(depth);
                format!("trace: {indent}{} returned {value}", f.full_name())
            }
        });
    }

    /// trace.run.3: a standard-module call reached an effect.
    #[inline(never)]
    pub(super) fn trace_effect(&self, module: &str, f: &str, values: &[Value], pos: Pos) {
        let prog = self.prog;
        emit(2, |json| {
            let args: Vec<String> = values.iter().map(|v| show(v, SHORT)).collect();
            if json {
                let list: Vec<String> = args.iter().map(|a| json_str(a)).collect();
                format!(
                    "{{\"api\":1,\"trace\":\"effect\",\"call\":\"{module}.{f}\",\"args\":[{}],{}}}",
                    list.join(","),
                    at_json(prog, pos)
                )
            } else {
                format!("trace: effect {module}.{f}({}) at {}", args.join(", "), at_text(prog, pos))
            }
        });
    }

    /// trace.run.3: a task is spawned (conc.task.1) or waited.
    #[inline(never)]
    pub(super) fn trace_task(&self, spawn: bool, name: &str, args: &[Value], pos: Pos) {
        let prog = self.prog;
        let kind = if spawn { "spawn" } else { "wait" };
        emit(if spawn { 3 } else { 4 }, |json| {
            let what = if spawn {
                format!("{name}({})", args.iter().map(|v| show(v, SHORT)).collect::<Vec<_>>().join(", "))
            } else {
                name.to_string()
            };
            let what = what.as_str();
            if json {
                format!("{{\"api\":1,\"trace\":\"{kind}\",\"task\":{},{}}}", json_str(what), at_json(prog, pos))
            } else {
                format!("trace: {kind} {what} at {}", at_text(prog, pos))
            }
        });
    }

    /// trace.run.3: a struct invariant was checked (contract.inv.2-3).
    #[inline(never)]
    pub(super) fn trace_invariant(&self, name: &str, c: &crate::ast::Contract, held: bool) {
        let prog = self.prog;
        let depth = self.depth;
        emit(5, |json| {
            if json {
                format!(
                    "{{\"api\":1,\"trace\":\"contract\",\"fn\":{},\"kind\":\"invariant\",\"cond\":{},\"held\":{held},{}}}",
                    json_str(name),
                    json_str(&c.text),
                    at_json(prog, c.pos)
                )
            } else {
                let verdict = if held { "held" } else { "FAILED" };
                format!("trace: {}contract {name} invariant `{}`: {verdict}", indent(depth), c.text)
            }
        });
    }

    /// trace.run.3: a contract condition was checked (contract.run.1).
    #[inline(never)]
    pub(super) fn trace_contract(&self, f: &FnDecl, c: &crate::ast::Contract, held: bool) {
        let prog = self.prog;
        let kind = if c.ensures { "ensures" } else { "requires" };
        let depth = self.depth;
        emit(5, |json| {
            if json {
                format!(
                    "{{\"api\":1,\"trace\":\"contract\",\"fn\":{},\"kind\":\"{kind}\",\"cond\":{},\"held\":{held},{}}}",
                    json_str(&f.full_name()),
                    json_str(&c.text),
                    at_json(prog, c.pos)
                )
            } else {
                let verdict = if held { "held" } else { "FAILED" };
                format!("trace: {}contract {} {kind} `{}`: {verdict}", indent(depth), f.full_name(), c.text)
            }
        });
    }

    /// trace.watch.1: after an assignment or `let mut`, show the watched
    /// variable if `name` is it, or (trace.watch.3) stands for it in this
    /// frame because a `mut` receiver or argument passed it on.
    #[inline(never)]
    pub(super) fn trace_watch(&self, name: &str, env: &Env<'_>, pos: Pos) {
        self.trace_watch_note(name, env, pos, None);
    }

    /// trace.watch.4: a write to one element of the watched collection, with what
    /// was written and where, along with the collection's whole new value.
    #[inline(never)]
    pub(super) fn trace_watch_note(&self, name: &str, env: &Env<'_>, pos: Pos, note: Option<&WatchNote>) {
        let Some(cfg) = CONFIG.get() else { return };
        let (frame, depth, via) = FRAMES.with(|f| {
            let f = f.borrow();
            let top = f.last();
            let via = top.is_some_and(|t| t.aliases.iter().any(|a| a == name));
            (top.map(|t| t.name.clone()), f.len(), via)
        });
        if cfg.watch != name && !via {
            return;
        }
        let Some(v) = lookup(env, name) else { return };
        let prog = self.prog;
        let watched = cfg.watch.as_str();
        emit(6, |json| {
            let value = show(v, LONG);
            let through = if via && name != watched { format!(" (as `{name}`)") } else { String::new() };
            if json {
                let frame = frame.as_deref().map_or(String::new(), |f| format!(",\"frame\":{}", json_str(f)));
                let via = if via && name != watched { format!(",\"via\":{}", json_str(name)) } else { String::new() };
                let op = note.map_or(String::new(), |n| n.json());
                format!("{{\"api\":1,\"trace\":\"watch\",\"name\":{},\"value\":{}{frame}{via}{op},{}}}", json_str(watched), json_str(&value), at_json(prog, pos))
            } else {
                // The frame is named only below the first, so a plain run reads as before.
                let in_frame = match &frame {
                    Some(f) if depth > 1 => format!(" in {f}"),
                    _ => String::new(),
                };
                let op = note.map_or(String::new(), |n| format!(" [{}]", n.text()));
                format!("watch: {watched} = {value} at {}{in_frame}{through}{op}", at_text(prog, pos))
            }
        });
    }

    /// trace.watch.4: what a builtin mutating method is about to change, read before it runs.
    pub(super) fn watch_before(&self, env: &Env<'_>, p: &Place, name: &str, args: &[Value]) -> Option<WatchBefore> {
        let (len, removed) = view(env, p, |v| match (v, name) {
            (Value::List(xs), "pop") => (xs.len(), xs.last()),
            (Value::List(xs), "remove_at") => (xs.len(), usize::try_from(args[0].int()).ok().and_then(|i| xs.get(i))),
            (Value::List(xs), _) => (xs.len(), None),
            (Value::Map(t), "remove") => (t.len(), t.get(&args[0]).map(|i| t.val(i).clone())),
            (Value::Map(t) | Value::Set(t), _) => (t.len(), None),
            _ => (0, None),
        })
        .ok()?;
        Some(WatchBefore { len, removed, args: args.to_vec() })
    }

    /// trace.watch.4: after a builtin mutating method on a watched collection.
    pub(super) fn watch_after(&self, env: &Env<'_>, p: &Place, name: &str, before: WatchBefore, pos: Pos) {
        if p.slot == IN_CELL {
            return;
        }
        let Some(&(root, _)) = env.get(p.slot) else { return };
        let len = view(env, p, |v| match v {
            Value::List(xs) => xs.len(),
            Value::Map(t) | Value::Set(t) => t.len(),
            _ => 0,
        })
        .unwrap_or(0);
        let a = &before.args;
        let key = |v: &Value| show(v, LONG);
        let note = match name {
            "push" => WatchNote::at("push", Some(key(&a[0])), "index", Some(before.len.to_string())),
            "pop" if len < before.len => WatchNote::at("pop", before.removed.as_ref().map(key), "index", Some(len.to_string())),
            "insert" => WatchNote::at("insert", Some(key(&a[1])), "index", Some(key(&a[0]))),
            "remove_at" if len < before.len => WatchNote::at("remove_at", before.removed.as_ref().map(key), "index", Some(key(&a[0]))),
            "remove" if len < before.len && before.removed.is_some() => {
                WatchNote::at("remove", before.removed.as_ref().map(key), "key", Some(key(&a[0])))
            }
            "remove" if len < before.len => WatchNote::at("remove", Some(key(&a[0])), "", None),
            "add" if len > before.len => WatchNote::at("add", Some(key(&a[0])), "", None),
            "swap" => WatchNote::at("swap", None, "index", Some(format!("{}, {}", key(&a[0]), key(&a[1])))),
            "extend" | "clear" | "reverse" | "sort" => WatchNote::at(name, None, "", None),
            _ => return,
        };
        self.trace_watch_note(root, env, pos, Some(&note));
    }

    /// trace.watch.4: `NAME[i] = e` and `NAME[k] = e`, after the write: `p` names the element.
    pub(super) fn watch_element(&self, root: &str, env: &Env<'_>, p: &Place, pos: Pos) {
        let (Some(step), Ok(new)) = (p.path.last(), load(env, p)) else {
            return self.trace_watch(root, env, pos);
        };
        let note = match step {
            Step::Index(i, _) => WatchNote::at("set", Some(show(&new, LONG)), "index", Some(i.to_string())),
            Step::Key(k, _) => WatchNote::at("set", Some(show(&new, LONG)), "key", Some(show(k, LONG))),
            _ => return self.trace_watch(root, env, pos),
        };
        self.trace_watch_note(root, env, pos, Some(&note));
    }

    /// trace.watch.3: before a call, find which names in the callee will
    /// hold the watched variable: a `mut` receiver or `mut` argument that
    /// is rooted in it. `args` are the call's argument expressions.
    #[inline(never)]
    pub(super) fn watch_call(&self, f: &FnDecl, recv_root: Option<&str>, args: &[crate::ast::Arg]) {
        let Some(cfg) = CONFIG.get() else { return };
        let watched = |n: &str| -> bool {
            cfg.watch == n || FRAMES.with(|fr| fr.borrow().last().is_some_and(|t| t.aliases.iter().any(|a| a == n)))
        };
        let mut aliases = Vec::new();
        if f.receiver.is_some_and(|r| r.mutable) && recv_root.is_some_and(&watched) {
            aliases.push("self".to_string());
        }
        for (a, p) in args.iter().zip(&f.params) {
            if p.mutable && crate::ast::place_root(&a.value).is_some_and(&watched) {
                aliases.push(p.name.clone());
            }
        }
        PENDING.with(|p| p.borrow_mut().push((f as *const FnDecl as usize, aliases)));
    }

    /// trace.watch.3: a body starts; it takes the aliases its call site found.
    #[inline(never)]
    pub(super) fn watch_enter(&self, f: &FnDecl) {
        let key = f as *const FnDecl as usize;
        let aliases = PENDING.with(|p| {
            let mut p = p.borrow_mut();
            if p.last().is_some_and(|(k, _)| *k == key) { p.pop().map(|(_, a)| a) } else { None }
        });
        FRAMES.with(|fr| fr.borrow_mut().push(WatchFrame { name: f.full_name(), aliases: aliases.unwrap_or_default() }));
    }

    #[inline(never)]
    pub(super) fn watch_leave(&self) {
        FRAMES.with(|fr| fr.borrow_mut().pop());
    }

    /// trace.snap.2: a trap is leaving a block; keep the frame's locals
    /// before the blocks drop them.
    #[inline(never)]
    pub(super) fn snap_block(&mut self, env: &Env<'_>) {
        if let Some(s) = &mut self.snap
            && s.pending.is_none()
            && s.frames.len() < FRAME_LIMIT
        {
            s.pending = Some(locals(env, LONG));
        }
    }

    /// trace.snap.2: a trap is leaving `f`, which was called at `pos`.
    #[inline(never)]
    pub(super) fn unwound(&mut self, f: &FnDecl, env: &Env<'_>, pos: Pos, t: Trap) -> Trap {
        if let Some(s) = &mut self.snap {
            let pending = s.pending.take();
            if s.frames.len() < FRAME_LIMIT {
                let locals = pending.unwrap_or_else(|| locals(env, LONG));
                let params = f.params.len() + usize::from(f.receiver.is_some());
                s.frames.push(Frame { func: f.full_name(), called_at: pos, params, locals });
            } else {
                s.omitted += 1;
            }
        }
        t
    }

    /// trace.snap.2: the values of the names a failed contract condition reads.
    #[inline(never)]
    pub(super) fn snap_condition(&mut self, cond: &Expr, env: &Env<'_>) {
        if self.snap.is_none() {
            return;
        }
        let mut names = Vec::new();
        names_in(cond, &mut names);
        let values = names.iter().filter_map(|n| lookup(env, n).map(|v| (n.to_string(), show(v, LONG)))).collect();
        if let Some(s) = &mut self.snap {
            s.condition = values;
        }
    }
}

/// trace.test.1: from here, a test's events are kept for its report.
pub(super) fn capture_begin() {
    if CONFIG.get().is_none() {
        return;
    }
    FRAMES.with(|f| f.borrow_mut().clear());
    PENDING.with(|p| p.borrow_mut().clear());
    CAPTURE.with(|c| *c.borrow_mut() = Some(Capture { lines: Vec::new(), dropped: 0 }));
}

/// The events kept since `capture_begin`, with a closing line saying how many
/// the budget dropped (trace.test.2).
pub(super) fn capture_end() -> Vec<String> {
    let Some(cap) = CAPTURE.with(|c| c.borrow_mut().take()) else { return Vec::new() };
    let mut lines = cap.lines;
    if cap.dropped > 0 {
        let json = CONFIG.get().is_some_and(|c| c.json);
        lines.push(if json {
            format!("{{\"api\":1,\"trace\":\"dropped\",\"count\":{}}}", cap.dropped)
        } else {
            format!("trace: {} more events dropped", cap.dropped)
        });
    }
    lines
}

/// What a builtin mutating method will change, read before it runs (trace.watch.4).
pub(super) struct WatchBefore {
    len: usize,
    removed: Option<Value>,
    args: Vec<Value>,
}

/// The element-level part of a watch event (trace.watch.4).
pub(super) struct WatchNote {
    op: &'static str,
    element: Option<String>,
    at_label: &'static str,
    at: Option<String>,
}

impl WatchNote {
    fn at(op: &str, element: Option<String>, at_label: &'static str, at: Option<String>) -> Self {
        let op = ["push", "pop", "insert", "remove_at", "remove", "add", "swap", "extend", "clear", "reverse", "sort", "set"]
            .into_iter()
            .find(|o| *o == op)
            .unwrap_or("set");
        WatchNote { op, element, at_label, at }
    }

    /// `push 3 at index 2`
    fn text(&self) -> String {
        let mut t = self.op.to_string();
        if let Some(e) = &self.element {
            t.push(' ');
            t.push_str(e);
        }
        if let Some(a) = &self.at {
            t.push_str(&format!(" at {} {a}", self.at_label));
        }
        t
    }

    /// `,"op":"push","element":"3","index":"2"`
    fn json(&self) -> String {
        let mut j = format!(",\"op\":{}", json_str(self.op));
        if let Some(e) = &self.element {
            j.push_str(&format!(",\"element\":{}", json_str(e)));
        }
        if let Some(a) = &self.at {
            j.push_str(&format!(",{}:{}", json_str(self.at_label), json_str(a)));
        }
        j
    }
}
