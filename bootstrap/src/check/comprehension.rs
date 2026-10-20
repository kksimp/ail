//! Comprehensions: `[e for x in xs if c]`, `{e for ...}` and
//! `{k: v for ...}` (expr.comp.1-4, ADR 0182).

use super::{Checker, Kind, Local, Ty, has_err};
use crate::ast::{CompFor, CompKind, Expr};

impl<'p> Checker<'p> {
    /// The type of a comprehension. Each clause's variable is a new immutable
    /// name, visible to later clauses and to the element (expr.comp.2); the
    /// element and its value are checked once, against the expected element
    /// type when the context gives one (expr.comp.3).
    pub(super) fn comprehension(
        &mut self,
        kind: CompKind,
        elem: &'p Expr,
        value: Option<&'p Expr>,
        clauses: &'p [CompFor],
        expect: Option<&Ty>,
    ) -> Ty {
        let expect = match expect {
            Some(Ty::Opt(t)) => Some(&**t),
            e => e,
        };
        let (hint, vhint) = match (kind, expect) {
            (CompKind::List, Some(Ty::List(t))) | (CompKind::Set, Some(Ty::Set(t))) => (Some((**t).clone()), None),
            (CompKind::Map, Some(Ty::Map(k, v))) => (Some((**k).clone()), Some((**v).clone())),
            _ => (None, None),
        };
        let mark = self.locals.len();
        // expr.comp.4: `continue` skips an element and `break` ends the comprehension.
        self.loops += 1;
        for c in clauses {
            let t = self.iterable(&c.iter);
            if self.declare(&c.var, c.var_pos) {
                self.locals.push(Local { name: c.var.clone(), ty: t, mutable: false, kind: Kind::LoopVar, decl: None });
            }
            if let Some(cond) = &c.cond {
                self.condition(cond);
            }
        }
        let t = self.comp_part(elem, hint);
        let v = value.map(|v| self.comp_part(v, vhint));
        self.loops -= 1;
        self.end_scope(mark);
        if !matches!(kind, CompKind::List) && !has_err(&t) && t != Ty::Never {
            self.require_key(&t, elem.pos());
        }
        match kind {
            CompKind::List => Ty::List(Box::new(t)),
            CompKind::Set => Ty::Set(Box::new(t)),
            CompKind::Map => Ty::Map(Box::new(t), Box::new(v.unwrap_or(Ty::Err))),
        }
    }

    /// The element (or key, or value) of a comprehension: a value that fits
    /// `hint` when there is one (expr.comp.3).
    fn comp_part(&mut self, e: &'p Expr, hint: Option<Ty>) -> Ty {
        let hint = hint.filter(|h| !has_err(h));
        let t = self.expr(e, hint.as_ref(), true);
        if self.reject_unhandled(&t, e.pos()) {
            return Ty::Err;
        }
        if t == Ty::Unit {
            self.error("expr.comp.3", e.pos(), "a comprehension's element must be a value");
            return Ty::Err;
        }
        match hint {
            Some(h) => {
                if !self.accepts(&h, &t) {
                    let msg = format!("the elements here are {}, but this is {}", self.show(&h), self.show(&t));
                    self.error("expr.comp.3", e.pos(), msg);
                }
                h
            }
            None => t,
        }
    }
}
