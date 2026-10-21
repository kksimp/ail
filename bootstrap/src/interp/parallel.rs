//! Parallel loops on a thread pool (conc.par.8-11, ADRs 0272-0275).
//!
//! A `par for` loop that lane 39's analysis accepts (conc.par.2-5), and
//! with `ail run --parallel` any loop it proves independent, runs its
//! iterations on `min(cores, 8)` threads. The interpreter's values are
//! `Rc`, so they never cross threads: the variables the body uses are
//! copied into a thread-safe `Plain` form, each worker rebuilds them and
//! runs whole chunks of iterations with its own interpreter, and sends back
//! only what the chunk wrote: changed list elements and `int` partial sums.
//! The main thread applies them in chunk order, so the result is the
//! sequential one (conc.par.1). Anything unexpected (a trap, a handle, an
//! accumulator that isn't an `int`, a sum that would overflow) discards the
//! workers' results and runs the loop in order, which gives the exact
//! sequential behaviour, trap included, because the body has no effects.

use std::io;
use std::rc::Rc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering as AtomicOrdering};

use super::{Env, Failure, Flow, Interp, Table, Trap, Value, build};
use crate::ast::{Block, CompFor, Expr, FnDecl, Program, Stmt};
use crate::check::Resolved;
use crate::check::parallel::{Outcome, Plan, analyze, analyze_comps, pure_fns};
use crate::diag::Pos;
use crate::fasthash::FastMap as HashMap;
use crate::num::Num;

/// The most worker threads (conc.par.9): enough for the 4-core target with
/// headroom, and few enough that copying captured values stays cheap.
const MAX_THREADS: usize = 8;

/// Chunks per thread, so a slow chunk doesn't leave the others idle.
const CHUNKS_PER_THREAD: usize = 4;

/// A worker's stack: as deep as the main interpreter thread's.
const WORKER_STACK: usize = 256 * 1024 * 1024;

/// Which loops run on the pool (conc.par.8): `par for` only (the default),
/// every loop the analysis proves independent (`--parallel`), or none
/// (`--deterministic`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    ParOnly = 0,
    Proven = 1,
    Off = 2,
}

static MODE: AtomicU8 = AtomicU8::new(Mode::ParOnly as u8);

pub fn set_mode(mode: Mode) {
    MODE.store(mode as u8, AtomicOrdering::Relaxed);
}

fn mode() -> Mode {
    match MODE.load(AtomicOrdering::Relaxed) {
        1 => Mode::Proven,
        2 => Mode::Off,
        _ => Mode::ParOnly,
    }
}

/// The pool size (conc.par.9): `AIL_THREADS` if set, else the machine's
/// hardware threads, at most `MAX_THREADS`.
pub fn threads() -> usize {
    static N: OnceLock<usize> = OnceLock::new();
    *N.get_or_init(|| {
        if let Some(n) = std::env::var("AIL_THREADS").ok().and_then(|s| s.trim().parse::<usize>().ok()) {
            return n.max(1);
        }
        std::thread::available_parallelism().map_or(1, |n| n.get()).min(MAX_THREADS)
    })
}

/// With `AIL_PAR_LOG` set, say on standard error how each pooled loop ran
/// (conc.par.9), so a test or an agent can see that the pool was used.
fn log(pos: Pos, what: &str) {
    static ON: OnceLock<bool> = OnceLock::new();
    if *ON.get_or_init(|| std::env::var_os("AIL_PAR_LOG").is_some()) {
        eprintln!("par: line {}: {what}", pos.line);
    }
}

/// The loops that may run on the pool, by the position of their `for`.
pub(super) type Plans = HashMap<Pos, Plan>;

/// The plans of every loop that may run on the pool under the current mode.
pub(super) fn plans(prog: &Program, resolved: &Resolved) -> Option<Rc<Plans>> {
    let mode = mode();
    if mode == Mode::Off || threads() < 2 {
        return None;
    }
    // conc.par.12: `par for` comprehensions the checker accepted.
    let comps = analyze_comps(prog, resolved);
    if mode == Mode::ParOnly && !has_par(prog) && comps.is_empty() {
        return None;
    }
    let plans: Plans = analyze(prog, resolved)
        .into_iter()
        .chain(comps)
        .filter(|v| match v.outcome {
            Outcome::Sequential => false,
            Outcome::Asserted => true,
            Outcome::Parallel => v.par || mode == Mode::Proven,
        })
        .filter_map(|v| Some((v.pos, v.plan?)))
        .collect();
    (!plans.is_empty()).then(|| Rc::new(plans))
}

