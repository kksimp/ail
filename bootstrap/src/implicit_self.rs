//! Implicit `self` (trait.method.6-7, ADR 0141). Inside a method, the
//! receiver's fields and methods are in scope by bare name. Before checking,
//! this pass rewrites `x` to `self.x` and `f(...)` to `self.f(...)`, and
//! reports locals and parameters that would hide a member.

use std::collections::HashSet;

use crate::ast::{Block, Cond, Else, Expr, InterpPart, Pattern, Program, Stmt, Within};
use crate::diag::{Diagnostic, Pos};

/// The prelude's functions, which a bare call could also mean.
const PRELUDE_FNS: &[&str] = &["print", "unreachable", "assert", "assert_eq"];

pub fn rewrite(prog: &mut Program, diags: &mut Vec<Diagnostic>) {
    let free: HashSet<String> = prog
        .fns
        .iter()
        .filter(|f| f.owner.is_none() && f.within == Within::TopLevel)
        .map(|f| f.name.clone())
        .chain(PRELUDE_FNS.iter().map(|n| n.to_string()))
        .collect();
    for i in 0..prog.fns.len() {
        if prog.fns[i].receiver.is_none() || prog.fns[i].body.is_none() {
            continue;
        }
        let (owner, fields, methods) = members(prog, i);
        if fields.is_empty() && methods.is_empty() {
            continue;
        }
        let mut cx = Cx { owner, fields, methods, free: &free, diags: &mut *diags };
        for p in &prog.fns[i].params {
            cx.declare(&p.name, p.pos);
        }
        let mut body = prog.fns[i].body.take().expect("a body");
        cx.block(&mut body);
        prog.fns[i].body = Some(body);
    }
}

/// The type (or trait) whose members method `i` sees, its field names, and
/// its method names.
fn members(prog: &Program, i: usize) -> (String, HashSet<String>, HashSet<String>) {
    let f = &prog.fns[i];
    let method_names = |fs: &[usize]| -> Vec<String> {
        fs.iter().filter(|&&m| prog.fns[m].receiver.is_some()).map(|&m| prog.fns[m].name.clone()).collect()
    };
    let ty = match f.within {
        Within::Trait(t) => {
            let methods = method_names(&prog.traits[t].methods).into_iter().collect();
            return (prog.traits[t].name.clone(), HashSet::new(), methods);
        }
        Within::Impl(k) => prog.impls[k].target.name.clone(),
        Within::TopLevel => match &f.owner {
            Some(o) => o.name.clone(),
            None => return (String::new(), HashSet::new(), HashSet::new()),
        },
    };
    let fields: HashSet<String> =
        prog.structs.iter().filter(|s| s.name == ty).flat_map(|s| s.fields.iter().map(|fd| fd.name.clone())).collect();
    let mut methods: HashSet<String> = prog
        .fns
        .iter()
        .filter(|g| g.receiver.is_some())
        .filter(|g| match g.within {
            Within::TopLevel => g.owner.as_ref().is_some_and(|o| o.name == ty),
            Within::Impl(k) => prog.impls[k].target.name == ty,
            Within::Trait(_) => false,
        })
        .map(|g| g.name.clone())
        .collect();
    for imp in prog.impls.iter().filter(|imp| imp.target.name == ty) {
        if let Some(t) = prog.traits.iter().find(|t| t.name == imp.trait_name) {
            methods.extend(method_names(&t.methods));
        }
    }
    (ty, fields, methods)
}

struct Cx<'a> {
    owner: String,
    fields: HashSet<String>,
    methods: HashSet<String>,
    free: &'a HashSet<String>,
    diags: &'a mut Vec<Diagnostic>,
}

