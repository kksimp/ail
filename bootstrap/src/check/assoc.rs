//! Associated functions in traits (trait.assoc.1-4, ADR 0349): `fn zero()
//! Self` in a trait, called as `T.zero()` on a bounded type parameter or as
//! `Type.zero()` on an implementing type. The interpreter learns the concrete
//! type from `Resolved::assoc_calls` and `Resolved::type_args`.

use super::{Checker, Con, Def, Sig, Ty, subst};
use crate::ast::Arg;
use crate::diag::Pos;

impl<'p> Checker<'p> {
    /// The trait function `name` without `self` that trait `tr` declares.
    fn assoc_fn(&self, tr: usize, name: &str) -> Option<usize> {
        self.trait_method(tr, name).filter(|&m| self.prog.fns[m].receiver.is_none())
    }

    /// Whether trait `tr`, or one of its supertraits, declares a function
    /// without `self`, so generic code bound by it needs its type at run time.
    fn has_assoc(&self, tr: usize) -> bool {
        let own = |t: usize| self.prog.traits[t].methods.iter().any(|&m| self.prog.fns[m].receiver.is_none());
        own(tr) || self.supers[tr].iter().any(|&s| own(s))
    }

    /// Record the type arguments of a call whose type parameters are bound
    /// by a trait with associated functions (trait.assoc.3): the
    /// interpreter binds them for the callee's `T.name(...)` calls.
    pub(super) fn note_type_args(&mut self, vars: &[usize], binds: &[(usize, Ty)], pos: Pos) {
        let needed: Vec<(usize, Ty)> = binds
            .iter()
            .filter(|(v, _)| vars.contains(v) && self.tparams[*v].bounds.iter().any(|&b| self.has_assoc(b)))
            .cloned()
            .collect();
        if !needed.is_empty() {
            self.resolved.type_args.insert(pos, needed);
        }
        self.last_binds = binds.to_vec();
    }

    /// `T.name(args)` where `T` is a type parameter in scope (trait.assoc.2).
    /// `None` when `tn` isn't a type parameter.
    pub(super) fn param_assoc_call(
        &mut self,
        tn: &str,
        name: &str,
        pos: Pos,
        args: &'p [Arg],
        expect: Option<&Ty>,
    ) -> Option<Ty> {
        let &(_, p) = self.scope.iter().rev().find(|(n, _)| n == tn)?;
        let mut traits: Vec<usize> = Vec::new();
        for &b in &self.tparams[p].bounds {
            traits.push(b);
            traits.extend(self.supers[b].iter().copied());
        }
        let found = traits.iter().find_map(|&tr| self.assoc_fn(tr, name).map(|f| (tr, f)));
        let Some((tr, f)) = found else {
            self.args_only(args);
            let bounds: Vec<String> = self.tparams[p].bounds.iter().map(|&b| format!("`{}`", self.prog.traits[b].name)).collect();
            let offer = (0..self.prog.traits.len()).find(|&t| self.assoc_fn(t, name).is_some());
            let help = if let Some(t) = offer {
                format!("add the bound `{}` to `{tn}`, whose trait declares `fn {name}`", self.prog.traits[t].name)
            } else if bounds.is_empty() {
                format!("add a bound whose trait declares `fn {name}(...)`: `[{tn} Trait]`")
            } else {
                format!("`{tn}` is bound by {}; add a bound whose trait declares `fn {name}(...)` without `self`", bounds.join(", "))
            };
            self.error_help("trait.assoc.2", pos, format!("`{tn}` has no associated function `{name}`"), help);
            // Type the call as the trait that declares it would, so its
            // `else e` binds and reports nothing more.
            let Some(t) = offer else { return Some(Ty::Err) };
            let f = self.assoc_fn(t, name).expect("found above");
            let s = self.sigs[f].clone();
            let ret = subst(&s.ret, &[(self.trait_self[t], Ty::Err)]);
            return Some(ret);
        };
        self.resolved.assoc_calls.insert(pos, (tr, Ty::Param(p)));
        self.resolved.symbols.insert(pos, format!("method:{}.{name}", self.prog.traits[tr].name));
        let sig = self.sigs[f].clone();
        let binds = vec![(self.trait_self[tr], Ty::Param(p))];
        let what = format!("{tn}.{name}");
        Some(self.check_call(&what, &sig, binds, args, None, pos, expect))
    }