fn has_par(prog: &Program) -> bool {
    fn block(b: &Block) -> bool {
        b.iter().any(|s| match s {
            Stmt::For { par: true, .. } => true,
            Stmt::For { body, .. } | Stmt::While { body, .. } | Stmt::With { body, .. } => block(body),
            Stmt::Expr(Expr::If { branches, els, .. }) => {
                branches.iter().any(|(_, b)| block(b)) || els.as_ref().is_some_and(|b| block(b))
            }
            Stmt::Expr(Expr::Match { arms, .. }) => arms.iter().any(|a| block(&a.body)),
            _ => false,
        })
    }
    prog.fns.iter().filter_map(|f| f.body.as_ref()).any(block)
}

/// A value with no `Rc`, so it can cross threads. Floats are kept as bits
/// so a round trip is exact.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Plain {
    Unit,
    Int(i64),
    Float(u64),
    Num(Num, i64),
    Bool(bool),
    Str(String),
    Bytes(Vec<u8>),
    Char(char),
    Struct(usize, Vec<Plain>),
    Enum(usize, usize, Vec<Plain>),
    List(Vec<Plain>),
    Map(Vec<(Plain, Plain)>),
    Set(Vec<Plain>),
    None,
    Nested(u32),
    Fail(Box<Plain>, Vec<String>, Vec<Pos>),
    Tensor(crate::tensor::Tensor),
    /// The same atomic cell, shared with the workers (conc.atomic.6).
    Atomic(std::sync::Arc<super::atomic::Cell>),
}

impl Plain {
    /// Whether an atomic is inside: iterations that ran already changed it,
    /// so the loop can't be run again in order (conc.atomic.6).
    fn has_atomic(&self) -> bool {
        match self {
            Plain::Atomic(_) => true,
            Plain::Struct(_, ps) | Plain::Enum(_, _, ps) | Plain::List(ps) | Plain::Set(ps) => {
                ps.iter().any(Plain::has_atomic)
            }
            Plain::Map(kvs) => kvs.iter().any(|(k, v)| k.has_atomic() || v.has_atomic()),
            Plain::Fail(e, _, _) => e.has_atomic(),
            _ => false,
        }
    }
}

/// `v` as a `Plain`; `None` for a task, group, channel or mutex handle,
/// which a worker must not touch (conc.par.10).
fn plain(v: &Value) -> Option<Plain> {
    let all = |vs: &[Value]| vs.iter().map(plain).collect::<Option<Vec<_>>>();
    Some(match v {
        Value::Unit => Plain::Unit,
        Value::Int(i) => Plain::Int(*i),
        Value::Float(x) => Plain::Float(x.to_bits()),
        Value::Num(k, b) => Plain::Num(*k, *b),
        Value::Bool(b) => Plain::Bool(*b),
        Value::Str(s) => Plain::Str(s.to_string()),
        Value::Bytes(b) => Plain::Bytes((**b).clone()),
        Value::Char(c) => Plain::Char(*c),
        Value::Struct(s, fs) => Plain::Struct(*s, all(fs)?),
        Value::Enum(e, i, fs) => Plain::Enum(*e, *i, all(fs)?),
        Value::List(xs) => Plain::List(all(xs)?),
        Value::Map(t) => Plain::Map(t.entries().map(|(k, v)| Some((plain(k)?, plain(v)?))).collect::<Option<_>>()?),
        Value::Set(t) => Plain::Set(t.keys().map(plain).collect::<Option<_>>()?),
        Value::None => Plain::None,
        Value::Nested(n) => Plain::Nested(*n),
        Value::Fail(f) => Plain::Fail(Box::new(plain(&f.error)?), f.context.clone(), f.trace.clone()),
        // A handle or a shared `ref` has identity a copy would lose (conc.par.8).
        Value::Handle(_) | Value::Ref(_) | Value::Fn(_) => return None,
        Value::Tensor(t) => Plain::Tensor((**t).clone()),
        Value::Atomic(c) => Plain::Atomic(std::sync::Arc::clone(c)),
    })
}

