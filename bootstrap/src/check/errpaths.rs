//! Failure paths (err.paths.1, ADR 0285): which error types each function can
//! fail with, and every place it can fail or hand a failure on. `ail api` and
//! `ail check --errors` print what is recorded here. Also the signature fix
//! that `err.*` diagnostics offer (err.fix.1).

use super::*;

/// One place where a function fails, passes a failure on, or handles one.
#[derive(Clone, Debug)]
pub struct FailPath {
    /// The function, by index into `Program::fns`.
    pub func: usize,
    pub pos: Pos,
    /// `propagate` (`?`), `handle` (`else`), `return` (of an error value),
    /// or `absent` (`?` or `else` on an optional, which has no error).
    pub kind: &'static str,
    /// For `handle`: `default` (`else d`), `leave` (`else return`, `else
    /// continue`, ...) or `block`.
    pub how: Option<&'static str>,
    /// The function or method the failure comes from, when it is a call.
    pub call: Option<String>,
    pub errors: Vec<String>,
    /// `?` carried context text (err.ctx.2).
    pub context: bool,
}

/// The name of the function or method an expression calls, if it is a call.
pub fn callee_text(e: &Expr) -> Option<String> {
    match e {
        Expr::Call { callee, .. } => match &**callee {
            Expr::Name(n, _) | Expr::TypeArgs { name: n, .. } => Some(n.clone()),
            _ => None,
        },
        Expr::Method { recv, name, .. } => match &**recv {
            Expr::Name(r, _) => Some(format!("{r}.{name}")),
            _ => Some(format!(".{name}")),
        },
        _ => None,
    }
}

impl<'p> Checker<'p> {
    /// Remember a failure path of the function being checked (final pass only).
    pub(super) fn note_path(
        &mut self,
        pos: Pos,
        kind: &'static str,
        how: Option<&'static str>,
        value: Option<&Expr>,
        errors: &[Ty],
        context: bool,
    ) {
        if !self.track {
            return;
        }
        let errors = self.error_names(errors);
        let call = value.and_then(callee_text);
        self.resolved.fail_paths.push(FailPath { func: self.cur_fn, pos, kind, how, call, errors, context });
    }

    /// Sorted, distinct display names of error types.
    pub(super) fn error_names(&self, errors: &[Ty]) -> Vec<String> {
        let mut names: Vec<String> = errors.iter().map(|t| self.ty_name(t)).collect();
        names.sort();
        names.dedup();
        names
    }

    /// Each fallible function's final error set (err.api.1).
    pub(super) fn record_fn_errors(&mut self) {
        for (i, f) in self.prog.fns.iter().enumerate() {
            if crate::ast::is_prelude(f.pos) {
                continue;
            }
            if let Ty::Res(_, errs) = &self.sigs[i].ret {
                let names = self.error_names(&self.errs_of(errs));
                self.resolved.fn_errors.insert(i, names);
            }
        }
    }

    /// The return type that would let this function fail with `extra` on top of
    /// what it can already fail with, written the way the source writes it:
    /// `T ! A | B`. `None` when the function has an optional return type.
    pub(super) fn failing_ret(&self, extra: &[Ty]) -> Option<String> {
        let mut names = match &self.ret {
            Ty::Res(_, errs) => self.error_names(&self.errs_of(errs)),
            Ty::Opt(_) => return None,
            _ => Vec::new(),
        };
        names.extend(self.error_names(extra));
        names.sort();
        names.dedup();
        let ok = match &self.ret {
            Ty::Res(ok, _) => (**ok).clone(),
            t => t.clone(),
        };
        let ok = if ok == Ty::Unit { String::new() } else { self.ty_name(&ok) };
        Some(format!("{ok} ! {}", names.join(" | ")).trim_start().to_string())
    }

    /// An edit giving the current function the return type `ret` (err.fix.1):
    /// appended to the signature line, so it is offered only when the
    /// function's own return type is absent or a plain value type.
    pub(super) fn signature_fix(&self, extra: &[Ty]) -> Option<(String, Edit)> {
        if matches!(self.ret, Ty::Res(..) | Ty::Opt(_)) {
            return None;
        }
        let names = self.error_names(extra).join(" | ");
        let title = format!("make `{}` fail with {names}", self.fn_name);
        Some((title, Edit::at_line_end(self.fn_head, format!(" ! {names}"))))
    }
}