    /// `Type.name(args)` naming an associated function of a trait that
    /// `Type` implements (trait.assoc.4). `None` when there is none.
    pub(super) fn impl_assoc_call(
        &mut self,
        def: Def,
        explicit: Option<Vec<(usize, Ty)>>,
        name: &str,
        pos: Pos,
        args: &'p [Arg],
        expect: Option<&Ty>,
    ) -> Option<Ty> {
        let con = match def {
            Def::Struct(s) => Con::Struct(s),
            Def::Enum(e) => Con::Enum(e),
        };
        let prog = self.prog;
        let (tr, f, i) = (0..prog.traits.len()).find_map(|tr| {
            let decl = self.assoc_fn(tr, name)?;
            let &i = self.impl_of.get(&(tr, con))?;
            let own = prog.impls[i].methods.iter().copied().find(|&m| prog.fns[m].name == name);
            Some((tr, own.unwrap_or(decl), i))
        })?;
        let target = self.impl_info[i].target.clone();
        let tn = self.def_name(def);
        let sig: Sig = if prog.fns[f].within == crate::ast::Within::Trait(tr) {
            // The trait's default body, with `Self` the implementing type.
            let s = self.sigs[f].clone();
            let b = vec![(self.trait_self[tr], target.clone())];
            Sig {
                params: s.params.iter().map(|(m, t)| (*m, subst(t, &b))).collect(),
                ret: subst(&s.ret, &b),
                generics: self.impl_info[i].params.clone(),
                owner: self.impl_info[i].params.len(),
                ..s
            }
        } else {
            self.sigs[f].clone()
        };
        // Explicit `Type[int].name()` arguments name the type's parameters,
        // which the `impl` block's parameters stand for in order.
        let binds = match explicit {
            Some(b) => self.impl_info[i].params.iter().copied().zip(b.into_iter().map(|(_, t)| t)).collect(),
            None => Vec::new(),
        };
        self.resolved.targets.insert(pos, f);
        self.resolved.symbols.insert(pos, format!("method:{tn}.{name}"));
        let t = self.check_call(&format!("{tn}.{name}"), &sig, binds, args, None, pos, expect);
        let concrete = subst(&target, &self.last_binds);
        self.resolved.assoc_calls.insert(pos, (tr, concrete));
        Some(t)
    }

    /// `List[T].name(args)` on a builtin type that implements a trait with an
    /// associated function (trait.assoc.4, ADR 0748): `List[U].from_json(j)`.
    /// `None` when no trait offers `name` for `ty`.
    pub(super) fn builtin_assoc_call(
        &mut self,
        ty: Ty,
        name: &str,
        pos: Pos,
        args: &'p [Arg],
        expect: Option<&Ty>,
    ) -> Option<Ty> {
        let con = super::con_of(&ty)?;
        let prog = self.prog;
        let (tr, f, i) = (0..prog.traits.len()).find_map(|tr| {
            let decl = self.assoc_fn(tr, name)?;
            let &i = self.impl_of.get(&(tr, con))?;
            let own = prog.impls[i].methods.iter().copied().find(|&m| prog.fns[m].name == name);
            Some((tr, own.unwrap_or(decl), i))
        })?;
        let target = self.impl_info[i].target.clone();
        let params = self.impl_info[i].params.clone();
        let mut binds = Vec::new();
        super::bind_types(&target, &ty, &params, &mut binds);
        let sig: Sig = if prog.fns[f].within == crate::ast::Within::Trait(tr) {
            let s = self.sigs[f].clone();
            let b = vec![(self.trait_self[tr], target.clone())];
            Sig {
                params: s.params.iter().map(|(m, t)| (*m, subst(t, &b))).collect(),
                ret: subst(&s.ret, &b),
                generics: params,
                owner: self.impl_info[i].params.len(),
                ..s
            }
        } else {
            self.sigs[f].clone()
        };
        self.resolved.targets.insert(pos, f);
        self.resolved.symbols.insert(pos, format!("method:{}.{name}", self.show(&ty)));
        let t = self.check_call(&format!("{}.{name}", self.show(&ty)), &sig, binds, args, None, pos, expect);
        self.resolved.assoc_calls.insert(pos, (tr, ty));
        Some(t)
    }
}