fn value(p: &Plain) -> Value {
    let all = |ps: &[Plain]| ps.iter().map(value).collect::<Vec<_>>();
    match p {
        Plain::Unit => Value::Unit,
        Plain::Int(i) => Value::Int(*i),
        Plain::Float(b) => Value::Float(f64::from_bits(*b)),
        Plain::Num(k, b) => Value::Num(*k, *b),
        Plain::Bool(b) => Value::Bool(*b),
        Plain::Str(s) => Value::Str(Rc::from(s.as_str())),
        Plain::Bytes(b) => Value::Bytes(Rc::new(b.clone())),
        Plain::Char(c) => Value::Char(*c),
        Plain::Struct(s, fs) => Value::Struct(*s, Rc::new(all(fs))),
        Plain::Enum(e, i, fs) => Value::Enum(*e, *i, Rc::new(all(fs))),
        Plain::List(xs) => Value::List(Rc::new(all(xs))),
        Plain::Map(kvs) => {
            let mut t = Table::default();
            for (k, v) in kvs {
                t.insert(value(k), Some(value(v)));
            }
            Value::Map(Rc::new(t))
        }
        Plain::Set(ks) => {
            let mut t = Table::default();
            for k in ks {
                t.insert(value(k), None);
            }
            Value::Set(Rc::new(t))
        }
        Plain::None => Value::None,
        Plain::Nested(n) => Value::Nested(*n),
        Plain::Tensor(t) => Value::Tensor(Rc::new(t.clone())),
        Plain::Fail(e, context, trace) => {
            Value::Fail(Rc::new(Failure { error: value(e), context: context.clone(), trace: trace.clone() }))
        }
        Plain::Atomic(c) => Value::Atomic(std::sync::Arc::clone(c)),
    }
}

/// Whether two values are the same without looking inside shared parts:
/// an element a chunk didn't write is still the one it started with.
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Unit, Value::Unit) | (Value::None, Value::None) => true,
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => x.to_bits() == y.to_bits(),
        (Value::Num(k, x), Value::Num(l, y)) => k == l && x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Char(x), Value::Char(y)) => x == y,
        (Value::Nested(x), Value::Nested(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => Rc::ptr_eq(x, y),
        (Value::Bytes(x), Value::Bytes(y)) => Rc::ptr_eq(x, y),
        (Value::Struct(_, x), Value::Struct(_, y))
        | (Value::Enum(_, _, x), Value::Enum(_, _, y))
        | (Value::List(x), Value::List(y)) => Rc::ptr_eq(x, y),
        (Value::Map(x), Value::Map(y)) | (Value::Set(x), Value::Set(y)) => Rc::ptr_eq(x, y),
        (Value::Fail(x), Value::Fail(y)) => Rc::ptr_eq(x, y),
        _ => false,
    }
}

/// What one chunk of iterations wrote.
#[derive(Default)]
struct Chunk {
    /// (list, index, new element), in the order found.
    elements: Vec<(usize, usize, Plain)>,
    /// Per accumulator: the chunk's total and the least and greatest of its
    /// running totals, each from 0.
    sums: Vec<(i128, i128, i128)>,
}

impl<'a, W: io::Write> Interp<'a, W> {
    /// conc.par.8: run `for var in iter` on the pool if it has a plan; false
    /// if it doesn't, and the caller runs it in order.
    pub(super) fn par_for(
        &mut self,
        pos: Pos,
        var: &'a str,
        iter: &'a Expr,
        body: &'a Block,
        env: &mut Env<'a>,
    ) -> Flow<bool> {
        let Some(plan) = self.par.as_ref().and_then(|p| p.get(&pos)).cloned() else { return Ok(false) };
        // The iteration values, computed once (expr.loop.4).
        let items: Vec<Value> = match iter {
            Expr::Range { lo: Some(lo), hi: Some(hi), inclusive, .. } => {
                let lo = self.eval(lo, env)?.int();
                let hi = self.eval(hi, env)?.int();
                let hi = if *inclusive { hi } else { hi.saturating_sub(1) };
                if lo > hi { Vec::new() } else { (lo..=hi).map(Value::Int).collect() }
            }
            _ => match self.eval(iter, env)? {
                Value::List(items) => items.to_vec(),
                Value::Set(t) => t.key_list(),
                other => unreachable!("checked: iterable, found {other:?}"),
            },
        };
        let threads = threads();
        if items.len() >= 2 * threads
            && let Some(done) = self.on_pool(&plan, threads, var, &items, body, env)
        {
            log(pos, &format!("loop: {} iterations on {threads} threads", items.len()));
            return done.map(|()| true);
        }
        log(pos, &format!("loop: {} iterations in order", items.len()));
        // conc.par.11: in order, from the same iteration values.
        for item in items {
            if !self.loop_body(var, item, body, env)? {
                break;
            }
        }
        Ok(true)
    }

