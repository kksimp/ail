//! Parallel loops (conc.par.1-7, ADRs 0236-0239 and 0246-0247): which `for` loops have
//! independent iterations, and the checks on `par for`.
//!
//! Runs after type checking and reads only the program, the checker's
//! `Resolved` facts and the inferred effects (eff.infer.*). A loop is
//! independent (conc.par.2) when its body has no ordering effect, doesn't
//! leave the loop early, and every place it writes that outlives an
//! iteration is an element picked by the loop variable (conc.par.3) or an
//! `int` accumulator added to with `+=` (conc.par.4).

use std::collections::{BTreeMap, HashMap};

use super::Resolved;
use super::effects::{CAPABILITIES, Effects, fn_id};
use crate::ast::{
    BinOp, Block, CompFor, Cond, Else, Expr, FnDecl, InterpPart, PathStep, Pattern, Program, Stmt, TypeKind, UnOp, is_prelude,
    place_path,
};
use crate::diag::{Diagnostic, Pos, json_str};

/// Effects that order a loop's iterations (conc.par.2): output, `unsafe`,
/// tasks, and every capability.
fn ordering(e: &str) -> bool {
    matches!(e, "io" | "unsafe" | "spawn") || CAPABILITIES.contains(&e)
}

/// Built-in methods that change their receiver (type.coll.5-11).
const MUTATING: &[&str] = &["push", "pop", "swap", "sort", "remove", "add"];

/// What the analysis concluded about one loop (conc.par.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Proven independent: the loop may run in parallel (conc.par.2).
    Parallel,
    /// Not proven, but `par for` asserts it and nothing disproves it (conc.par.5).
    Asserted,
    /// Runs in order.
    Sequential,
}

impl Outcome {
    pub fn word(self) -> &'static str {
        match self {
            Outcome::Parallel => "parallel",
            Outcome::Asserted => "asserted",
            Outcome::Sequential => "sequential",
        }
    }
}

/// One loop's verdict (conc.par.6).
#[derive(Debug)]
pub struct Verdict {
    /// The `for` (or `par`).
    pub pos: Pos,
    /// The enclosing function's stable id.
    pub func: String,
    pub var: String,
    pub par: bool,
    pub outcome: Outcome,
    pub rule: &'static str,
    pub reason: String,
    /// Where the reason is: the first write or exit that orders the loop.
    pub at: Pos,
    help: Option<String>,
    /// How the interpreter's pool may run the loop, when it can
    /// (conc.par.8); meaningful only when the outcome isn't sequential.
    pub plan: Option<Plan>,
}

/// What a worker needs to run a loop's iterations (conc.par.8, ADR 0272):
/// the variables declared outside the body that it reads or writes, the
/// lists whose elements it writes, and the `int` accumulators it adds to.
#[derive(Debug, Clone, Default)]
pub struct Plan {
    pub captures: Vec<String>,
    pub lists: Vec<String>,
    pub sums: Vec<String>,
}

/// The plan for a loop whose writes are all list elements, or single `+=`
/// accumulators outside nested loops; `None` for any other write.
fn plan_of(writes: &[Write], reads: &[Read], mutable: bool) -> Option<Plan> {
    if mutable {
        return None;
    }
    let mut plan = Plan::default();
    let mut roots: Vec<&str> = writes.iter().map(|w| w.root).collect();
    roots.sort_unstable();
    roots.dedup();
    for root in roots {
        let ws: Vec<&Write> = writes.iter().filter(|w| w.root == root).collect();
        let element = |w: &&Write| match &w.kind {
            Kind::Element(s) | Kind::Unproven(s) => s.first().is_some_and(|s| s.starts_with('[')),
            _ => false,
        };
        if ws.iter().all(element) {
            plan.lists.push(root.to_string());
        } else if ws.len() == 1 && ws[0].kind == Kind::Add && ws[0].depth == 0 {
            plan.sums.push(root.to_string());
        } else {
            return None;
        }
    }
    let mut captures: Vec<&str> = writes.iter().map(|w| w.root).chain(reads.iter().map(|r| r.root)).collect();
    captures.sort_unstable();
    captures.dedup();
    plan.captures = captures.into_iter().map(String::from).collect();
    Some(plan)
}

/// Why a loop isn't proven independent. `provable` problems make
/// iterations interfere for certain; the others are what `par for` asserts
/// away (conc.par.5).
struct Problem {
    rule: &'static str,
    reason: String,
    help: String,
    at: Pos,
    provable: bool,
    /// A call of a handle's method (conc.par.2).
    handle: bool,
    /// A call of an atomic's method (conc.atomic.6).
    atomic: bool,
}

/// How a write reaches a place that outlives an iteration.
#[derive(Debug, PartialEq)]
enum Kind {
    /// `x = ...`, `x.push(...)`, `f(mut x)`: the whole variable.
    Whole,
    /// `x += e` (conc.par.4).
    Add,
    /// `x[i]...` with `i` the loop variable: the steps up to `[i]` (conc.par.3).
    Element(Vec<String>),
    /// Any other element: the steps written.
    Unproven(Vec<String>),
}

struct Write<'a> {
    root: &'a str,
    kind: Kind,
    pos: Pos,
    /// Loops nested inside the body around the write.
    depth: usize,
}

/// A read of a variable that outlives the iteration.
struct Read<'a> {
    root: &'a str,
    steps: Vec<String>,
    /// Only `.len()` of the place.
    len_only: bool,
    pos: Pos,
}

/// The analysis of one loop body.
struct Scan<'a> {
    prog: &'a Program,
    resolved: &'a Resolved,
    var: &'a str,
    /// The loop is over a range, so `var` differs in every iteration.
    range: bool,
    scopes: Vec<Vec<&'a str>>,
    /// Loops nested inside the body, so `break` there is local.
    depth: usize,
    writes: Vec<Write<'a>>,
    reads: Vec<Read<'a>>,
    problems: Vec<Problem>,
    /// Per function: it may call a method of a handle (`SYNC`, conc.par.2)
    /// or of an atomic (`ATOMIC`, conc.atomic.6).
    sync: &'a [u8],
    /// Free functions by name.
    fns: &'a HashMap<&'a str, usize>,
}

/// `[i]`, `[i - 1]`, `.name`; `[...]` for an index too complex to show.
fn step_text(step: &PathStep) -> String {
    match step {
        PathStep::Field(n) => format!(".{n}"),
        PathStep::Index(e) => format!("[{}]", show(e).unwrap_or_else(|| "...".into())),
    }
}

