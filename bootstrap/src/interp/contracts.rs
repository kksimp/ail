//! Runtime contract checks (contract.req.1, contract.ens.1, contract.run.1-3,
//! ADR 0211-0212).

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

use super::*;
use crate::ast::{Contract, Expr, Program, Within};

/// What changed a struct that now breaks an invariant.
#[derive(Clone, Copy)]
pub(super) enum Culprit<'a> {
    Built,
    Method(&'a FnDecl),
}

/// `--no-contracts` turns the checks off (contract.run.3).
static ENABLED: AtomicBool = AtomicBool::new(true);

thread_local! {
    /// A condition is being evaluated: calls it makes aren't checked
    /// (contract.run.2), so `ensures result == f(n - 1) * n` can't recurse.
    static CHECKING: Cell<bool> = const { Cell::new(false) };
}

pub fn set_enabled(on: bool) {
    ENABLED.store(on, AtomicOrdering::Relaxed);
}

/// Whether `f`'s contracts are checked on this call (contract.run.1-3).
#[inline]
pub(super) fn active(f: &FnDecl) -> bool {
    (!f.contracts.is_empty() || f.inherits.is_some()) && ENABLED.load(AtomicOrdering::Relaxed) && !CHECKING.with(Cell::get)
}

/// Where a condition comes from: the function's own line, or one inherited
/// from the trait method it implements (contract.inh.1-2).
type Source<'a> = Option<&'a FnDecl>;

impl<'a, W: Write> Interp<'a, W> {
    /// `f`'s `requires` or `ensures` conditions in checking order: the
    /// trait's first (contract.inh.1-2), then its own.
    fn conditions(&self, f: &'a FnDecl, ensures: bool) -> Vec<(&'a Contract, Source<'a>)> {
        let prog: &'a Program = self.prog;
        let inherited = f.inherits.map(|m| &prog.fns[m]);
        let theirs = inherited.into_iter().flat_map(|t| t.contracts.iter().map(move |c| (c, Some(t))));
        let own = f.contracts.iter().map(|c| (c, None));
        theirs.chain(own).filter(|(c, _)| c.ensures == ensures).collect()
    }