    /// Run the iterations on `threads` workers and apply what they wrote;
    /// `None` if the loop must run in order instead (conc.par.11).
    fn on_pool(
        &mut self,
        plan: &Plan,
        threads: usize,
        var: &'a str,
        items: &[Value],
        body: &'a Block,
        env: &mut Env<'a>,
    ) -> Option<Flow<()>> {
        // The variables the body uses, by their names in `env`.
        let mut captures: Vec<(&'a str, Plain)> = Vec::new();
        for name in &plan.captures {
            if let Some((n, v)) = env.iter().rev().find(|(n, _)| *n == name.as_str()) {
                captures.push((*n, plain(v)?));
            }
        }
        let slot = |name: &str| captures.iter().position(|(n, _)| *n == name);
        let lists: Vec<usize> = plan.lists.iter().map(|r| slot(r)).collect::<Option<_>>()?;
        let sums: Vec<usize> = plan.sums.iter().map(|r| slot(r)).collect::<Option<_>>()?;
        // conc.par.4: an accumulator must hold an `int`.
        let starts: Vec<i128> = sums
            .iter()
            .map(|&i| match captures[i].1 {
                Plain::Int(x) => Some(i128::from(x)),
                _ => None,
            })
            .collect::<Option<_>>()?;
        let items: Vec<Plain> = items.iter().map(plain).collect::<Option<_>>()?;
        // conc.atomic.6: a loop that reaches an atomic can't run again in
        // order, so it reports its own trap instead.
        let atomic = captures.iter().any(|(_, p)| p.has_atomic());
        let chunks = (threads * CHUNKS_PER_THREAD).min(items.len());
        let bounds: Vec<(usize, usize)> =
            (0..chunks).map(|c| (c * items.len() / chunks, (c + 1) * items.len() / chunks)).collect();
        let results: Vec<Option<Result<Chunk, Option<Trap>>>> = {
            let next = AtomicUsize::new(0);
            let failed = AtomicBool::new(false);
            let out: Vec<std::sync::Mutex<Option<Result<Chunk, Option<Trap>>>>> =
                (0..chunks).map(|_| std::sync::Mutex::new(None)).collect();
            let (prog, resolved, depth) = (self.prog, self.resolved, self.depth);
            let shared = Shared { captures: &captures, lists: &lists, sums: &sums, items: &items, bounds: &bounds };
            std::thread::scope(|s| {
                for _ in 0..threads.min(chunks) {
                    let (next, failed, out, shared) = (&next, &failed, &out, &shared);
                    let spawned = std::thread::Builder::new()
                        .stack_size(WORKER_STACK)
                        .spawn_scoped(s, move || worker(prog, resolved, depth, var, body, shared, next, failed, out));
                    if spawned.is_err() {
                        failed.store(true, AtomicOrdering::Relaxed);
                    }
                }
            });
            let results: Vec<_> = out.into_iter().map(|m| m.into_inner().unwrap_or_else(|e| e.into_inner())).collect();
            if failed.load(AtomicOrdering::Relaxed) {
                if !atomic {
                    return None;
                }
                let trap = results.into_iter().find_map(|r| match r {
                    Some(Err(Some(t))) => Some(t),
                    _ => None,
                });
                return Some(Err(trap.unwrap_or_else(|| atomic_trap(pos_of(body))).into()));
            }
            results
        };
        let results: Vec<Chunk> = match results.into_iter().map(|r| r.and_then(Result::ok)).collect::<Option<_>>() {
            Some(r) => r,
            None if atomic => return Some(Err(atomic_trap(pos_of(body)).into())),
            None => return None,
        };
        // conc.par.4: the sequential running total overflows exactly when a
        // chunk's offset plus one of its running totals leaves `int`.
        let mut totals = starts;
        for r in &results {
            for (k, &(total, lo, hi)) in r.sums.iter().enumerate() {
                let range = i128::from(i64::MIN)..=i128::from(i64::MAX);
                if !range.contains(&(totals[k] + lo)) || !range.contains(&(totals[k] + hi)) {
                    if atomic {
                        let message = "an accumulator of this loop overflows `int`".to_string();
                        return Some(Err(Trap { rule: "conc.par.4", pos: pos_of(body), message }.into()));
                    }
                    return None;
                }
                totals[k] += total;
            }
        }
        for r in results {
            for (l, index, p) in r.elements {
                let name = captures[lists[l]].0;
                let Some((_, Value::List(items))) = env.iter_mut().rev().find(|(n, _)| *n == name) else {
                    unreachable!("captured: a list")
                };
                Rc::make_mut(items)[index] = value(&p);
            }
        }
        for (k, &i) in sums.iter().enumerate() {
            let name = captures[i].0;
            if let Some((_, v)) = env.iter_mut().rev().find(|(n, _)| *n == name) {
                *v = Value::Int(totals[k] as i64);
            }
        }
        Some(Ok(()))
    }
}

/// What every worker reads.
struct Shared<'s, 'a> {
    captures: &'s [(&'a str, Plain)],
    lists: &'s [usize],
    sums: &'s [usize],
    items: &'s [Plain],
    bounds: &'s [(usize, usize)],
}