/// A short index expression as source text: names, literals, arithmetic,
/// fields, indexes and `len()`.
fn show(e: &Expr) -> Option<String> {
    Some(match e {
        Expr::Name(n, _) => n.clone(),
        Expr::Int(k, _) => k.to_string(),
        Expr::Binary { op, lhs, rhs, .. } if op.is_arithmetic() => {
            let sym = match op {
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Mul => "*",
                BinOp::Div => "/",
                _ => "%",
            };
            format!("{} {sym} {}", show(lhs)?, show(rhs)?)
        }
        Expr::Field { recv, name, .. } => format!("{}.{name}", show(recv)?),
        Expr::Index { recv, index, .. } => format!("{}[{}]", show(recv)?, show(index)?),
        Expr::Method { recv, name, args, .. } if args.is_empty() => format!("{}.{name}()", show(recv)?),
        _ => return None,
    })
}

fn place_text(root: &str, steps: &[String]) -> String {
    format!("{root}{}", steps.concat())
}

impl<'a> Scan<'a> {
    fn local(&self, n: &str) -> bool {
        n == self.var || self.scopes.iter().any(|s| s.contains(&n))
    }

    fn bind(&mut self, n: &'a str) {
        if let Some(s) = self.scopes.last_mut() {
            s.push(n);
        }
    }

    fn problem(&mut self, rule: &'static str, at: Pos, provable: bool, reason: String, help: impl Into<String>) {
        self.problems.push(Problem { rule, reason, help: help.into(), at, provable, handle: false, atomic: false });
    }

    fn exit(&mut self, what: &str, at: Pos) {
        self.problem(
            "conc.par.2",
            at,
            true,
            format!("leaves the loop early with {what}"),
            "a later iteration must not run after an earlier one leaves; keep the loop sequential (drop `par`), or collect the results and decide after the loop",
        );
    }

    fn block(&mut self, b: &'a Block) {
        self.scopes.push(Vec::new());
        for s in b {
            self.stmt(s);
        }
        self.scopes.pop();
    }

    fn scoped(&mut self, names: Vec<&'a str>, b: &'a Block) {
        self.scopes.push(names);
        self.block(b);
        self.scopes.pop();
    }

    fn stmt(&mut self, s: &'a Stmt) {
        match s {
            Stmt::Let { name, value, otherwise, .. } => {
                self.expr(value);
                if let Some(o) = otherwise {
                    self.otherwise(o);
                }
                self.bind(name);
            }
            Stmt::LetTuple { names, value, .. } => {
                self.expr(value);
                names.iter().for_each(|b| self.bind(&b.name));
            }
            Stmt::Assign { target, op, value, .. } => {
                self.write(target, *op, target.pos());
                self.expr(value);
            }
            Stmt::While { cond, body, .. } => {
                self.expr(cond);
                self.depth += 1;
                self.block(body);
                self.depth -= 1;
            }
            Stmt::For { var, mutable, iter, body, .. } => {
                if *mutable {
                    self.write(iter, None, iter.pos());
                } else {
                    self.expr(iter);
                    self.channel_loop(iter);
                }
                self.depth += 1;
                self.scoped(vec![var.as_str()], body);
                self.depth -= 1;
            }
            Stmt::Break(p) if self.depth == 0 => self.exit("`break`", *p),
            Stmt::Break(_) | Stmt::Continue(_) => {}
            Stmt::Return(e, p) => {
                if let Some(e) = e {
                    self.expr(e);
                }
                self.exit("`return`", *p);
            }
            Stmt::Expr(e) => self.expr(e),
            Stmt::Handle { value, otherwise } => {
                self.expr(value);
                self.otherwise(otherwise);
            }
            // `defer` runs at block exit; its statement is part of the body (own.defer.1).
            Stmt::Defer(inner, _) => self.stmt(inner),
            Stmt::Unsafe { body, .. } => self.block(body),
            // conc.share.6: holding a mutex synchronises with other tasks.
            Stmt::With { mutex, var, body, pos, .. } => {
                self.expr(mutex);
                self.sync_problem(*pos, "holds a mutex with `with`".into());
                self.scoped(vec![var.as_str()], body);
            }
        }
    }

    fn otherwise(&mut self, o: &'a Else) {
        match o {
            Else::Value(e) => self.expr(e),
            Else::Leave(s) => self.stmt(s),
            Else::Block { err, body, .. } => self.scoped(err.iter().map(|(n, _)| n.as_str()).collect(), body),
        }
    }

    /// A write to `target`: a variable, a field or an element.
    fn write(&mut self, target: &'a Expr, op: Option<BinOp>, pos: Pos) {
        let Some((root, path)) = place_path(target) else {
            self.expr(target);
            return;
        };
        for step in &path {
            if let PathStep::Index(i) = step {
                self.expr(i);
            }
        }
        if self.local(root) {
            return;
        }
        let steps: Vec<String> = path.iter().map(step_text).collect();
        let kind = if path.is_empty() {
            if op == Some(BinOp::Add) { Kind::Add } else { Kind::Whole }
        } else {
            let at_var = path.iter().position(|s| matches!(s, PathStep::Index(Expr::Name(n, _)) if n == self.var));
            match at_var {
                Some(k) if self.range && self.invariant(&path[..k]) => Kind::Element(steps[..=k].to_vec()),
                _ => Kind::Unproven(steps),
            }
        };
        self.writes.push(Write { root, kind, pos, depth: self.depth });
    }

    /// The steps pick the same place in every iteration: fields, literal
    /// indexes, and indexes by variables declared outside the body.
    fn invariant(&self, steps: &[PathStep]) -> bool {
        steps.iter().all(|s| match s {
            PathStep::Field(_) | PathStep::Index(Expr::Int(..)) => true,
            PathStep::Index(Expr::Name(n, _)) => !self.local(n),
            PathStep::Index(_) => false,
        })
    }

    fn read_place(&mut self, e: &'a Expr, len_only: bool) -> bool {
        let Some((root, path)) = place_path(e) else { return false };
        for step in &path {
            if let PathStep::Index(i) = step {
                self.expr(i);
            }
        }
        if !self.local(root) {
            let steps = path.iter().map(step_text).collect();
            self.reads.push(Read { root, steps, len_only, pos: e.pos() });
        }
        true
    }

    /// A problem that orders the iterations because they synchronise
    /// through a handle (conc.par.2).
    fn sync_problem(&mut self, at: Pos, reason: String) {
        self.problem(
            "conc.par.2",
            at,
            true,
            reason,
            "synchronisation orders the iterations; keep the loop sequential (drop `par`)",
        );
        if let Some(p) = self.problems.last_mut() {
            p.handle = true;
        }
    }

