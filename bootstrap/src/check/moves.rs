//! Use after move (own.uniq.2, own.move.2, own.move.4, own.defer.3): a
//! flow-sensitive walk over each function that follows which bindings have
//! been moved out on every path. The checker has already recorded, in
//! `Resolved::moves`, each name whose use moves its value (ADR 0190).

use crate::fasthash::FastMap as HashMap;

use super::Resolved;
use crate::ast::{Block, Cond, Else, Expr, InterpPart, Pattern, Program, Stmt};
use crate::diag::{Diagnostic, Pos};

/// Where a binding was moved, by which rule, and whether every path to
/// here moved it.
#[derive(Clone, Copy, PartialEq)]
struct Moved {
    at: Pos,
    rule: &'static str,
    certain: bool,
}

/// The moved bindings on the current path; `None` when no path reaches here.
type State = Option<HashMap<String, Moved>>;

fn join(a: State, b: State) -> State {
    match (a, b) {
        (None, s) | (s, None) => s,
        (Some(mut a), Some(b)) => {
            for (n, m) in a.iter_mut() {
                m.certain &= b.get(n).is_some_and(|mb| mb.certain);
            }
            for (n, mb) in b {
                a.entry(n).or_insert(Moved { certain: false, ..mb });
            }
            Some(a)
        }
    }
}

struct Loop {
    breaks: State,
    continues: State,
    /// How many names were in scope when the loop started.
    outer: usize,
}

struct Walker<'p> {
    moves: &'p HashMap<Pos, &'static str>,
    state: State,
    /// Names in scope, innermost last (names aren't shadowed: expr.bind.4).
    scope: Vec<&'p str>,
    loops: Vec<Loop>,
    /// Inside a deferred statement (own.defer.3).
    deferred: bool,
    diags: Vec<Diagnostic>,
}

pub fn check(prog: &Program, resolved: &Resolved) -> Vec<Diagnostic> {
    if resolved.moves.is_empty() {
        return Vec::new();
    }
    let moves: HashMap<Pos, &'static str> = resolved.moves.iter().map(|(p, r)| (*p, *r)).collect();
    let mut w = Walker { moves: &moves, state: None, scope: Vec::new(), loops: Vec::new(), deferred: false, diags: Vec::new() };
    for f in &prog.fns {
        let Some(body) = &f.body else { continue };
        w.state = Some(HashMap::default());
        w.scope = f.params.iter().map(|p| p.name.as_str()).collect();
        w.block(body);
    }
    w.diags
}

impl<'p> Walker<'p> {
    fn block(&mut self, b: &'p Block) {
        let mark = self.scope.len();
        let mut deferred: Vec<&'p Stmt> = Vec::new();
        for s in b {
            if let Stmt::Defer(inner, _) = s {
                deferred.push(inner);
            }
            self.stmt(s);
        }
        // own.defer.3: a deferred statement runs at the end of the block, so
        // what it uses must still be there.
        for s in deferred {
            let saved = self.deferred;
            self.deferred = true;
            self.uses_only(s);
            self.deferred = saved;
        }
        for n in self.scope.drain(mark..) {
            if let Some(s) = &mut self.state {
                s.remove(n);
            }
        }
    }

    /// Check the uses in `s` against the current state without moving anything.
    fn uses_only(&mut self, s: &'p Stmt) {
        let saved = self.state.clone();
        self.stmt(s);
        self.state = saved;
    }

    fn declare(&mut self, n: &'p str) {
        self.scope.push(n);
        if let Some(s) = &mut self.state {
            s.remove(n);
        }
    }

    fn stmt(&mut self, s: &'p Stmt) {
        match s {
            Stmt::Let { name, value, otherwise, .. } => {
                self.expr(value);
                if let Some(o) = otherwise {
                    self.otherwise(o);
                }
                self.declare(name);
            }
            Stmt::Assign { target, op, value, .. } => {
                self.expr(value);
                match target {
                    // A whole binding gets a new value; `x += 1` reads it first.
                    Expr::Name(n, _) => {
                        if op.is_some() {
                            self.expr(target);
                        }
                        if let Some(st) = &mut self.state {
                            st.remove(n.as_str());
                        }
                    }
                    _ => self.expr(target),
                }
            }
            Stmt::While { cond, body, .. } => {
                self.expr(cond);
                self.loop_body(body, None);
            }
            Stmt::Unsafe { body, .. } => self.block(body),
            Stmt::With { mutex, body, .. } => {
                self.expr(mutex);
                self.block(body);
            }
            Stmt::For { var, iter, body, .. } => {
                self.expr(iter);
                self.loop_body(body, Some(var));
            }
            Stmt::Break(_) => {
                let st = self.state.take();
                if let Some(l) = self.loops.last_mut() {
                    l.breaks = join(l.breaks.take(), st);
                }
            }
            Stmt::Continue(_) => {
                let st = self.state.take();
                if let Some(l) = self.loops.last_mut() {
                    l.continues = join(l.continues.take(), st);
                }
            }
            Stmt::Return(value, _) => {
                if let Some(e) = value {
                    self.expr(e);
                }
                self.state = None;
            }
            Stmt::Expr(e) => self.expr(e),
            Stmt::Handle { value, otherwise } => {
                self.expr(value);
                self.otherwise(otherwise);
            }
            // Checked at the end of the block, with the state there.
            Stmt::Defer(..) => {}
        }
    }

