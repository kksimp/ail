//! Key functions: `xs.sorted(by: |x| key)`, `sort`, `min` and `max`
//! (expr.key.1-4, ADR 0250). A key function is an argument form, not a value:
//! it can't be stored, returned or passed to a user function.

use super::{Checker, Kind, Local, Ty};
use crate::ast::{Arg, Expr};
use crate::diag::Pos;

const KEYED: &[&str] = &["sorted", "sort", "min", "max"];

impl<'p> Checker<'p> {
    /// A built-in call with a `by:` argument or a `|x|` key function, or
    /// `None` for any other call.
    pub(super) fn keyed_call(&mut self, recv: &'p Expr, rt: &Ty, name: &str, pos: Pos, args: &'p [Arg]) -> Option<Ty> {
        if !args.iter().any(|a| a.param.is_some() || a.label.as_ref().is_some_and(|(l, _)| l == "by")) {
            return None;
        }
        let elem = match rt {
            Ty::List(e) => Some((**e).clone()),
            Ty::Set(e) if name != "sort" => Some((**e).clone()),
            _ => None,
        };
        let shape_ok = KEYED.contains(&name)
            && elem.is_some()
            && args.len() == 1
            && args[0].label.as_ref().is_some_and(|(l, _)| l == "by");
        if !shape_ok {
            for a in args {
                match &a.param {
                    Some((_, ppos)) => self.key_misplaced(*ppos),
                    None => {
                        self.expr(&a.value, None, true);
                    }
                }
            }
            let at = args[0].label.as_ref().map_or(pos, |(_, p)| *p);
            self.error_help(
                "expr.key.1",
                at,
                format!("`{name}` takes no `by:` argument here"),
                "`by: |x| key` goes in `sorted`, `sort`, `min` or `max` of a list (or a set, except `sort`)",
            );
            return Some(Ty::Err);
        }
        let elem = elem.expect("checked above");
        let a = &args[0];
        let Some((var, vpos)) = &a.param else {
            self.expr(&a.value, None, true);
            self.error_help(
                "expr.key.1",
                a.value.pos(),
                "`by:` takes a key function",
                "write `by: |x| key`, e.g. `by: |p| p.age`",
            );
            return Some(Ty::Err);
        };
        // expr.key.2: the parameter is a new immutable name, like a loop variable.
        let mark = self.locals.len();
        if self.declare(var, *vpos) {
            self.locals.push(Local {
                name: var.clone(),
                ty: elem.clone(),
                mutable: false,
                kind: Kind::LoopVar,
                decl: None,
            });
        }
        let kt = self.expr(&a.value, None, true);
        self.end_scope(mark);
        if !matches!(kt, Ty::Err) && !self.implements(&kt, self.std.ord) {
            let msg = format!("a sort key must be ordered (`Ord`), and this is {}", self.show(&kt));
            self.error_help("expr.key.3", a.value.pos(), msg, "use a number, string or `Ord` type as the key");
        }
        if name == "sort" {
            self.require_mutable(recv, Some("trait.method.3"), "the receiver of `sort`");
        }
        Some(match name {
            "sorted" => Ty::List(Box::new(elem)),
            "sort" => Ty::Unit,
            _ => Ty::Opt(Box::new(elem)),
        })
    }

    /// expr.key.1: a `|x|` key function outside `by:` of a built-in.
    pub(super) fn key_misplaced(&mut self, at: Pos) {
        self.error_help(
            "expr.key.1",
            at,
            "a `|x|` key function is only allowed as `by:` of `sorted`, `sort`, `min` or `max`",
            "use a comprehension or a loop, e.g. `[f(x) for x in xs]`",
        );
    }
}