    /// `for v in ch` receives from a channel (conc.share.7), which synchronises.
    fn channel_loop(&mut self, iter: &Expr) {
        if self.resolved.handle_calls.contains(&iter.pos()) {
            self.sync_problem(iter.pos(), "receives from a channel with `for ... in`".into());
        }
    }

    /// A call of function `i`, which may synchronise through a handle.
    fn calls_sync(&mut self, i: usize, name: &str, at: Pos) {
        // Empty while `context` computes the direct facts.
        let flags = self.sync.get(i).copied().unwrap_or(0);
        if flags & SYNC == 0 && flags & ATOMIC != 0 {
            self.atomic_problem(at, format!("calls `{name}`, which uses an `Atomic`"));
        }
        if flags & SYNC != 0 {
            self.problem(
                "conc.par.2",
                at,
                true,
                format!("calls `{name}`, which uses a task, channel or mutex handle"),
                "synchronisation orders the iterations; keep the loop sequential (drop `par`)",
            );
        }
    }

    /// conc.atomic.6: an atomic's operations may observe the order of the
    /// iterations, so only `par for` asserts that they don't matter.
    fn atomic_problem(&mut self, at: Pos, reason: String) {
        self.problem(
            "conc.atomic.6",
            at,
            false,
            reason,
            "iterations update the atomic in any order; write `par for` if the result doesn't depend on it",
        );
        if let Some(p) = self.problems.last_mut() {
            p.atomic = true;
        }
    }

    fn arg(&mut self, mutable: bool, value: &'a Expr) {
        if mutable {
            self.write(value, None, value.pos());
        } else {
            self.expr(value);
        }
    }

    fn expr(&mut self, e: &'a Expr) {
        match e {
            // A `ref` cell is shared by every copy of it (own.ref.1), so
            // iterations that make one may also write through it.
            Expr::Ref { value, pos } => {
                self.expr(value);
                self.problem(
                    "conc.par.2",
                    *pos,
                    false,
                    "makes a `ref` cell, which copies share".to_string(),
                    "iterations may write through the cell in any order; keep the loop sequential (drop `par`)",
                );
            }
            Expr::Str(..)
            | Expr::Bytes(..)
            | Expr::Int(..)
            | Expr::Float(..)
            | Expr::Char(..)
            | Expr::Bool(..)
            | Expr::None(_)
            | Expr::Hole(_)
            | Expr::TypeArgs { .. } => {}
            Expr::Name(..) | Expr::Field { .. } | Expr::Index { .. } if self.read_place(e, false) => {}
            Expr::Name(..) => {}
            Expr::Field { recv, .. } => self.expr(recv),
            Expr::Index { recv, index, .. } => {
                self.expr(recv);
                self.expr(index);
            }
            Expr::Interp(parts, _) => {
                for p in parts {
                    if let InterpPart::Expr(x) | InterpPart::Fmt(x, _) = p {
                        self.expr(x);
                    }
                }
            }
            Expr::List(items, _) | Expr::Set(items, _) | Expr::Tuple(items, _) => items.iter().for_each(|x| self.expr(x)),
            Expr::Map(pairs, _) => {
                for (k, v) in pairs {
                    self.expr(k);
                    self.expr(v);
                }
            }
            Expr::Call { callee, args, .. } => {
                match &**callee {
                    Expr::Name(n, p) | Expr::TypeArgs { name: n, pos: p, .. } => {
                        if let Some(&i) = self.fns.get(n.as_str()) {
                            self.calls_sync(i, n, *p);
                        }
                    }
                    _ => self.expr(callee),
                }
                args.iter().for_each(|a| self.arg(a.mutable, &a.value));
            }
            Expr::Method { recv, name, name_pos, args } => {
                let std_module = self.resolved.std_calls.contains_key(name_pos);
                let task_module = matches!(
                    self.resolved.std_calls.get(name_pos),
                    Some(crate::stdlib::StdCall::Fn("task", _))
                );
                if self.resolved.atomic_calls.contains(name_pos) {
                    self.atomic_problem(*name_pos, format!("calls `.{name}()` on an `Atomic`"));
                } else if self.resolved.handle_calls.contains(name_pos) || task_module {
                    self.problem(
                        "conc.par.2",
                        *name_pos,
                        true,
                        format!("calls `.{name}()` on a task, channel or mutex handle"),
                        "synchronisation orders the iterations; keep the loop sequential (drop `par`)",
                    );
                    if let Some(p) = self.problems.last_mut() {
                        p.handle = true;
                    }
                } else if let Some(&i) = self.resolved.targets.get(name_pos) {
                    self.calls_sync(i, name, *name_pos);
                }
                let mutating = match self.resolved.targets.get(name_pos) {
                    Some(&f) => self.prog.fns[f].receiver.is_some_and(|r| r.mutable),
                    // An atomic's methods change the shared cell, not the binding.
                    None => {
                        !std_module
                            && !self.resolved.atomic_calls.contains(name_pos)
                            && !self.resolved.calls.contains_key(name_pos)
                            && MUTATING.contains(&name.as_str())
                    }
                };
                if mutating {
                    self.write(recv, None, recv.pos());
                } else if !(name == "len" && self.read_place(recv, true)) {
                    self.expr(recv);
                }
                args.iter().for_each(|a| self.arg(a.mutable, &a.value));
            }
            Expr::Unary { operand, .. } => self.expr(operand),
            Expr::Binary { lhs, rhs, .. } => {
                self.expr(lhs);
                self.expr(rhs);
            }
            Expr::Range { lo, hi, .. } => {
                for x in [lo, hi].into_iter().flatten() {
                    self.expr(x);
                }
            }
            Expr::If { branches, els, .. } => {
                for (cond, body) in branches {
                    match cond {
                        Cond::Expr(x) => {
                            self.expr(x);
                            self.block(body);
                        }
                        Cond::Bind { name, value, .. } => {
                            self.expr(value);
                            self.scoped(vec![name.as_str()], body);
                        }
                    }
                }
                if let Some(b) = els {
                    self.block(b);
                }
            }
            Expr::Match { scrutinee, arms, .. } => {
                self.expr(scrutinee);
                for arm in arms {
                    if let Some(g) = &arm.guard {
                        self.expr(g);
                    }
                    self.scoped(pattern_names(&arm.pat), &arm.body);
                }
            }
            Expr::Try { expr, context, pos } => {
                self.expr(expr);
                if let Some(c) = context {
                    self.expr(c);
                }
                self.exit("`?`", *pos);
            }
            Expr::Else { value, otherwise, .. } => {
                self.expr(value);
                self.otherwise(otherwise);
            }
            // expr.comp.1: the clauses' variables are local, and `else break`
            // ends the comprehension, not the loop.
            Expr::Comp { elem, value, clauses, .. } => {
                self.depth += 1;
                self.scopes.push(Vec::new());
                for c in clauses {
                    self.expr(&c.iter);
                    self.channel_loop(&c.iter);
                    self.bind(&c.var);
                    c.unpack.iter().for_each(|b| self.bind(&b.name));
                    if let Some(cond) = &c.cond {
                        self.expr(cond);
                    }
                }
                self.expr(elem);
                if let Some(v) = value {
                    self.expr(v);
                }
                self.scopes.pop();
                self.depth -= 1;
            }
        }
    }
}

