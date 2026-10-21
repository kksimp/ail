//! Named arguments (expr.call.3, ADR 0454): after checking, put the
//! arguments of every call that names a parameter into parameter order, so
//! the interpreter and the bytecode tier see only positional calls.

use crate::ast::*;
use crate::check::Resolved;

/// Reorder the arguments of each call the checker recorded in
/// `resolved.arg_order`, and drop their labels.
pub fn reorder(prog: &mut Program, resolved: &Resolved) {
    if resolved.arg_order.is_empty() {
        return;
    }
    let mut cx = Cx { order: resolved };
    for f in prog.fns.iter_mut() {
        if let Some(b) = &mut f.body {
            cx.block(b);
        }
    }
    for c in prog.consts.iter_mut() {
        cx.expr(&mut c.value);
    }
}

struct Cx<'a> {
    order: &'a Resolved,
}

impl Cx<'_> {
    fn block(&mut self, b: &mut Block) {
        b.iter_mut().for_each(|s| self.stmt(s));
    }

    fn stmt(&mut self, s: &mut Stmt) {
        match s {
            Stmt::Let { value, otherwise, .. } => {
                self.expr(value);
                if let Some(o) = otherwise {
                    self.otherwise(o);
                }
            }
            Stmt::Assign { target, value, .. } => {
                self.expr(target);
                self.expr(value);
            }
            Stmt::While { cond, body, .. } => {
                self.expr(cond);
                self.block(body);
            }
            Stmt::For { iter, body, .. } | Stmt::With { mutex: iter, body, .. } => {
                self.expr(iter);
                self.block(body);
            }
            Stmt::Unsafe { body, .. } => self.block(body),
            Stmt::Return(Some(e), _) | Stmt::Expr(e) | Stmt::LetTuple { value: e, .. } => self.expr(e),
            Stmt::Handle { value, otherwise } => {
                self.expr(value);
                self.otherwise(otherwise);
            }
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return(None, _) => {}
            Stmt::Defer(s, _) => self.stmt(s),
        }
    }

    fn otherwise(&mut self, o: &mut Else) {
        match o {
            Else::Value(e) => self.expr(e),
            Else::Leave(s) => self.stmt(s),
            Else::Block { body, .. } => self.block(body),
        }
    }

    /// Put `args` in the order recorded for the call whose first label is at
    /// `args`' first labelled argument, if there is one.
    fn args(&mut self, args: &mut Vec<Arg>) {
        let first = args.iter().find_map(|a| a.label.as_ref().map(|(_, p)| *p));
        if let Some(order) = first.and_then(|p| self.order.arg_order.get(&p)) {
            let mut slots: Vec<Option<Arg>> = std::mem::take(args).into_iter().map(Some).collect();
            for &k in order {
                let mut a = slots[k].take().expect("checked: each argument once");
                a.label = None;
                args.push(a);
            }
        }
        for a in args.iter_mut() {
            self.expr(&mut a.value);
        }
    }

    fn expr(&mut self, e: &mut Expr) {
        match e {
            Expr::Call { callee, args, .. } => {
                self.expr(callee);
                self.args(args);
            }
            Expr::Method { recv, args, .. } => {
                self.expr(recv);
                self.args(args);
            }
            Expr::Interp(parts, _) => {
                for p in parts {
                    if let InterpPart::Expr(x) | InterpPart::Fmt(x, _) = p {
                        self.expr(x);
                    }
                }
            }
            Expr::List(items, _) | Expr::Set(items, _) | Expr::Tuple(items, _) => items.iter_mut().for_each(|x| self.expr(x)),
            Expr::Comp { elem, value, clauses, .. } => {
                for c in clauses {
                    self.expr(&mut c.iter);
                    if let Some(cond) = &mut c.cond {
                        self.expr(cond);
                    }
                }
                self.expr(elem);
                if let Some(v) = value {
                    self.expr(v);
                }
            }
            Expr::Map(pairs, _) => pairs.iter_mut().for_each(|(k, v)| {
                self.expr(k);
                self.expr(v);
            }),
            Expr::Field { recv, .. } => self.expr(recv),
            Expr::Index { recv, index, .. } => {
                self.expr(recv);
                self.expr(index);
            }
            Expr::Unary { operand, .. } | Expr::Ref { value: operand, .. } => self.expr(operand),
            Expr::Binary { lhs, rhs, .. } => {
                self.expr(lhs);
                self.expr(rhs);
            }
            Expr::Range { lo, hi, .. } => {
                if let Some(x) = lo {
                    self.expr(x);
                }
                if let Some(x) = hi {
                    self.expr(x);
                }
            }
            Expr::If { branches, els, .. } => {
                for (cond, body) in branches.iter_mut() {
                    match cond {
                        Cond::Expr(c) => self.expr(c),
                        Cond::Bind { value, .. } => self.expr(value),
                    }
                    self.block(body);
                }
                if let Some(b) = els {
                    self.block(b);
                }
            }
            Expr::Match { scrutinee, arms, .. } => {
                self.expr(scrutinee);
                for arm in arms.iter_mut() {
                    if let Some(g) = &mut arm.guard {
                        self.expr(g);
                    }
                    self.block(&mut arm.body);
                }
            }
            Expr::Else { value, otherwise, .. } => {
                self.expr(value);
                self.otherwise(otherwise);
            }
            Expr::Try { expr, context, .. } => {
                self.expr(expr);
                if let Some(c) = context {
                    self.expr(c);
                }
            }
            Expr::Str(..)
            | Expr::Bytes(..)
            | Expr::Int(..)
            | Expr::Float(..)
            | Expr::Char(..)
            | Expr::Bool(..)
            | Expr::None(_)
            | Expr::Hole(_)
            | Expr::Name(..)
            | Expr::TypeArgs { .. } => {}
        }
    }
}