/// A worker: its own interpreter and copies of the captured variables; it
/// takes chunks until none is left or one fails.
#[allow(clippy::too_many_arguments)]
fn worker<'a>(
    prog: &'a Program,
    resolved: &'a Resolved,
    depth: usize,
    var: &'a str,
    body: &'a Block,
    shared: &Shared<'_, 'a>,
    next: &AtomicUsize,
    failed: &AtomicBool,
    out: &[std::sync::Mutex<Option<Result<Chunk, Option<Trap>>>>],
) {
    let mut it = build(prog, resolved, io::sink());
    it.depth = depth;
    let mut env: Env<'a> = shared.captures.iter().map(|(n, p)| (*n, value(p))).collect();
    loop {
        let c = next.fetch_add(1, AtomicOrdering::Relaxed);
        if c >= shared.bounds.len() || failed.load(AtomicOrdering::Relaxed) {
            return;
        }
        let r = chunk(&mut it, var, body, shared, shared.bounds[c], &mut env);
        let ok = r.is_ok();
        *out[c].lock().unwrap_or_else(|e| e.into_inner()) = Some(r);
        if !ok {
            failed.store(true, AtomicOrdering::Relaxed);
            return;
        }
    }
}

/// Run iterations `lo..hi`; an error, with the trap if one ended it, on a
/// trap or anything the main thread must redo in order.
fn chunk<'a>(
    it: &mut Interp<'a, io::Sink>,
    var: &'a str,
    body: &'a Block,
    shared: &Shared<'_, 'a>,
    (lo, hi): (usize, usize),
    env: &mut Env<'a>,
) -> Result<Chunk, Option<Trap>> {
    let before: Vec<Value> = shared.lists.iter().map(|&l| env[l].1.clone()).collect();
    let mut sums = vec![(0i128, 0i128, 0i128); shared.sums.len()];
    for &s in shared.sums {
        env[s].1 = Value::Int(0);
    }
    for item in &shared.items[lo..hi] {
        match it.loop_body(var, value(item), body, env) {
            Ok(true) => {}
            Err(super::Exit::Trap(t)) => return Err(Some(*t)),
            Ok(false) | Err(_) => return Err(None),
        }
        for (k, &s) in shared.sums.iter().enumerate() {
            let Value::Int(x) = env[s].1 else { return Err(None) };
            let x = i128::from(x);
            sums[k] = (x, sums[k].1.min(x), sums[k].2.max(x));
        }
    }
    let mut elements = Vec::new();
    for (l, old) in before.iter().enumerate() {
        let (Value::List(old), Value::List(new)) = (old, &env[shared.lists[l]].1) else { return Err(None) };
        if Rc::ptr_eq(old, new) {
            continue;
        }
        if old.len() != new.len() {
            return Err(None);
        }
        for (i, (a, b)) in old.iter().zip(new.iter()).enumerate() {
            if !same(a, b) {
                elements.push((l, i, plain(b).ok_or(None)?));
            }
        }
    }
    Ok(Chunk { elements, sums })
}