fn pattern_names(p: &Pattern) -> Vec<&str> {
    match p {
        Pattern::Variant { bindings: Some(b), .. } => b.iter().map(|(n, _)| n.as_str()).filter(|n| *n != "_").collect(),
        Pattern::Bind(n, _) => vec![n.as_str()],
        Pattern::Tuple(ps, _) => ps.iter().flat_map(pattern_names).collect(),
        _ => Vec::new(),
    }
}

/// Whether `name`, declared in `f`, holds an `int` (`Some(true)`), a float
/// (`Some(false)`), or something the analysis can't tell (conc.par.4).
fn holds_int(f: &FnDecl, resolved: &Resolved, name: &str) -> Option<bool> {
    if let Some(p) = f.params.iter().find(|p| p.name == name) {
        return match &p.ty.kind {
            TypeKind::Named { name, args } if args.is_empty() => match name.as_str() {
                "int" | "i64" | "i8" | "i16" | "i32" | "u8" | "u16" | "u32" | "u64" => Some(true),
                "float" | "f64" | "f32" => Some(false),
                _ => None,
            },
            _ => None,
        };
    }
    let mut found = None;
    each_let(f.body.as_ref()?, &mut |n, value| {
        if n == name {
            found = Some(literal_kind(value, resolved));
        }
    });
    found.flatten()
}

/// Whether `name` is a `ref` in `f`: a `ref T` parameter or `name := ref ...`.
fn is_ref_local(f: &FnDecl, name: &str) -> bool {
    if let Some(p) = f.params.iter().find(|p| p.name == name) {
        return matches!(p.ty.kind, TypeKind::Ref(_));
    }
    let mut found = false;
    if let Some(b) = f.body.as_ref() {
        each_let(b, &mut |n, value| {
            if n == name {
                found = matches!(value, Expr::Ref { .. });
            }
        });
    }
    found
}

fn literal_kind(e: &Expr, resolved: &Resolved) -> Option<bool> {
    match e {
        Expr::Int(_, p) => Some(!resolved.float_literals.contains(p)),
        Expr::Float(..) => Some(false),
        Expr::Unary { op: UnOp::Neg, operand, .. } => literal_kind(operand, resolved),
        _ => None,
    }
}

/// Calls `f` with each `name := value` in `b`, nested blocks included.
fn each_let<'a>(b: &'a Block, f: &mut dyn FnMut(&'a str, &'a Expr)) {
    for s in b {
        if let Stmt::Let { name, value, .. } = s {
            f(name, value);
        }
        each_block(s, &mut |inner| each_let(inner, f));
    }
}

/// Calls `f` with each block directly inside `s` (not nested deeper).
pub(crate) fn each_block<'a>(s: &'a Stmt, f: &mut dyn FnMut(&'a Block)) {
    fn other<'a>(o: &'a Else, f: &mut dyn FnMut(&'a Block)) {
        match o {
            Else::Value(e) => expr_blocks(e, f),
            Else::Leave(s) => each_block(s, f),
            Else::Block { body, .. } => f(body),
        }
    }
    match s {
        Stmt::Let { value, otherwise, .. } => {
            expr_blocks(value, f);
            if let Some(o) = otherwise {
                other(o, f);
            }
        }
        Stmt::Assign { target, value, .. } => {
            expr_blocks(target, f);
            expr_blocks(value, f);
        }
        Stmt::While { cond: e, body, .. } | Stmt::For { iter: e, body, .. } | Stmt::With { mutex: e, body, .. } => {
            expr_blocks(e, f);
            f(body);
        }
        Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return(None, _) => {}
        Stmt::Return(Some(e), _) | Stmt::Expr(e) | Stmt::LetTuple { value: e, .. } => expr_blocks(e, f),
        Stmt::Handle { value, otherwise } => {
            expr_blocks(value, f);
            other(otherwise, f);
        }
        Stmt::Unsafe { body, .. } => f(body),
        Stmt::Defer(inner, _) => each_block(inner, f),
    }
}

/// Calls `f` with each block inside `e` (`if`, `match`, `else` blocks).
fn expr_blocks<'a>(e: &'a Expr, f: &mut dyn FnMut(&'a Block)) {
    match e {
        Expr::If { branches, els, .. } => {
            for (cond, body) in branches {
                let (Cond::Expr(x) | Cond::Bind { value: x, .. }) = cond;
                expr_blocks(x, f);
                f(body);
            }
            if let Some(b) = els {
                f(b);
            }
        }
        Expr::Match { scrutinee, arms, .. } => {
            expr_blocks(scrutinee, f);
            for a in arms {
                if let Some(g) = &a.guard {
                    expr_blocks(g, f);
                }
                f(&a.body);
            }
        }
        Expr::Else { value, otherwise, .. } => {
            expr_blocks(value, f);
            match &**otherwise {
                Else::Value(x) => expr_blocks(x, f),
                Else::Leave(s) => each_block(s, f),
                Else::Block { body, .. } => f(body),
            }
        }
        Expr::Call { callee, args, .. } => {
            expr_blocks(callee, f);
            args.iter().for_each(|a| expr_blocks(&a.value, f));
        }
        Expr::Method { recv, args, .. } => {
            expr_blocks(recv, f);
            args.iter().for_each(|a| expr_blocks(&a.value, f));
        }
        Expr::Field { recv, .. } => expr_blocks(recv, f),
        Expr::Index { recv: a, index: b, .. } | Expr::Binary { lhs: a, rhs: b, .. } => {
            expr_blocks(a, f);
            expr_blocks(b, f);
        }
        Expr::Unary { operand: x, .. } | Expr::Try { expr: x, .. } => expr_blocks(x, f),
        Expr::Comp { elem, value, clauses, .. } => {
            for c in clauses {
                expr_blocks(&c.iter, f);
                c.cond.iter().for_each(|x| expr_blocks(x, f));
            }
            expr_blocks(elem, f);
            value.iter().for_each(|x| expr_blocks(x, f));
        }
        Expr::List(items, _) | Expr::Set(items, _) | Expr::Tuple(items, _) => items.iter().for_each(|x| expr_blocks(x, f)),
        Expr::Map(pairs, _) => pairs.iter().for_each(|(k, v)| {
            expr_blocks(k, f);
            expr_blocks(v, f);
        }),
        _ => {}
    }
}

