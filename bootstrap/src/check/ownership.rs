//! Ownership checks made while typing (ADRs 0063, 0066-0069, 0190-0193):
//! unique types (own.uniq.1-6), `move` parameters and arguments
//! (own.move.1-4), `ref T` (own.ref.1-5) and `defer` (own.defer.1-3).
//!
//! The checker records each name whose use moves its binding's value in
//! `Resolved::moves`; `moves.rs` then follows control flow to report uses
//! after a move.

use super::{Checker, Kind, Ty};
use crate::ast::{Arg, Expr, Stmt, place_path};
use crate::diag::Pos;

impl<'p> Checker<'p> {
    /// A value of `t` can't be copied (own.uniq.1). Optionals and results of
    /// a unique type hold one, so they move too.
    pub(super) fn is_unique(&self, t: &Ty) -> bool {
        match t {
            Ty::Struct(i, _) => self.prog.structs[*i].unique,
            Ty::Enum(i, _) => self.prog.enums[*i].unique,
            Ty::Opt(t) | Ty::Res(t, _) => self.is_unique(t),
            _ => false,
        }
    }

    /// own.uniq.5: a unique value has one owner, so it can't be shared or
    /// held where the bootstrap would copy it.
    pub(super) fn no_unique(&mut self, t: &Ty, pos: Pos, what: &str) {
        if self.is_unique(t) {
            let msg = format!("{} is a unique type, so it can't be {what}", self.show(t));
            self.error_help(
                "own.uniq.5",
                pos,
                msg,
                "hold it in a local, a `move` parameter, or a field of another `unique` type",
            );
        }
    }

    /// own.uniq.5: a field of a unique type makes its owner unique.
    pub(super) fn check_unique_fields(&mut self) {
        let prog = self.prog;
        for (s, decl) in prog.structs.iter().enumerate() {
            if decl.unique {
                continue;
            }
            for (f, field) in decl.fields.iter().enumerate() {
                let t = self.field_types[s][f].clone();
                self.unique_field(&t, field.ty.pos, &decl.name, "struct");
            }
        }
        for (e, decl) in prog.enums.iter().enumerate() {
            if decl.unique {
                continue;
            }
            for (v, variant) in decl.variants.iter().enumerate() {
                for (f, field) in variant.fields.iter().enumerate() {
                    let t = self.variant_types[e][v][f].clone();
                    self.unique_field(&t, field.ty.pos, &decl.name, "enum");
                }
            }
        }
    }

    fn unique_field(&mut self, t: &Ty, pos: Pos, owner: &str, kind: &str) {
        if self.is_unique(t) {
            let msg = format!("{} is a unique type, so a field of it makes `{owner}` unique too", self.show(t));
            self.error_help("own.uniq.5", pos, msg, format!("declare it `unique {kind} {owner}`"));
        }
    }

    /// own.uniq.6: only a unique type has a cleanup, and it is written out.
    pub(super) fn check_drop_impls(&mut self) {
        let Some(drop) = self.std.drop else { return };
        let prog = self.prog;
        for (i, imp) in prog.impls.iter().enumerate() {
            if self.impl_info[i].tr != Some(drop) {
                continue;
            }
            let target = self.impl_info[i].target.clone();
            if target != Ty::Err && !self.is_unique(&target) {
                let name = imp.target.name.clone();
                self.error_help(
                    "own.uniq.6",
                    imp.target.pos,
                    format!("only a unique type has a cleanup, and `{name}` can be copied"),
                    format!("declare it `unique struct {name}` or `unique enum {name}`"),
                );
            } else if imp.methods.is_empty() {
                self.error_help(
                    "own.uniq.6",
                    imp.pos,
                    "a cleanup has no automatic form",
                    "write `fn drop(mut self)` with the cleanup indented below",
                );
            }
        }
    }

    /// own.uniq.6: `x.drop()` would run the cleanup twice.
    pub(super) fn reject_drop_call(&mut self, rt: &Ty, name: &str, pos: Pos) -> bool {
        let reject = name == "drop" && self.is_unique(rt) && self.std.drop.is_some();
        if reject {
            self.error_help(
                "own.uniq.6",
                pos,
                "the cleanup runs by itself when the value's owner goes out of scope",
                "to release it early, pass it to a function that takes it: `fn close(move f File)`",
            );
        }
        reject
    }

    /// `e` gives its value away: bound, assigned, returned, stored in a
    /// field, or left as a block's value. A unique value moves (own.uniq.2).
    pub(super) fn consume(&mut self, e: &'p Expr, t: &Ty) {
        if self.is_unique(t) {
            self.take(e, "own.uniq.2");
        }
    }