    /// A loop body runs any number of times: a binding from outside the loop
    /// that is moved on the way back to the top would be used again by the
    /// next iteration (own.uniq.2, own.move.2).
    fn loop_body(&mut self, body: &'p Block, var: Option<&'p str>) {
        let entry = self.state.clone();
        let mark = self.scope.len();
        if let Some(v) = var {
            self.declare(v);
        }
        self.loops.push(Loop { breaks: None, continues: None, outer: mark });
        self.block(body);
        let l = self.loops.pop().expect("the loop");
        let back = join(self.state.take(), l.continues);
        self.scope.truncate(mark);
        if let Some(back) = &back {
            let outer: Vec<&str> = self.scope[..l.outer].to_vec();
            let mut again: Vec<(&String, &Moved)> = back
                .iter()
                .filter(|(n, _)| outer.contains(&n.as_str()))
                .filter(|(n, _)| !entry.as_ref().is_some_and(|e| e.contains_key(*n)))
                .collect();
            again.sort_by_key(|(_, m)| (m.at.line, m.at.col));
            for (n, m) in again {
                self.diags.push(
                    Diagnostic::new("own.move.4", m.at, format!("`{n}` is moved inside a loop, so the next iteration would use it after the move"))
                        .help(format!("give `{n}` a new value before the loop repeats, or `break` after the move")),
                );
            }
        }
        let mut after = join(entry, l.breaks);
        after = join(after, back);
        if let Some(s) = &mut after {
            if let Some(v) = var {
                s.remove(v);
            }
        }
        self.state = after;
    }

    fn otherwise(&mut self, o: &'p Else) {
        let before = self.state.clone();
        match o {
            Else::Value(e) => self.expr(e),
            Else::Leave(s) => self.stmt(s),
            Else::Block { err, body, .. } => {
                let mark = self.scope.len();
                if let Some((n, _)) = err {
                    self.declare(n);
                }
                self.block(body);
                self.scope.truncate(mark);
            }
        }
        // The fallback runs only when the value is absent or failed.
        self.state = join(before, self.state.take());
    }

    fn use_name(&mut self, n: &'p str, pos: Pos) {
        let Some(st) = &mut self.state else { return };
        if let Some(m) = st.remove(n) {
            let (msg, note) = if self.deferred {
                (format!("the deferred statement uses `{n}`, which is moved before the block ends"), "moved here")
            } else if m.certain {
                (format!("`{n}` is used after it was moved"), "moved here")
            } else {
                (format!("`{n}` may have been moved on some paths to here"), "moved here on some paths")
            };
            // own.move.4: a move on only some paths still ends the binding.
            let rule = if self.deferred {
                "own.defer.3"
            } else if m.certain {
                m.rule
            } else {
                "own.move.4"
            };
            self.diags.push(
                Diagnostic::new(rule, pos, msg)
                    .note(m.at, note)
                    .help(format!("use `{n}` before the move, or give it a new value with `{n} = ...` first")),
            );
            return;
        }
        if let Some(&rule) = self.moves.get(&pos) {
            if self.deferred {
                self.diags.push(
                    Diagnostic::new("own.defer.3", pos, format!("a deferred statement can't move `{n}`"))
                        .help("move it in an ordinary statement at the end of the block"),
                );
                return;
            }
            st.insert(n.to_string(), Moved { at: pos, rule, certain: true });
        }
    }

    fn exprs(&mut self, es: &'p [Expr]) {
        for e in es {
            self.expr(e);
        }
    }