/// Every `for` loop in `b`, outer loops before the loops inside them.
fn loops<'a>(b: &'a Block, out: &mut Vec<&'a Stmt>) {
    for s in b {
        if matches!(s, Stmt::For { .. }) {
            out.push(s);
        }
        each_block(s, &mut |inner| loops(inner, out));
    }
}

/// The verdict on loop `s` of function `f` (conc.par.2-5).
fn judge(cx: &Context, f: &FnDecl, s: &Stmt) -> Verdict {
    let Context { prog, resolved, fx, .. } = *cx;
    let Stmt::For { par, var, mutable, iter, body, pos, .. } = s else { unreachable!("loops() gives `for` loops") };
    let mut scan = Scan {
        prog,
        resolved,
        var,
        range: matches!(iter, Expr::Range { .. }) && !mutable,
        scopes: Vec::new(),
        depth: 0,
        writes: Vec::new(),
        reads: Vec::new(),
        problems: Vec::new(),
        sync: &cx.sync,
        fns: &cx.fns,
    };
    scan.block(body);
    if !mutable {
        scan.channel_loop(iter);
    }
    if let Some((e, source)) = fx.of_block(resolved, body).into_iter().find(|(e, _)| ordering(e)) {
        let help = if e == "io" {
            "collect each iteration's result in a list (`out[i] = ...`) and print after the loop"
        } else {
            "move the effect out of the loop and pass its result in"
        };
        scan.problem("conc.par.2", *pos, true, format!("has effect `{e}` via {source}"), help);
    }
    let Scan { writes, reads, mut problems, .. } = scan;
    let plan = plan_of(&writes, &reads, *mutable);
    let mut roots: BTreeMap<&str, Vec<&Write>> = BTreeMap::new();
    for w in &writes {
        roots.entry(w.root).or_default().push(w);
    }
    let mut reductions = Vec::new();
    let mut elements = Vec::new();
    for (root, ws) in &roots {
        let reads_of: Vec<&Read> = reads.iter().filter(|r| r.root == *root).collect();
        if let Some(w) = ws.iter().find(|w| w.kind == Kind::Whole) {
            problems.push(Problem {
                rule: "conc.par.2",
                reason: format!("writes `{root}`, which outlives the iteration"),
                help: format!(
                    "declare `{root}` inside the loop, give each iteration its own element (`{root}[{var}]` in a range loop), or keep the loop sequential"
                ),
                at: w.pos,
                provable: true,
                handle: false,
                atomic: false,
            });
        } else if ws.iter().any(|w| w.kind == Kind::Add) {
            let at = ws[0].pos;
            if let Some(w) = ws.iter().find(|w| w.kind != Kind::Add) {
                problems.push(Problem {
                    rule: "conc.par.4",
                    reason: format!("adds to `{root}` and also writes part of it"),
                    help: format!("use a separate accumulator for `{root} +=`"),
                    at: w.pos,
                    provable: true,
                    handle: false,
                    atomic: false,
                });
            } else if let Some(r) = reads_of.first() {
                problems.push(Problem {
                    rule: "conc.par.4",
                    reason: format!("reads `{root}`, which the iterations add to"),
                    help: format!("an accumulator may only be added to; read `{root}` after the loop"),
                    at: r.pos,
                    provable: true,
                    handle: false,
                    atomic: false,
                });
            } else {
                match holds_int(f, resolved, root) {
                    Some(true) => reductions.push(*root),
                    Some(false) => problems.push(Problem {
                        rule: "conc.par.4",
                        reason: format!("adds floats into `{root}`; float `+` isn't associative, so another order changes the result"),
                        help: "collect the values in a list and call `.sum()`, which adds in a fixed tree order that can run in parallel (type.coll.12)".into(),
                        at,
                        provable: true,
                        handle: false,
                        atomic: false,
                    }),
                    None => problems.push(Problem {
                        rule: "conc.par.4",
                        reason: format!("adds to `{root}`, which the analysis can't tell is an `int`"),
                        help: format!("declare `{root}` with an `int` literal in this function"),
                        at,
                        provable: false,
                        handle: false,
                        atomic: false,
                    }),
                }
            }
        } else {
            let sigs: Vec<&Vec<String>> = ws
                .iter()
                .filter_map(|w| match &w.kind {
                    Kind::Element(s) => Some(s),
                    _ => None,
                })
                .collect();
            let unproven = ws.iter().find(|w| matches!(w.kind, Kind::Unproven(_)));
            if let (Some(w), true) = (unproven, is_ref_local(f, root)) {
                // conc.par.2: every iteration writes the one cell a `ref` names.
                problems.push(Problem {
                    rule: "conc.par.2",
                    reason: format!("writes through the `ref` `{root}`, which every iteration shares"),
                    help: format!(
                        "give each iteration its own element (`out[{var}] = ...`), keep `{root}` in a `Mutex` or `Atomic`, or drop `par`"
                    ),
                    at: w.pos,
                    provable: true,
                    handle: false,
                    atomic: false,
                });
            } else if let Some(w) = unproven {
                let Kind::Unproven(steps) = &w.kind else { unreachable!() };
                let why = if scan_range(iter, *mutable) {
                    format!("its index isn't the loop variable `{var}`")
                } else {
                    "the loop isn't over a range, so its variable may repeat".to_string()
                };
                problems.push(Problem {
                    rule: "conc.par.3",
                    reason: format!(
                        "writes `{}`, and {why}: two iterations may write the same element",
                        place_text(root, steps)
                    ),
                    help: format!("write `{root}[{var}]` from a range loop, or assert independence with `par for`"),
                    at: w.pos,
                    provable: false,
                    handle: false,
                    atomic: false,
                });
            } else if sigs.windows(2).any(|p| p[0] != p[1]) {
                problems.push(Problem {
                    rule: "conc.par.3",
                    reason: format!("writes `{root}` through different paths"),
                    help: "write one element path per iteration".into(),
                    at: ws[1].pos,
                    provable: false,
                    handle: false,
                    atomic: false,
                });
            } else {
                let sig = sigs[0];
                for r in &reads_of {
                    let ok = r.steps.len() >= sig.len() && r.steps[..sig.len()] == sig[..]
                        || r.len_only && r.steps.len() < sig.len() && r.steps[..] == sig[..r.steps.len()];
                    if ok {
                        continue;
                    }
                    let whole = r.steps.len() < sig.len();
                    problems.push(Problem {
                        rule: "conc.par.3",
                        reason: if whole {
                            format!("reads all of `{}`, which other iterations write", place_text(root, &r.steps))
                        } else {
                            format!(
                                "reads `{}` beside writing `{}`: another iteration may write it",
                                place_text(root, &r.steps),
                                place_text(root, sig)
                            )
                        },
                        help: format!("read only `{}` in the body, or keep the loop sequential", place_text(root, sig)),
                        at: r.pos,
                        provable: whole,
                        handle: false,
                        atomic: false,
                    });
                    break;
                }
                elements.push(place_text(root, sig));
            }
        }
    }
    problems.sort_by_key(|p| (!p.provable, p.at.line, p.at.col));
    let func = fn_id(prog, f);
    let verdict = |outcome, rule, reason, at, help| Verdict {
        pos: *pos,
        func: func.clone(),
        var: var.clone(),
        par: *par,
        outcome,
        rule,
        reason,
        at,
        help,
        plan: plan.clone(),
    };
    match problems.into_iter().next() {
        Some(p) => {
            let outcome = if *par && !p.provable { Outcome::Asserted } else { Outcome::Sequential };
            verdict(outcome, p.rule, p.reason, p.at, Some(p.help))
        }
        None if !reductions.is_empty() => {
            let names: Vec<String> = reductions.iter().map(|r| format!("`{r}`")).collect();
            let more = if elements.is_empty() {
                String::new()
            } else {
                format!(" and writes only `{}`", elements.join("`, `"))
            };
            verdict(
                Outcome::Parallel,
                "conc.par.4",
                format!("adds to {} in a fixed order{more}", names.join(", ")),
                *pos,
                None,
            )
        }
        None if !elements.is_empty() => {
            verdict(Outcome::Parallel, "conc.par.3", format!("writes only `{}`", elements.join("`, `")), *pos, None)
        }
        None => {
            verdict(Outcome::Parallel, "conc.par.2", "writes nothing that outlives the iteration".into(), *pos, None)
        }
    }
}