    /// Move the value of `e` out of its binding, recording the move for the
    /// flow check and the interpreter.
    fn take(&mut self, e: &'p Expr, rule: &'static str) {
        match e {
            Expr::Name(n, pos) => match self.local(n).map(|l| l.kind) {
                Some(Kind::Binding | Kind::MoveParam) => {
                    self.resolved.moves.insert(*pos, rule);
                }
                Some(kind) => {
                    let (what, help) = match kind {
                        Kind::SelfParam => (
                            "`self` belongs to the caller".to_string(),
                            "a method can't take ownership of its receiver yet".to_string(),
                        ),
                        Kind::LoopVar => (
                            format!("the loop variable `{n}` belongs to the loop"),
                            "move the whole collection instead".to_string(),
                        ),
                        _ => (
                            format!("the parameter `{n}` belongs to the caller"),
                            format!("declare it `move {n} Type` to take ownership"),
                        ),
                    };
                    self.error_help("own.move.3", *pos, format!("{what}, so it can't be moved"), help);
                }
                None => {}
            },
            _ if place_path(e).is_some() => {
                let root = place_path(e).map_or("", |(r, _)| r);
                self.error_help(
                    "own.uniq.1",
                    e.pos(),
                    format!("this value is unique and belongs to `{root}`, so it can't be copied out of it"),
                    format!("move the whole of `{root}`, or use the value where it is"),
                );
            }
            _ if rule == "own.move.2" => self.error_help(
                "own.move.3",
                e.pos(),
                "only a local binding can be moved",
                "remove the `move`; a new value is already the callee's",
            ),
            _ => {}
        }
    }

    /// own.move.2: an argument to a `move` parameter is marked `move`, and
    /// only such an argument is.
    pub(super) fn move_arg(&mut self, a: &'p Arg, param_moves: bool) {
        match (param_moves, a.moved) {
            (true, true) => self.take(&a.value, "own.move.2"),
            (true, false) => self.error_help(
                "own.move.2",
                a.value.pos(),
                "this parameter is `move`, so the argument must be marked `move`",
                "write `move` before the argument; the value can't be used after the call",
            ),
            (false, true) => {
                self.error_help("own.move.2", a.value.pos(), "this parameter isn't `move`", "remove the `move`")
            }
            (false, false) => {}
        }
    }

    /// `ref e` (own.ref.1, ADR 0067).
    pub(super) fn ref_expr(&mut self, value: &'p Expr, pos: Pos, expect: Option<&Ty>) -> Ty {
        let hint = match expect {
            Some(Ty::Ref(t)) => Some((**t).clone()),
            _ => None,
        };
        let t = self.expr(value, hint.as_ref(), true);
        if self.reject_unhandled(&t, value.pos()) {
            return Ty::Err;
        }
        if t == Ty::Unit {
            self.error("own.ref.1", value.pos(), "`ref` needs a value, and this expression produces none");
            return Ty::Err;
        }
        self.no_unique(&t, pos, "the contents of a `ref`");
        Ty::Ref(Box::new(t))
    }

    /// own.ref.4: `e` is reached through a `ref`, so it may be changed
    /// whatever its binding. A `mut self` method may be called on a `ref`
    /// itself (`receiver`).
    pub(super) fn writes_through_ref(&mut self, e: &'p Expr, receiver: bool) -> bool {
        if receiver && matches!(self.quiet_type(e), Ty::Ref(_)) {
            return true;
        }
        match e {
            Expr::Field { recv, .. } | Expr::Index { recv, .. } => {
                matches!(self.quiet_type(recv), Ty::Ref(_)) || self.writes_through_ref(recv, false)
            }
            _ => false,
        }
    }

    /// `defer S` (own.defer.1-3): `S` runs when the block exits, so it can't
    /// leave the block or declare anything.
    pub(super) fn defer_stmt(&mut self, inner: &'p Stmt, pos: Pos) {
        let what = match inner {
            Stmt::Expr(_) | Stmt::Assign { .. } | Stmt::Handle { .. } => None,
            Stmt::Return(..) => Some("`return`"),
            Stmt::Break(_) => Some("`break`"),
            Stmt::Continue(_) => Some("`continue`"),
            Stmt::Let { .. } => Some("a binding"),
            Stmt::Defer(..) => Some("another `defer`"),
            Stmt::While { .. } | Stmt::For { .. } => Some("a loop"),
        };
        if let Some(what) = what {
            self.error_help(
                "own.defer.3",
                pos,
                format!("a deferred statement can't be {what}"),
                "defer a call or an assignment, e.g. `defer f.close()`",
            );
            return;
        }
        let ty = self.stmt(inner, None, false);
        if matches!(inner, Stmt::Expr(_)) {
            self.reject_unhandled(&ty, inner.pos());
        }
    }
}