/// Where a loop body starts, for a trap that belongs to no one iteration.
fn pos_of(body: &Block) -> Pos {
    body.first().map_or(Pos { line: 0, col: 0 }, Stmt::pos)
}

/// conc.atomic.6: a loop that reaches an atomic stopped on the pool for a
/// reason other than a trap, and can't run again in order.
fn atomic_trap(pos: Pos) -> Trap {
    let message = "a parallel loop that updates an atomic couldn't finish on the pool".to_string();
    Trap { rule: "conc.atomic.6", pos, message }
}

impl<'a, W: io::Write> Interp<'a, W> {
    /// conc.par.12: `[elem par for c.var in c.iter if c.cond]` with a plan:
    /// the elements computed on the pool, in order. `None` without a plan,
    /// and the caller evaluates it as a plain comprehension.
    pub(super) fn par_comp(&mut self, elem: &'a Expr, c: &'a CompFor, env: &mut Env<'a>) -> Flow<Option<Value>> {
        let Some(plan) = self.par.as_ref().and_then(|p| p.get(&c.var_pos)).cloned() else { return Ok(None) };
        let items: Vec<Value> = match &c.iter {
            Expr::Range { lo: Some(lo), hi: Some(hi), inclusive, .. } => {
                let lo = self.eval(lo, env)?.int();
                let hi = self.eval(hi, env)?.int();
                let hi = if *inclusive { hi } else { hi.saturating_sub(1) };
                if lo > hi { Vec::new() } else { (lo..=hi).map(Value::Int).collect() }
            }
            iter => match self.eval(iter, env)? {
                Value::List(items) => items.to_vec(),
                Value::Set(t) => t.key_list(),
                other => unreachable!("checked: iterable, found {other:?}"),
            },
        };
        let threads = threads();
        if items.len() >= 2 * threads
            && let Some(out) = self.comp_on_pool(&plan, threads, elem, c, &items, env)
        {
            log(c.var_pos, &format!("comprehension: {} elements on {threads} threads", items.len()));
            return Ok(Some(Value::List(Rc::new(out))));
        }
        log(c.var_pos, &format!("comprehension: {} elements in order", items.len()));
        // conc.par.11: in order, from the same items.
        let mut out = Vec::with_capacity(items.len());
        for item in items {
            env.push((&c.var, item));
            let r = comp_element(self, elem, c, env);
            env.pop();
            match r {
                Ok(Some(v)) => out.push(v),
                Ok(None) | Err(super::Exit::Continue) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(Some(Value::List(Rc::new(out))))
    }

    /// The elements computed by `threads` workers, in order; `None` if
    /// they must be computed in order instead (conc.par.11).
    fn comp_on_pool(
        &mut self,
        plan: &Plan,
        threads: usize,
        elem: &'a Expr,
        c: &'a CompFor,
        items: &[Value],
        env: &mut Env<'a>,
    ) -> Option<Vec<Value>> {
        let mut captures: Vec<(&'a str, Plain)> = Vec::new();
        for name in &plan.captures {
            if let Some((n, v)) = env.iter().rev().find(|(n, _)| *n == name.as_str()) {
                captures.push((*n, plain(v)?));
            }
        }
        let items: Vec<Plain> = items.iter().map(plain).collect::<Option<_>>()?;
        let chunks = (threads * CHUNKS_PER_THREAD).min(items.len());
        let bounds: Vec<(usize, usize)> =
            (0..chunks).map(|k| (k * items.len() / chunks, (k + 1) * items.len() / chunks)).collect();
        let next = AtomicUsize::new(0);
        let failed = AtomicBool::new(false);
        let out: Vec<std::sync::Mutex<Option<Vec<Plain>>>> = (0..chunks).map(|_| std::sync::Mutex::new(None)).collect();
        let (prog, resolved, depth) = (self.prog, self.resolved, self.depth);
        std::thread::scope(|s| {
            for _ in 0..threads.min(chunks) {
                let (next, failed, out, captures, items, bounds) = (&next, &failed, &out, &captures, &items, &bounds);
                let spawned = std::thread::Builder::new().stack_size(WORKER_STACK).spawn_scoped(s, move || {
                    let mut it = build(prog, resolved, io::sink());
                    it.depth = depth;
                    let mut env: Env<'a> = captures.iter().map(|(n, p)| (*n, value(p))).collect();
                    loop {
                        let k = next.fetch_add(1, AtomicOrdering::Relaxed);
                        if k >= bounds.len() || failed.load(AtomicOrdering::Relaxed) {
                            return;
                        }
                        let (lo, hi) = bounds[k];
                        let mut done = Vec::with_capacity(hi - lo);
                        for item in &items[lo..hi] {
                            env.push((&c.var, value(item)));
                            let r = comp_element(&mut it, elem, c, &mut env);
                            env.pop();
                            match r {
                                Ok(Some(v)) => match plain(&v) {
                                    Some(p) => done.push(p),
                                    None => {
                                        failed.store(true, AtomicOrdering::Relaxed);
                                        return;
                                    }
                                },
                                Ok(None) | Err(super::Exit::Continue) => {}
                                Err(_) => {
                                    failed.store(true, AtomicOrdering::Relaxed);
                                    return;
                                }
                            }
                        }
                        *out[k].lock().unwrap_or_else(|e| e.into_inner()) = Some(done);
                    }
                });
                if spawned.is_err() {
                    failed.store(true, AtomicOrdering::Relaxed);
                }
            }
        });
        if failed.load(AtomicOrdering::Relaxed) {
            return None;
        }
        let mut all = Vec::with_capacity(items.len());
        for m in out {
            all.extend(m.into_inner().unwrap_or_else(|e| e.into_inner())?.iter().map(value));
        }
        Some(all)
    }
}

/// One element of a `par for` comprehension with its variable bound:
/// `None` when the filter rejects it (expr.comp.3).
fn comp_element<'a, W: io::Write>(
    it: &mut Interp<'a, W>,
    elem: &'a Expr,
    c: &'a CompFor,
    env: &mut Env<'a>,
) -> Flow<Option<Value>> {
    if let Some(cond) = &c.cond
        && !matches!(it.eval(cond, env)?, Value::Bool(true))
    {
        return Ok(None);
    }
    it.eval(elem, env).map(Some)
}

/// A task that runs on its own thread outside the scheduler (conc.par.10):
/// its result, or its trap.
pub(super) type Pooled = std::thread::JoinHandle<Result<Plain, Trap>>;

/// A pooled task's outcome as a value.
pub(super) fn unpool(task: Pooled, pos: Pos) -> Result<Value, Trap> {
    match task.join() {
        Ok(r) => r.map(|p| value(&p)),
        Err(_) => Err(Trap { rule: "conc.par.10", pos, message: "a pooled task's thread failed".into() }),
    }
}

impl<'a, W: io::Write> Interp<'a, W> {
    /// conc.par.10: start `spawn f(args)` on its own thread, outside the
    /// one-at-a-time scheduler, when `f` has no ordering effect and touches
    /// no handle, so when it runs can't be observed. `None` if it must go
    /// to the scheduler.
    pub(super) fn pool_task(&mut self, f: &'a FnDecl, args: &[Value], pos: Pos) -> Option<Pooled> {
        if mode() == Mode::Off || threads() < 2 {
            return None;
        }
        let (prog, resolved) = (self.prog, self.resolved);
        let pure = self.pure.get_or_insert_with(|| Rc::new(pure_fns(prog, resolved)));
        let i = prog.fns.iter().position(|g| std::ptr::eq(g, f))?;
        if !pure[i] {
            return None;
        }
        let args: Vec<Plain> = args.iter().map(plain).collect::<Option<_>>()?;
        // SAFETY: as for the scheduler's tasks (`tasks::spawn`): every
        // pooled thread is joined, by `wait`, the end of its block, or
        // `tasks::shutdown`, before the program and facts are dropped.
        let (prog, resolved, f) = unsafe {
            (
                std::mem::transmute::<&Program, &'static Program>(prog),
                std::mem::transmute::<&Resolved, &'static Resolved>(resolved),
                std::mem::transmute::<&FnDecl, &'static FnDecl>(f),
            )
        };
        log(pos, &format!("task `{}` on its own thread", f.full_name()));
        std::thread::Builder::new()
            .stack_size(WORKER_STACK)
            .spawn(move || {
                let mut it = build(prog, resolved, io::sink());
                let (v, _) = it.call(f, None, args.iter().map(value).collect(), pos)?;
                plain(&v).ok_or_else(|| Trap {
                    rule: "conc.par.10",
                    pos,
                    message: "a task on the pool returned a task, channel or mutex handle".into(),
                })
            })
            .ok()
    }
}