impl Cx<'_> {
    /// trait.method.7: a local or parameter can't take a field's name. (A
    /// method's name is fine: a method is only reached by a call.)
    fn declare(&mut self, name: &str, pos: Pos) {
        if !self.fields.contains(name) {
            return;
        }
        self.diags.push(
            Diagnostic::new(
                "trait.method.7",
                pos,
                format!("`{name}` is a field of `{}`, so a method's local can't have that name", self.owner),
            )
            .help(format!("rename the local; inside the method, `{name}` already means `self.{name}`")),
        );
    }

    fn block(&mut self, b: &mut Block) {
        for s in b {
            self.stmt(s);
        }
    }

    fn stmt(&mut self, s: &mut Stmt) {
        match s {
            Stmt::Let { name, pos, value, otherwise, .. } => {
                self.expr(value);
                if let Some(o) = otherwise {
                    self.otherwise(o);
                }
                let (name, pos) = (name.clone(), *pos);
                self.declare(&name, pos);
            }
            Stmt::Assign { target, value, .. } => {
                self.expr(target);
                self.expr(value);
            }
            Stmt::While { cond, body, .. } => {
                self.expr(cond);
                self.block(body);
            }
            Stmt::For { var, var_pos, iter, body, .. } => {
                self.expr(iter);
                let (var, pos) = (var.clone(), *var_pos);
                self.declare(&var, pos);
                self.block(body);
            }
            Stmt::Return(Some(e), _) | Stmt::Expr(e) => self.expr(e),
            Stmt::Handle { value, otherwise } => {
                self.expr(value);
                self.otherwise(otherwise);
            }
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return(None, _) => {}
        }
    }

    fn otherwise(&mut self, o: &mut Else) {
        match o {
            Else::Value(e) => self.expr(e),
            Else::Leave(s) => self.stmt(s),
            Else::Block { err, body, .. } => {
                if let Some((name, pos)) = err.clone() {
                    self.declare(&name, pos);
                }
                self.block(body);
            }
        }
    }

    fn pattern(&mut self, p: &Pattern) {
        match p {
            Pattern::Variant { bindings, .. } => {
                for (name, pos) in bindings.iter().flatten() {
                    self.declare(name, *pos);
                }
            }
            Pattern::Or(alts, _) => alts.iter().for_each(|a| self.pattern(a)),
            Pattern::Wildcard(_) | Pattern::Literal(..) => {}
        }
    }

    fn expr(&mut self, e: &mut Expr) {
        match e {
            Expr::Name(n, pos) if self.fields.contains(n.as_str()) => {
                let (name, pos) = (std::mem::take(n), *pos);
                *e = Expr::Field { recv: Box::new(Expr::Name("self".into(), pos)), name, name_pos: pos };
            }
            Expr::Call { callee, args, pos } => {
                for a in args.iter_mut() {
                    self.expr(&mut a.value);
                }
                let Expr::Name(n, name_pos) = &**callee else {
                    self.expr(callee);
                    return;
                };
                if !self.methods.contains(n.as_str()) {
                    return;
                }
                if self.free.contains(n.as_str()) {
                    let msg = format!("`{n}` is both a method of `{}` and a function", self.owner);
                    let help = format!("write `self.{n}(...)` for the method, or rename one of them");
                    self.diags.push(Diagnostic::new("trait.method.7", *pos, msg).help(help));
                    return;
                }
                let (name, name_pos) = (n.clone(), *name_pos);
                let args = std::mem::take(args);
                *e = Expr::Method { recv: Box::new(Expr::Name("self".into(), name_pos)), name, name_pos, args };
            }
            Expr::Interp(parts, _) => {
                for p in parts {
                    if let InterpPart::Expr(x) | InterpPart::Fmt(x, _) = p {
                        self.expr(x);
                    }
                }
            }
            Expr::List(items, _) | Expr::Set(items, _) => items.iter_mut().for_each(|x| self.expr(x)),
            Expr::Map(pairs, _) => pairs.iter_mut().for_each(|(k, v)| {
                self.expr(k);
                self.expr(v);
            }),
            Expr::Method { recv, args, .. } => {
                self.expr(recv);
                args.iter_mut().for_each(|a| self.expr(&mut a.value));
            }
            Expr::Field { recv, .. } => self.expr(recv),
            Expr::Index { recv, index, .. } => {
                self.expr(recv);
                self.expr(index);
            }
            Expr::Unary { operand, .. } => self.expr(operand),
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
            Expr::If { branches, els, else_err, .. } => {
                for (cond, body) in branches.iter_mut() {
                    match cond {
                        Cond::Expr(c) => self.expr(c),
                        Cond::Bind { name, pos, value } => {
                            self.expr(value);
                            let (name, pos) = (name.clone(), *pos);
                            self.declare(&name, pos);
                        }
                    }
                    self.block(body);
                }
                if let Some((name, pos)) = else_err.clone() {
                    self.declare(&name, pos);
                }
                if let Some(b) = els {
                    self.block(b);
                }
            }
            Expr::Match { scrutinee, arms, .. } => {
                self.expr(scrutinee);
                for arm in arms.iter_mut() {
                    self.pattern(&arm.pat);
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