    fn expr(&mut self, e: &'p Expr) {
        match e {
            Expr::Str(..)
            | Expr::Int(..)
            | Expr::Float(..)
            | Expr::Char(..)
            | Expr::Bool(..)
            | Expr::None(_)
            | Expr::Hole(_)
            | Expr::TypeArgs { .. } => {}
            Expr::Name(n, pos) => self.use_name(n, *pos),
            Expr::Interp(parts, _) => {
                for p in parts {
                    if let InterpPart::Expr(x) | InterpPart::Fmt(x, _) = p {
                        self.expr(x);
                    }
                }
            }
            Expr::List(items, _) | Expr::Set(items, _) => self.exprs(items),
            Expr::Map(pairs, _) => {
                for (k, v) in pairs {
                    self.expr(k);
                    self.expr(v);
                }
            }
            Expr::Call { callee, args, .. } => {
                if !matches!(**callee, Expr::Name(..)) {
                    self.expr(callee);
                }
                for a in args {
                    self.expr(&a.value);
                }
            }
            Expr::Method { recv, args, .. } => {
                self.expr(recv);
                for a in args {
                    self.expr(&a.value);
                }
            }
            Expr::Field { recv, .. } | Expr::Unary { operand: recv, .. } | Expr::Ref { value: recv, .. } => {
                self.expr(recv)
            }
            Expr::Index { recv, index, .. } => {
                self.expr(recv);
                self.expr(index);
            }
            Expr::Binary { op, lhs, rhs, .. } => {
                self.expr(lhs);
                if matches!(op, crate::ast::BinOp::And | crate::ast::BinOp::Or) {
                    let before = self.state.clone();
                    self.expr(rhs);
                    self.state = join(before, self.state.take());
                } else {
                    self.expr(rhs);
                }
            }
            Expr::Range { lo, hi, .. } => {
                for x in [lo, hi].into_iter().flatten() {
                    self.expr(x);
                }
            }
            Expr::If { branches, els, .. } => {
                let mut out: State = None;
                for (cond, body) in branches {
                    let mark = self.scope.len();
                    match cond {
                        Cond::Expr(c) => self.expr(c),
                        Cond::Bind { value, .. } => self.expr(value),
                    }
                    let after_cond = self.state.clone();
                    if let Cond::Bind { name, .. } = cond {
                        self.declare(name);
                    }
                    self.block(body);
                    self.scope.truncate(mark);
                    out = join(out, self.state.take());
                    self.state = after_cond;
                }
                match els {
                    Some(b) => {
                        self.block(b);
                        self.state = join(out, self.state.take());
                    }
                    None => self.state = join(out, self.state.take()),
                }
            }
            Expr::Match { scrutinee, arms, .. } => {
                self.expr(scrutinee);
                let start = self.state.take();
                let mut out: State = None;
                for arm in arms {
                    self.state = start.clone();
                    let mark = self.scope.len();
                    self.pattern(&arm.pat);
                    self.block(&arm.body);
                    self.scope.truncate(mark);
                    out = join(out, self.state.take());
                }
                self.state = if arms.is_empty() { start } else { out };
            }
            Expr::Try { expr, context, .. } => {
                self.expr(expr);
                // The context is evaluated only on the way out of the function.
                if let Some(c) = context {
                    let saved = self.state.clone();
                    self.expr(c);
                    self.state = saved;
                }
            }
            Expr::Else { value, otherwise, .. } => {
                self.expr(value);
                self.otherwise(otherwise);
            }
            Expr::Comp { elem, value, clauses, .. } => self.comp(elem, value.as_deref(), clauses),
        }
    }

    /// A comprehension (expr.comp.1): the first source is evaluated once and
    /// the rest repeats, like a loop body, so a move of an outer binding in
    /// it would repeat too (own.move.4).
    fn comp(&mut self, elem: &'p Expr, value: Option<&'p Expr>, clauses: &'p [crate::ast::CompFor]) {
        let mark = self.scope.len();
        let mut entry = None;
        for (i, c) in clauses.iter().enumerate() {
            self.expr(&c.iter);
            if i == 0 {
                entry = self.state.clone();
            }
            self.declare(&c.var);
            if let Some(cond) = &c.cond {
                self.expr(cond);
            }
        }
        self.expr(elem);
        if let Some(v) = value {
            self.expr(v);
        }
        self.scope.truncate(mark);
        if let Some(st) = &mut self.state {
            clauses.iter().for_each(|c| _ = st.remove(&c.var));
        }
        let (Some(entry), Some(now)) = (entry, &self.state) else { return };
        let mut again: Vec<(&String, &Moved)> = now.iter().filter(|(n, _)| !entry.contains_key(*n)).collect();
        again.sort_by_key(|(_, m)| (m.at.line, m.at.col));
        for (n, m) in again {
            self.diags.push(
                Diagnostic::new("own.move.4", m.at, format!("`{n}` is moved inside a comprehension, so the next element would use it after the move"))
                    .help(format!("move `{n}` outside the comprehension")),
            );
        }
    }

    fn pattern(&mut self, p: &'p Pattern) {
        match p {
            Pattern::Variant { bindings, .. } => {
                for (n, _) in bindings.iter().flatten() {
                    self.declare(n);
                }
            }
            Pattern::Or(alts, _) => alts.iter().for_each(|a| self.pattern(a)),
            Pattern::Wildcard(_) | Pattern::Literal(..) => {}
        }
    }
}