fn scan_range(iter: &Expr, mutable: bool) -> bool {
    matches!(iter, Expr::Range { .. }) && !mutable
}

/// What every loop's analysis shares.
struct Context<'a> {
    prog: &'a Program,
    resolved: &'a Resolved,
    fx: &'a Effects<'a>,
    sync: Vec<u8>,
    fns: HashMap<&'a str, usize>,
}

/// Free functions by name, and which functions synchronise through a handle.
fn context<'a>(prog: &'a Program, resolved: &'a Resolved, fx: &'a Effects<'a>) -> Context<'a> {
    let fns: HashMap<&str, usize> = prog
        .fns
        .iter()
        .enumerate()
        .filter(|(_, f)| f.owner.is_none() && f.top_level())
        .map(|(i, f)| (f.name.as_str(), i))
        .collect();
    let mut cx = Context { prog, resolved, fx, sync: vec![0; prog.fns.len()], fns };
    // A function synchronises if its body calls a handle's method, or it
    // calls a function that does: a fixed point over the call graph.
    let direct: Vec<u8> = prog
        .fns
        .iter()
        .map(|f| {
            f.body.as_ref().map_or(0, |b| {
                let mut s = Scan {
                    prog,
                    resolved,
                    var: "",
                    range: false,
                    scopes: Vec::new(),
                    depth: 0,
                    writes: Vec::new(),
                    reads: Vec::new(),
                    problems: Vec::new(),
                    sync: &[],
                    fns: &HashMap::new(),
                };
                s.block(b);
                s.problems.iter().fold(0, |acc, p| acc | if p.handle { SYNC } else { 0 } | if p.atomic { ATOMIC } else { 0 })
            })
        })
        .collect();
    cx.sync = direct;
    loop {
        let add: Vec<(usize, u8)> = (0..prog.fns.len())
            .map(|i| (i, fx.calls_of(i).iter().fold(cx.sync[i], |acc, &j| acc | cx.sync[j])))
            .filter(|&(i, flags)| flags != cx.sync[i])
            .collect();
        if add.is_empty() {
            break;
        }
        add.into_iter().for_each(|(i, flags)| cx.sync[i] = flags);
    }
    cx
}

/// `Context::sync` flags: calls a handle's method, or an atomic's.
const SYNC: u8 = 1;
const ATOMIC: u8 = 2;

/// conc.par.10: the functions a task may run on the pool: no ordering
/// effect (conc.par.2) and no handle, through every call.
pub fn pure_fns(prog: &Program, resolved: &Resolved) -> Vec<bool> {
    let fx = Effects::infer(prog, resolved);
    let cx = context(prog, resolved, &fx);
    (0..prog.fns.len()).map(|i| cx.sync[i] & SYNC == 0 && !fx.of(i).iter().any(|e| ordering(e))).collect()
}

/// The verdict on every `for` loop of the program's own functions, in
/// source order (conc.par.6).
pub fn analyze(prog: &Program, resolved: &Resolved) -> Vec<Verdict> {
    let fx = Effects::infer(prog, resolved);
    let cx = context(prog, resolved, &fx);
    let mut out = Vec::new();
    for f in prog.fns.iter().filter(|f| !is_prelude(f.pos)) {
        let Some(body) = &f.body else { continue };
        let mut found = Vec::new();
        loops(body, &mut found);
        out.extend(found.into_iter().map(|s| judge(&cx, f, s)));
    }
    out.sort_by_key(|v| (v.pos.line, v.pos.col));
    out
}

/// conc.par.5: a `par for` whose iterations provably interfere is an error.
pub fn check_par(prog: &Program, resolved: &Resolved) -> Vec<Diagnostic> {
    let any_par = prog.fns.iter().filter_map(|f| f.body.as_ref()).any(|b| {
        let mut found = Vec::new();
        loops(b, &mut found);
        found.iter().any(|s| matches!(s, Stmt::For { par: true, .. }))
    });
    if !any_par {
        return Vec::new();
    }
    analyze(prog, resolved)
        .into_iter()
        .filter(|v| v.par && v.outcome == Outcome::Sequential)
        .map(|v| {
            let d = Diagnostic::new(
                "conc.par.5",
                v.at,
                format!("`par for {}` iterations aren't independent: the body {} ({})", v.var, v.reason, v.rule),
            );
            match v.help {
                Some(h) => d.help(h),
                None => d,
            }
        })
        .collect()
}