    /// The receiver and parameters of `f` under the trait method's names, so
    /// an inherited condition reads them (contract.inh.3).
    fn renamed(&self, f: &FnDecl, from: &'a FnDecl, env: &Env<'a>) -> Env<'a> {
        let mut out: Env<'a> = Vec::with_capacity(env.len() + 2);
        let skip = usize::from(f.receiver.is_some());
        if from.receiver.is_some() {
            out.push(("self", env[0].1.clone()));
        }
        for (k, p) in from.params.iter().enumerate() {
            out.push((p.name.as_str(), env[skip + k].1.clone()));
        }
        out
    }

    /// contract.req.1: each `requires` holds once the parameters are bound.
    /// A failure is the caller's: it traps at the call. An implementation's
    /// own `requires` lines weaken the trait's (contract.inh.1): the call is
    /// fine if either set holds.
    #[inline(never)]
    pub(super) fn check_requires(&mut self, f: &'a FnDecl, env: &mut Env<'a>, call: Pos) -> Result<(), Trap> {
        let list = self.conditions(f, false);
        let inherited: Vec<_> = list.iter().filter(|(_, from)| from.is_some()).collect();
        let own_count = list.len() - inherited.len();
        let mut failed: Option<(&'a Contract, Source<'a>)> = None;
        // The trait's conditions, then the function's own; the call is fine
        // when all of one group hold, and a group with no lines holds.
        let mut group_ok = [true, true];
        for &(c, from) in &list {
            let slot = usize::from(from.is_none());
            let held = match from {
                Some(t) => {
                    let mut scope = self.renamed(f, t, env);
                    self.holds(c, &mut scope)?
                }
                None => self.holds(c, env)?,
            };
            if self.tracing & super::trace::CALLS != 0 {
                self.trace_contract(f, c, held);
            }
            if !held {
                if failed.is_none() || (slot == 0 && failed.is_some_and(|(_, s)| s.is_none())) {
                    failed = Some((c, from));
                }
                group_ok[slot] = false;
                if inherited.is_empty() || own_count == 0 {
                    break;
                }
            }
        }
        let weakened = !inherited.is_empty() && own_count > 0;
        let ok = if weakened { group_ok[0] || group_ok[1] } else { group_ok[0] && group_ok[1] };
        if ok {
            return Ok(());
        }
        let (c, from) = failed.expect("a failed group has a failed condition");
        match from {
            Some(t) => {
                let mut scope = self.renamed(f, t, env);
                self.holds(c, &mut scope)?;
                self.snap_condition(&c.cond, &scope);
            }
            None => self.snap_condition(&c.cond, env),
        }
        let args = self.arguments(f, env, call)?;
        let origin = from.map_or(String::new(), |t| format!(" (inherited from `{}`)", self.label(t)));
        let also = if weakened { ", and neither do its own `requires` lines" } else { "" };
        let message = format!(
            "`{}` requires `{}`{origin}, but it was called with {args}{also} (contract at {})",
            self.label(f),
            c.text,
            self.where_is(c.pos, call)
        );
        trap("contract.req.1", call, message)
    }

    /// contract.ens.3: the values `old(e)` names, evaluated on entry for
    /// every `ensures` condition that has any, in checking order.
    #[inline(never)]
    pub(super) fn entry_olds(&mut self, f: &'a FnDecl, env: &mut Env<'a>) -> Result<Vec<Value>, Trap> {
        let mut out = Vec::new();
        for (c, from) in self.conditions(f, true) {
            if c.olds.is_empty() {
                continue;
            }
            let mut scope = from.map(|t| self.renamed(f, t, env));
            for (_, e) in &c.olds {
                let target: &mut Env<'a> = match scope.as_mut() {
                    Some(s) => s,
                    None => &mut *env,
                };
                out.push(self.eval_pure(e, target, c.pos)?);
            }
        }
        Ok(out)
    }

    /// contract.ens.1: each `ensures` holds for a success value, with
    /// `result` bound to it and each `old(e)` to its value on entry. A
    /// failure is the function's: it traps at the `ensures` line and names
    /// where the function returned. An implementation's lines add to the
    /// trait's (contract.inh.2).
    #[inline(never)]
    pub(super) fn check_ensures(
        &mut self,
        f: &'a FnDecl,
        env: &mut Env<'a>,
        result: &Value,
        at: Pos,
        olds: &[Value],
    ) -> Result<(), Trap> {
        if matches!(result, Value::Fail(_)) {
            return Ok(());
        }
        let mut next = 0;
        for (c, from) in self.conditions(f, true) {
            let mine = &olds[next..next + c.olds.len()];
            next += c.olds.len();
            let mut scope = from.map(|t| self.renamed(f, t, env));
            let target: &mut Env<'a> = match scope.as_mut() {
                Some(s) => s,
                None => &mut *env,
            };
            let n = target.len();
            if !matches!(result, Value::Unit) {
                target.push(("result", result.clone()));
            }
            for ((name, _), v) in c.olds.iter().zip(mine) {
                target.push((name.as_str(), v.clone()));
            }
            let held = self.holds(c, target);
            if self.tracing & super::trace::CALLS != 0 {
                self.trace_contract(f, c, matches!(held, Ok(true)));
            }
            if !matches!(held, Ok(false)) {
                target.truncate(n);
                held?;
                continue;
            }
            self.snap_condition(&c.cond, target);
            target.truncate(n);
            let value = if matches!(result, Value::Unit) {
                String::new()
            } else {
                format!(" {}", self.describe(result, c.pos)?)
            };
            let args = self.arguments(f, env, c.pos)?;
            let origin = from.map_or(String::new(), |t| format!(" (inherited from `{}`)", self.label(t)));
            let message = format!(
                "`{}` ensures `{}`{origin}, but it returned{value} at {} with {args}",
                self.label(f),
                c.text,
                self.where_is(at, c.pos)
            );
            return trap("contract.ens.1", c.pos, message);
        }
        Ok(())
    }

    /// contract.inv.2-3: every `invariant` of struct `s` holds for `value`,
    /// which `by` just built or changed.
    #[inline(never)]
    pub(super) fn check_invariants(&mut self, s: usize, value: &Value, by: Culprit<'a>, at: Pos) -> Result<(), Trap> {
        let prog: &'a Program = self.prog;
        let decl = &prog.structs[s];
        for c in &decl.invariants {
            let mut scope: Env<'a> = vec![("self", value.clone())];
            let held = self.holds(c, &mut scope)?;
            if self.tracing & super::trace::CALLS != 0 {
                self.trace_invariant(&decl.name, c, held);
            }
            if held {
                continue;
            }
            self.snap_condition(&c.cond, &scope);
            let shown = self.describe(value, c.pos)?;
            let (rule, message) = match by {
                Culprit::Built => (
                    "contract.inv.2",
                    format!(
                        "`{}` invariant `{}` doesn't hold for {shown}, built at {} (invariant at {})",
                        decl.name,
                        c.text,
                        self.where_is(at, c.pos),
                        self.where_is(c.pos, at)
                    ),
                ),
                Culprit::Method(f) => (
                    "contract.inv.3",
                    format!(
                        "`{}` invariant `{}` was broken by `{}`, which returned at {} with self = {shown} (invariant at {})",
                        decl.name,
                        c.text,
                        self.label(f),
                        self.where_is(at, c.pos),
                        self.where_is(c.pos, at)
                    ),
                ),
            };
            return trap(rule, c.pos, message);
        }
        Ok(())
    }

    /// Whether invariants are checked now: checks are on and no condition is
    /// being evaluated (contract.run.2).
    #[inline]
    pub(super) fn invariants_active(&self) -> bool {
        self.has_invariants && ENABLED.load(AtomicOrdering::Relaxed) && !CHECKING.with(Cell::get)
    }

    fn eval_pure(&mut self, e: &'a Expr, env: &mut Env<'a>, pos: Pos) -> Result<Value, Trap> {
        let outer = CHECKING.with(|k| k.replace(true));
        let v = self.eval(e, env);
        CHECKING.with(|k| k.set(outer));
        match v {
            Ok(v) => Ok(v),
            Err(Exit::Trap(t)) => Err(*t),
            Err(_) => trap("contract.decl.2", pos, "the condition left the function".into()),
        }
    }

    /// Evaluate a condition with checks off for the calls inside it (contract.run.2).
    fn holds(&mut self, c: &'a Contract, env: &mut Env<'a>) -> Result<bool, Trap> {
        Ok(self.eval_pure(&c.cond, env, c.pos)?.bool())
    }

    /// `LINE:COL` of `pos`, or `PATH:LINE:COL` when it is in another file
    /// of the package than `from`.
    fn where_is(&self, pos: Pos, from: Pos) -> String {
        let Some(file) = self.prog.file_at(pos) else { return format!("{}:{}", pos.line, pos.col) };
        let line = pos.line - file.base;
        match self.prog.file_at(from) {
            Some(other) if other.base == file.base => format!("{line}:{}", pos.col),
            _ => format!("{}:{line}:{}", file.path, pos.col),
        }
    }

    /// `Type.name` for a method, also one written in an `impl` or `trait` block.
    fn label(&self, f: &FnDecl) -> String {
        match f.within {
            Within::Impl(k) => format!("{}.{}", self.prog.impls[k].target.name, f.name),
            Within::Trait(t) => format!("{}.{}", self.prog.traits[t].name, f.name),
            Within::TopLevel => f.full_name(),
        }
    }

    /// `a = 7, b = 0`: the receiver and parameters with their text forms.
    fn arguments(&mut self, f: &FnDecl, env: &Env<'a>, pos: Pos) -> Result<String, Trap> {
        let mut parts = Vec::with_capacity(env.len());
        for (name, v) in env.iter().take(f.params.len() + usize::from(f.receiver.is_some())) {
            parts.push(format!("{name} = {}", self.describe(v, pos)?));
        }
        Ok(if parts.is_empty() { "no arguments".into() } else { parts.join(", ") })
    }
}
