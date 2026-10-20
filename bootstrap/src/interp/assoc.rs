//! A trait's associated functions at run time (trait.assoc.2-4, ADR 0349):
//! `T.zero()` runs the implementation for the type `T` stands for. Calls
//! whose type parameters need it bind them (`Resolved::type_args`); the
//! bindings live on a stack while the callee runs.

use std::io::Write;

use super::{Env, Flow, Interp, Value};
use crate::ast::Arg;
use crate::check::{Con, Ty};
use crate::diag::Pos;

/// The type parameters `t` mentions.
fn params_of(t: &Ty, out: &mut Vec<usize>) {
    match t {
        Ty::Param(p) => out.push(*p),
        Ty::Struct(_, a) | Ty::Enum(_, a) => a.iter().for_each(|x| params_of(x, out)),
        Ty::List(x) | Ty::Set(x) | Ty::Opt(x) => params_of(x, out),
        Ty::Map(k, v) => {
            params_of(k, out);
            params_of(v, out);
        }
        _ => {}
    }
}

impl<'a, W: Write> Interp<'a, W> {
    /// `t` with the type parameters bound by the running calls replaced.
    fn concrete(&self, t: &Ty) -> Ty {
        let mut ps = Vec::new();
        params_of(t, &mut ps);
        if ps.is_empty() {
            return t.clone();
        }
        let binds: Vec<(usize, Ty)> =
            ps.iter().filter_map(|p| self.tyenv.iter().rev().find(|(q, _)| q == p).cloned()).collect();
        crate::check::subst(t, &binds)
    }

    /// The concrete type arguments a call at `pos` binds (trait.assoc.3).
    pub(super) fn call_type_args(&self, pos: Pos) -> Option<Vec<(usize, Ty)>> {
        let b = self.resolved.type_args.get(&pos)?;
        Some(b.iter().map(|(p, t)| (*p, self.concrete(t))).collect())
    }

    /// `T.name(args)` or `Type.name(args)` of trait `tr` (trait.assoc.2,
    /// trait.assoc.4): the function of the concrete type's `impl`, or the
    /// trait's default, with the `impl` block's parameters bound.
    pub(super) fn assoc_call(&mut self, name: &str, args: &'a [Arg], pos: Pos, env: &mut Env<'a>) -> Flow<Value> {
        let prog = self.prog;
        let (tr, t) = &self.resolved.assoc_calls[&pos];
        let ty = self.concrete(t);
        let con = crate::check::con_of(&ty).unwrap_or(Con::Int);
        let imp = self.resolved.impls.get(&(*tr, con)).copied();
        let own = imp.and_then(|i| prog.impls[i].methods.iter().copied().find(|&m| prog.fns[m].name == name));
        let f = match own {
            Some(m) => m,
            None => prog.traits[*tr].methods.iter().copied().find(|&m| prog.fns[m].name == name).expect("checked"),
        };
        // A default body's `Self.name()` runs for the same type (trait.self.1).
        let mut binds = vec![(self.resolved.trait_self[*tr], ty.clone())];
        if let Some(i) = imp
            && let Some(target) = self.resolved.impl_types.get(&i)
        {
            let mut vars = Vec::new();
            params_of(target, &mut vars);
            crate::check::bind_types(target, &ty, &vars, &mut binds);
        }
        let mark = self.tyenv.len();
        self.tyenv.extend(binds);
        let result = self.invoke(&prog.fns[f], None, args, pos, env);
        self.tyenv.truncate(mark);
        result
    }
}
