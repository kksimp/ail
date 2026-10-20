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

thread_local! {
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
        Value::Str(_) | Value::Char(_) | Value::Int(_) | Value::Num(..) | Value::Bool(_) | Value::None => {
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
                write_value(s, x, budget);
            }
            s.push(']');
        }
        Value::Struct(st, values) => NAMES.with(|n| {
            let n = n.borrow();
            let (name, names) = &n.structs[*st];
            s.push_str(name);
            fields(s, names, values);
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
            for (i, (k, x)) in t.entries().enumerate() {
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
    /// variable if `name` is it.
    #[inline(never)]
    pub(super) fn trace_watch(&self, name: &str, env: &Env<'_>, pos: Pos) {
        let Some(cfg) = CONFIG.get() else { return };
        if cfg.watch != name {
            return;
        }
        let Some(v) = lookup(env, name) else { return };
        let prog = self.prog;
        emit(6, |json| {
            let value = show(v, LONG);
            if json {
                format!("{{\"api\":1,\"trace\":\"watch\",\"name\":{},\"value\":{},{}}}", json_str(name), json_str(&value), at_json(prog, pos))
            } else {
                format!("watch: {name} = {value} at {}", at_text(prog, pos))
            }
        });
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