/// `ail effects --parallel` output (conc.par.6): one line per loop, or one
/// JSON object `{"ail_api":1,"loops":[...]}`. `at` gives a position's file
/// and line.
pub fn render(verdicts: &[Verdict], json: bool, at: &dyn Fn(Pos) -> String) -> String {
    if json {
        let items: Vec<String> = verdicts
            .iter()
            .map(|v| {
                format!(
                    "{{\"at\":{},\"fn\":{},\"var\":{},\"par\":{},\"verdict\":\"{}\",\"rule\":\"{}\",\"reason\":{},\"reason_at\":{}}}",
                    json_str(&at(v.pos)),
                    json_str(&v.func),
                    json_str(&v.var),
                    v.par,
                    v.outcome.word(),
                    v.rule,
                    json_str(&v.reason),
                    json_str(&at(v.at)),
                )
            })
            .collect();
        format!("{{\"ail_api\":1,\"loops\":[{}]}}\n", items.join(","))
    } else {
        verdicts
            .iter()
            .map(|v| {
                let kw = if v.par { "par for" } else { "for" };
                let place = if v.at == v.pos { String::new() } else { format!(" at {}", at(v.at)) };
                format!(
                    "{} {} {kw} {}: {} ({}: {}{place})\n",
                    at(v.pos),
                    v.func,
                    v.var,
                    v.outcome.word(),
                    v.rule,
                    v.reason
                )
            })
            .collect()
    }
}

/// Calls `f` with every `[e par for x in xs]` comprehension in `b`, in
/// source order (conc.par.12).
fn par_comps<'a>(b: &'a Block, f: &mut dyn FnMut(&'a Expr, &'a CompFor)) {
    fn walk<'a>(e: &'a Expr, f: &mut dyn FnMut(&'a Expr, &'a CompFor)) {
        match e {
            Expr::Comp { elem, value, clauses, .. } => {
                if let [c] = clauses.as_slice()
                    && c.par
                {
                    f(elem, c);
                }
                for c in clauses {
                    walk(&c.iter, f);
                    c.cond.iter().for_each(|x| walk(x, f));
                }
                walk(elem, f);
                value.iter().for_each(|v| walk(v, f));
            }
            Expr::Interp(parts, _) => parts.iter().for_each(|p| {
                if let InterpPart::Expr(x) | InterpPart::Fmt(x, _) = p {
                    walk(x, f);
                }
            }),
            Expr::List(xs, _) | Expr::Set(xs, _) | Expr::Tuple(xs, _) => xs.iter().for_each(|x| walk(x, f)),
            Expr::Map(kvs, _) => kvs.iter().for_each(|(k, v)| {
                walk(k, f);
                walk(v, f);
            }),
            Expr::Call { callee, args, .. } => {
                walk(callee, f);
                args.iter().for_each(|a| walk(&a.value, f));
            }
            Expr::Method { recv, args, .. } => {
                walk(recv, f);
                args.iter().for_each(|a| walk(&a.value, f));
            }
            Expr::Field { recv: x, .. }
            | Expr::Unary { operand: x, .. }
            | Expr::Try { expr: x, .. }
            | Expr::Ref { value: x, .. } => walk(x, f),
            Expr::Index { recv: a, index: b, .. } | Expr::Binary { lhs: a, rhs: b, .. } => {
                walk(a, f);
                walk(b, f);
            }
            Expr::Range { lo, hi, .. } => [lo, hi].into_iter().flatten().for_each(|x| walk(x, f)),
            Expr::If { branches, els, .. } => {
                for (cond, body) in branches {
                    let (Cond::Expr(x) | Cond::Bind { value: x, .. }) = cond;
                    walk(x, f);
                    block(body, f);
                }
                els.iter().for_each(|b| block(b, f));
            }
            Expr::Match { scrutinee, arms, .. } => {
                walk(scrutinee, f);
                for a in arms {
                    if let Some(g) = &a.guard {
                        walk(g, f);
                    }
                    block(&a.body, f);
                }
            }
            Expr::Else { value, otherwise, .. } => {
                walk(value, f);
                other(otherwise, f);
            }
            _ => {}
        }
    }
    fn other<'a>(o: &'a Else, f: &mut dyn FnMut(&'a Expr, &'a CompFor)) {
        match o {
            Else::Value(x) => walk(x, f),
            Else::Leave(s) => stmt(s, f),
            Else::Block { body, .. } => block(body, f),
        }
    }
    fn stmt<'a>(s: &'a Stmt, f: &mut dyn FnMut(&'a Expr, &'a CompFor)) {
        match s {
            Stmt::Let { value, otherwise, .. } => {
                walk(value, f);
                otherwise.iter().for_each(|o| other(o, f));
            }
            Stmt::Assign { target, value, .. } => {
                walk(target, f);
                walk(value, f);
            }
            Stmt::While { cond: e, body, .. } | Stmt::For { iter: e, body, .. } | Stmt::With { mutex: e, body, .. } => {
                walk(e, f);
                block(body, f);
            }
            Stmt::Return(Some(e), _) | Stmt::Expr(e) | Stmt::LetTuple { value: e, .. } => walk(e, f),
            Stmt::Handle { value, otherwise } => {
                walk(value, f);
                other(otherwise, f);
            }
            Stmt::Unsafe { body, .. } => block(body, f),
            Stmt::Defer(inner, _) => stmt(inner, f),
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return(None, _) => {}
        }
    }
    fn block<'a>(b: &'a Block, f: &mut dyn FnMut(&'a Expr, &'a CompFor)) {
        b.iter().for_each(|s| stmt(s, f));
    }
    block(b, f);
}

