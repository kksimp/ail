//! Ownership checks made while typing (ADRs 0063, 0066-0069, 0190-0193):
//! unique types (own.uniq.1-6), `move` parameters and arguments
//! (own.move.1-5), `move self` (own.move.6, ADR 0317), unique values in
//! lists and maps (own.uniq.8-10, ADRs 0318-0319), `ref T` (own.ref.1-7)
//! and `defer` (own.defer.1-3).
//!
//! The checker records each name whose use moves its binding's value in
//! `Resolved::moves`; `moves.rs` then follows control flow to report uses
//! after a move.

use super::{Checker, Kind, Ty};
use crate::ast::{Arg, Expr, Stmt, place_path};
use crate::diag::Pos;

impl<'p> Checker<'p> {
    /// A value of `t` can't be copied (own.uniq.1). Optionals and results of
    /// a unique type hold one, so they move too, and so do lists and maps
    /// of one (own.uniq.8).
    pub(super) fn is_unique(&self, t: &Ty) -> bool {
        match t {
            Ty::Struct(i, _) => self.prog.structs[*i].unique,
            Ty::Enum(i, _) => self.prog.enums[*i].unique,
            Ty::Opt(t) | Ty::Res(t, _) | Ty::List(t) | Ty::Map(_, t) => self.is_unique(t),
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
                // own.move.6: a `move self` method owns its receiver.
                Some(Kind::SelfParam) if self.prog.fns[self.cur_fn].receiver.is_some_and(|r| r.moved) => {
                    self.resolved.moves.insert(*pos, rule);
                }
                Some(kind) => {
                    let (what, help) = match kind {
                        Kind::SelfParam => (
                            "`self` belongs to the caller".to_string(),
                            "declare the method `move self` to take ownership of its receiver".to_string(),
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
            _ if place_path(e).is_some() && matches!(rule, "own.move.2" | "own.move.6") => {
                let root = place_path(e).map_or("", |(r, _)| r);
                let help = self.take_out_help(e, root).unwrap_or_else(|| format!("move `{root}` itself, or pass a new value"));
                self.error_help(
                    "own.move.3",
                    e.pos(),
                    format!("only a whole local binding can be moved, and this is part of `{root}`"),
                    help,
                );
            }
            _ if place_path(e).is_some() => {
                let root = place_path(e).map_or("", |(r, _)| r);
                let help = self
                    .take_out_help(e, root)
                    .unwrap_or_else(|| format!("move the whole of `{root}`, or use the value where it is"));
                self.error_help(
                    "own.uniq.1",
                    e.pos(),
                    format!("this value is unique and belongs to `{root}`, so it can't be copied out of it"),
                    help,
                );
            }
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

    /// own.uniq.9: an element is borrowed where it is; `pop`, `remove_at`
    /// and a map's `remove` take one out.
    fn take_out_help(&mut self, e: &'p Expr, root: &str) -> Option<String> {
        let Expr::Index { recv, .. } = e else { return None };
        Some(match self.quiet_type(recv) {
            Ty::Map(..) => format!("take it out with `{root}.remove(k)`, which gives a `T?`, or use it where it is"),
            _ => format!("take it out with `{root}.pop()` or `{root}.remove_at(i)`, or use it where it is"),
        })
    }

    /// own.move.6: calling a `move self` method on `recv` moves it.
    pub(super) fn move_receiver(&mut self, f: usize, recv: &'p Expr) {
        if self.prog.fns[f].receiver.is_some_and(|r| r.moved) {
            self.take(recv, "own.move.6");
        }
    }

    /// The elements of a list literal, or the values of a map literal, of
    /// a unique type move into it (own.uniq.8).
    pub(super) fn consume_elems(&mut self, es: impl IntoIterator<Item = &'p Expr>, elem: &Ty) {
        if self.is_unique(elem) {
            for e in es {
                self.take(e, "own.uniq.8");
            }
        }
    }

    /// own.uniq.9-10: a built-in method of a list or map whose elements are
    /// unique. Which parameters take ownership (their arguments are marked
    /// `move`); a method that would copy an element is an error.
    pub(super) fn unique_builtin(&mut self, rt: &Ty, name: &str, pos: Pos, nparams: usize) -> Vec<bool> {
        let (elem, list) = match rt {
            Ty::List(e) => (&**e, true),
            Ty::Map(_, v) => (&**v, false),
            _ => return Vec::new(),
        };
        if !self.is_unique(elem) {
            return Vec::new();
        }
        let help = match (list, name) {
            (true, "push" | "extend") => return vec![true],
            (true, "insert") => return vec![false, true],
            (true, "pop" | "remove_at" | "len" | "swap" | "clear" | "reverse" | "sort" | "contains" | "index_of") => {
                return vec![false; nparams];
            }
            (false, "remove" | "len" | "contains" | "keys") => return vec![false; nparams],
            (true, "first" | "last") => "borrow it with `xs[0]` or `xs[xs.len() - 1]`, or take it out with `xs.pop()`",
            (true, "min" | "max") => "find its index and borrow it with `xs[i]`",
            (true, _) => "use a comprehension that reads what it needs, e.g. `[c.id for c in xs]`",
            (false, "get") => "borrow it with `m[k]`, or take it out with `m.remove(k)`",
            (false, _) => "iterate the keys and borrow each value: `for k in m.keys()` then `m[k]`",
        };
        let what = if list { "list elements" } else { "map values" };
        let msg = format!("`{name}` would copy {what}, and {} is unique", self.show(elem));
        self.error_help("own.uniq.10", pos, msg, help);
        Vec::new()
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
            self.error("own.ref.5", value.pos(), "`ref` needs a value, and this expression produces none");
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
