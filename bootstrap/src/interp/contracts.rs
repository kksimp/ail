//! Runtime contract checks (contract.req.1, contract.ens.1, contract.run.1-3,
//! ADR 0211-0212).

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

use super::*;
use crate::ast::Contract;

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
    !f.contracts.is_empty() && ENABLED.load(AtomicOrdering::Relaxed) && !CHECKING.with(Cell::get)
}

impl<'a, W: Write> Interp<'a, W> {
    /// contract.req.1: each `requires` holds once the parameters are bound.
    /// A failure is the caller's: it traps at the call.
    #[inline(never)]
    pub(super) fn check_requires(&mut self, f: &'a FnDecl, env: &mut Env<'a>, call: Pos) -> Result<(), Trap> {
        for c in f.contracts.iter().filter(|c| !c.ensures) {
            if !self.holds(c, env)? {
                let args = self.arguments(f, env, call)?;
                let message = format!(
                    "`{}` requires `{}`, but it was called with {args} (contract at {}:{})",
                    f.full_name(),
                    c.text,
                    c.pos.line,
                    c.pos.col
                );
                return trap("contract.req.1", call, message);
            }
        }
        Ok(())
    }

    /// contract.ens.1: each `ensures` holds for a success value, with
    /// `result` bound to it. A failure is the function's: it traps at the
    /// `ensures` line and names where the function returned.
    #[inline(never)]
    pub(super) fn check_ensures(
        &mut self,
        f: &'a FnDecl,
        env: &mut Env<'a>,
        result: &Value,
        at: Pos,
    ) -> Result<(), Trap> {
        if matches!(result, Value::Fail(_)) {
            return Ok(());
        }
        let n = env.len();
        if !matches!(result, Value::Unit) {
            env.push(("result", result.clone()));
        }
        for c in f.contracts.iter().filter(|c| c.ensures) {
            if !self.holds(c, env)? {
                env.truncate(n);
                let value = if matches!(result, Value::Unit) {
                    String::new()
                } else {
                    format!(" {}", self.describe(result, c.pos)?)
                };
                let args = self.arguments(f, env, c.pos)?;
                let message = format!(
                    "`{}` ensures `{}`, but it returned{value} at {}:{} with {args}",
                    f.full_name(),
                    c.text,
                    at.line,
                    at.col
                );
                return trap("contract.ens.1", c.pos, message);
            }
        }
        env.truncate(n);
        Ok(())
    }

    /// Evaluate a condition with checks off for the calls inside it (contract.run.2).
    fn holds(&mut self, c: &'a Contract, env: &mut Env<'a>) -> Result<bool, Trap> {
        let outer = CHECKING.with(|k| k.replace(true));
        let v = self.eval(&c.cond, env);
        CHECKING.with(|k| k.set(outer));
        match v {
            Ok(v) => Ok(v.bool()),
            Err(Exit::Trap(t)) => Err(*t),
            Err(_) => trap("contract.decl.2", c.pos, "the condition left the function".into()),
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