/// The verdict on `[elem par for c.var in c.iter if c.cond]` (conc.par.12):
/// each element is computed as an iteration of a loop that writes nothing
/// outside it. Its `pos` is the clause variable's.
fn judge_comp(cx: &Context, f: &FnDecl, elem: &Expr, c: &CompFor) -> Verdict {
    let Context { prog, resolved, fx, .. } = *cx;
    let mut scan = Scan {
        prog,
        resolved,
        var: &c.var,
        range: false,
        scopes: vec![Vec::new()],
        depth: 0,
        writes: Vec::new(),
        reads: Vec::new(),
        problems: Vec::new(),
        sync: &cx.sync,
        fns: &cx.fns,
    };
    scan.channel_loop(&c.iter);
    if let Some(cond) = &c.cond {
        scan.expr(cond);
    }
    scan.expr(elem);
    let mut effects = fx.of_expr(resolved, elem);
    if let Some(cond) = &c.cond {
        effects.extend(fx.of_expr(resolved, cond));
    }
    if let Some((e, source)) = effects.into_iter().find(|(e, _)| ordering(e)) {
        let help = if e == "io" { "compute the values with `par for`, then print them after" } else { "move the effect out of the comprehension" };
        scan.problem("conc.par.2", elem.pos(), true, format!("has effect `{e}` via {source}"), help);
    }
    let Scan { writes, reads, mut problems, .. } = scan;
    for w in &writes {
        problems.push(Problem {
            rule: "conc.par.12",
            reason: format!("writes `{}`, which outlives the element", w.root),
            help: "an element of a `par for` comprehension only computes a value".into(),
            at: w.pos,
            provable: true,
            handle: false,
            atomic: false,
        });
    }
    problems.sort_by_key(|p| (!p.provable, p.at.line, p.at.col));
    let mut captures: Vec<&str> = reads.iter().map(|r| r.root).collect();
    captures.sort_unstable();
    captures.dedup();
    let plan = Plan { captures: captures.into_iter().map(String::from).collect(), ..Plan::default() };
    let (outcome, rule, reason, at, help) = match problems.into_iter().next() {
        Some(p) => (if p.provable { Outcome::Sequential } else { Outcome::Asserted }, p.rule, p.reason, p.at, Some(p.help)),
        None => (Outcome::Parallel, "conc.par.12", "computes each element independently".into(), c.var_pos, None),
    };
    Verdict { pos: c.var_pos, func: fn_id(prog, f), var: c.var.clone(), par: true, outcome, rule, reason, at, help, plan: Some(plan) }
}

/// The verdict on every `par for` comprehension, in source order (conc.par.12).
pub fn analyze_comps(prog: &Program, resolved: &Resolved) -> Vec<Verdict> {
    let mut found = Vec::new();
    for f in prog.fns.iter().filter(|f| !is_prelude(f.pos)) {
        if let Some(body) = &f.body {
            par_comps(body, &mut |e, c| found.push((f, e, c)));
        }
    }
    if found.is_empty() {
        return Vec::new();
    }
    let fx = Effects::infer(prog, resolved);
    let cx = context(prog, resolved, &fx);
    let mut out: Vec<Verdict> = found.into_iter().map(|(f, e, c)| judge_comp(&cx, f, e, c)).collect();
    out.sort_by_key(|v| (v.pos.line, v.pos.col));
    out
}

/// conc.par.12: a `par for` comprehension whose elements provably
/// interfere, or synchronise, is an error.
pub fn check_par_comps(prog: &Program, resolved: &Resolved) -> Vec<Diagnostic> {
    analyze_comps(prog, resolved)
        .into_iter()
        .filter(|v| v.outcome == Outcome::Sequential)
        .map(|v| {
            let d = Diagnostic::new(
                "conc.par.12",
                v.at,
                format!("`par for {}` elements aren't independent: the element {} ({})", v.var, v.reason, v.rule),
            );
            match v.help {
                Some(h) => d.help(h),
                None => d,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{check::check, lexer::lex, parser::parse_with_prelude};

    /// Each loop's `line outcome rule` for `body`, the body of `main`.
    fn verdicts(body: &str) -> Vec<String> {
        let src = format!("fn id(x int) int\n    x\nfn main()\n{body}");
        let (tokens, ld) = lex(&src);
        assert!(ld.is_empty(), "{ld:?}");
        let (prog, pd) = parse_with_prelude(tokens);
        assert!(pd.is_empty(), "{pd:?}");
        let (diags, resolved) = check(&prog);
        assert!(diags.iter().all(|d| d.rule == "conc.par.5"), "{diags:?}");
        analyze(&prog, &resolved)
            .iter()
            .map(|v| format!("{} {} {}", v.pos.line - 3, v.outcome.word(), v.rule))
            .collect()
    }

    #[test]
    fn element_writes_by_the_loop_variable_are_independent() {
        let body = "    mut xs := [1, 2, 3]\n    for i in 0..3\n        xs[i] = id(xs[i]) * 2\n    for i in 0..3\n        xs[i] = xs[2 - i]\n";
        assert_eq!(verdicts(body), ["2 parallel conc.par.3", "4 sequential conc.par.3"]);
    }

    #[test]
    fn locals_and_nested_loops() {
        let body = "    mut g := [[0, 0], [0, 0]]\n    for i in 0..2\n        mut t := 0\n        for j in 0..2\n            t += j\n            g[i][j] = t\n";
        assert_eq!(verdicts(body), ["2 parallel conc.par.3", "4 sequential conc.par.4"]);
    }

    #[test]
    fn reductions_need_a_known_int() {
        let body = "    xs := [1, 2]\n    mut a := 0\n    mut b := xs.len()\n    for x in xs\n        a += x\n    for x in xs\n        b += x\n    for x in xs\n        a += a\n    print(a + b)\n";
        assert_eq!(verdicts(body), ["4 parallel conc.par.4", "6 sequential conc.par.4", "8 sequential conc.par.4"]);
    }

    #[test]
    fn par_asserts_only_what_cannot_be_proven() {
        let body = "    p := [1, 0]\n    mut xs := [0, 0]\n    par for i in 0..2\n        xs[p[i]] = i\n    mut b := xs.len()\n    par for x in xs\n        b += x\n";
        assert_eq!(verdicts(body), ["3 asserted conc.par.3", "6 asserted conc.par.4"]);
    }

    #[test]
    fn exits_order_the_loop() {
        let body = "    for i in 0..3\n        while true\n            break\n        if i == 1\n            continue\n    for i in 0..3\n        if i == 1\n            return\n";
        assert_eq!(verdicts(body), ["1 parallel conc.par.2", "6 sequential conc.par.2"]);
    }

    #[test]
    fn handles_order_the_loop() {
        let src = "fn bump(m Mutex[int])\n    m.set(m.get() + 1)\nfn main()\n    m := Mutex(0)\n    for i in 0..3\n        m.set(i)\n    for i in 0..3\n        bump(m)\n    for i in 0..3\n        xs := [x * i for x in 0..3 if x > 0]\n        print(xs.len())\n    print(m.get())\n";
        let (tokens, _) = lex(src);
        let (prog, _) = parse_with_prelude(tokens);
        let (diags, resolved) = check(&prog);
        assert!(diags.is_empty(), "{diags:?}");
        let got: Vec<String> =
            analyze(&prog, &resolved).iter().map(|v| format!("{} {}", v.outcome.word(), v.reason)).collect();
        assert_eq!(
            got,
            [
                "sequential calls `.set()` on a task, channel or mutex handle",
                "sequential calls `bump`, which uses a task, channel or mutex handle",
                "sequential has effect `io` via print",
            ]
        );
    }
}
