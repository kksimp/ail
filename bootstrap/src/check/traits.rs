//! Traits, `impl` blocks and generics: which types implement which traits
//! (trait.impl.1-6, trait.auto.1-3, trait.std.2-4, trait.any.5), what a
//! method call can mean (trait.res.1), and type-parameter substitution and
//! inference (gen.infer.1).

use super::{Checker, Con, Def, Errs, ImplInfo, Sig, Ty, list_names, same};
use crate::ast::is_prelude;
use crate::diag::Diagnostic;

/// What `x.name(...)` can refer to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Cand {
    /// A method declared `fn Type.name(...)`, by function index.
    Inherent(usize),
    /// A method of a built-in type (`check/builtins.rs`).
    Builtin,
    /// The method of a trait the receiver's type implements.
    Trait(usize),
}

/// Replace the type parameters bound in `binds`.
pub(crate) fn subst(t: &Ty, binds: &[(usize, Ty)]) -> Ty {
    map_params(t, &|p| binds.iter().find(|(q, _)| *q == p).map(|(_, t)| t.clone()))
}

/// Like `subst`, but the parameters among `vars` that aren't bound yet
/// become unknown (`Err`).
pub(super) fn subst_unknown(t: &Ty, binds: &[(usize, Ty)], vars: &[usize]) -> Ty {
    map_params(t, &|p| match binds.iter().find(|(q, _)| *q == p) {
        Some((_, t)) => Some(t.clone()),
        None if vars.contains(&p) => Some(Ty::Err),
        None => None,
    })
}

fn map_params(t: &Ty, f: &dyn Fn(usize) -> Option<Ty>) -> Ty {
    let all = |ts: &[Ty]| ts.iter().map(|t| map_params(t, f)).collect();
    match t {
        Ty::Param(p) => f(*p).unwrap_or(Ty::Param(*p)),
        Ty::Struct(s, args) => Ty::Struct(*s, all(args)),
        Ty::Enum(e, args) => Ty::Enum(*e, all(args)),
        Ty::List(x) => Ty::List(Box::new(map_params(x, f))),
        Ty::Set(x) => Ty::Set(Box::new(map_params(x, f))),
        Ty::Map(k, v) => Ty::Map(Box::new(map_params(k, f)), Box::new(map_params(v, f))),
        Ty::Opt(x) => Ty::Opt(Box::new(map_params(x, f))),
        Ty::Conc(k, x) => Ty::Conc(*k, Box::new(map_params(x, f))),
        Ty::Res(x, Errs::Set(set)) => Ty::Res(Box::new(map_params(x, f)), Errs::Set(all(set))),
        Ty::Res(x, errs) => Ty::Res(Box::new(map_params(x, f)), errs.clone()),
        Ty::Union(m) => Ty::Union(all(m)),
        Ty::Fn(ps, r) => Ty::Fn(all(ps), Box::new(map_params(r, f))),
        t => t.clone(),
    }
}

/// Infer the parameters among `vars` that `pattern` mentions from the type
/// `actual` of a value given for it (gen.infer.1). The first binding wins;
/// later mismatches are reported by the caller.
pub(crate) fn bind(pattern: &Ty, actual: &Ty, vars: &[usize], binds: &mut Vec<(usize, Ty)>) {
    match (pattern, actual) {
        (_, Ty::Err | Ty::Never | Ty::Unit) => {}
        (Ty::Param(p), a) if vars.contains(p) => {
            if !binds.iter().any(|(q, _)| q == p) && !super::has_err(a) {
                binds.push((*p, a.clone()));
            }
        }
        (Ty::List(x), Ty::List(y)) | (Ty::Opt(x), Ty::Opt(y)) | (Ty::Set(x), Ty::Set(y)) => bind(x, y, vars, binds),
        (Ty::Conc(a, x), Ty::Conc(b, y)) if a == b => bind(x, y, vars, binds),
        (Ty::Map(k, v), Ty::Map(l, w)) => {
            bind(k, l, vars, binds);
            bind(v, w, vars, binds);
        }
        // Optional wrapping (type.opt.5).
        (Ty::Opt(x), y) => bind(x, y, vars, binds),
        (Ty::Struct(s, xs), Ty::Struct(t, ys)) | (Ty::Enum(s, xs), Ty::Enum(t, ys)) if s == t => {
            for (x, y) in xs.iter().zip(ys) {
                bind(x, y, vars, binds);
            }
        }
        (Ty::Fn(xs, r), Ty::Fn(ys, s)) if xs.len() == ys.len() => {
            for (x, y) in xs.iter().zip(ys) {
                bind(x, y, vars, binds);
            }
            bind(r, s, vars, binds);
        }
        (Ty::Res(x, _), Ty::Res(y, _)) => bind(x, y, vars, binds),
        (Ty::Res(x, _), y) => bind(x, y, vars, binds),
        _ => {}
    }
}

/// The type bound to `p`, or unknown.
pub(super) fn bound(binds: &[(usize, Ty)], p: usize) -> Ty {
    binds.iter().find(|(q, _)| *q == p).map_or(Ty::Err, |(_, t)| t.clone())
}

/// Whether `t` mentions the type parameter `p`.
fn mentions(t: &Ty, p: usize) -> bool {
    match t {
        Ty::Param(q) => *q == p,
        Ty::Struct(_, xs) | Ty::Enum(_, xs) | Ty::Union(xs) => xs.iter().any(|x| mentions(x, p)),
        Ty::List(x) | Ty::Opt(x) | Ty::Set(x) | Ty::Conc(_, x) => mentions(x, p),
        Ty::Map(k, v) => mentions(k, p) || mentions(v, p),
        Ty::Res(x, Errs::Set(set)) => mentions(x, p) || set.iter().any(|e| mentions(e, p)),
        Ty::Res(x, _) => mentions(x, p),
        Ty::Fn(ps, r) => ps.iter().any(|x| mentions(x, p)) || mentions(r, p),
        _ => false,
    }
}

/// The type constructor of a type that can have an `impl` (trait.impl.6).
pub(crate) fn con_of(t: &Ty) -> Option<Con> {
    Some(match t {
        Ty::Int => Con::Int,
        Ty::Float => Con::Float,
        Ty::Str => Con::Str,
        Ty::Char => Con::Char,
        Ty::Bool => Con::Bool,
        Ty::List(_) => Con::List,
        Ty::Struct(s, _) => Con::Struct(*s),
        Ty::Enum(e, _) => Con::Enum(*e),
        _ => return None,
    })
}

impl<'p> Checker<'p> {
    pub(super) fn collect_traits(&mut self) {
        let prog = self.prog;
        for (i, t) in prog.traits.iter().enumerate() {
            let name = t.name.as_str();
            match self.traits.get(name) {
                Some(&j) if is_prelude(prog.traits[j].pos) => {
                    self.error("mod.unit.1", t.pos, format!("`{name}` is already a built-in trait"));
                }
                Some(_) => {
                    self.error("mod.unit.1", t.pos, format!("`{name}` is declared more than once in this module"))
                }
                None => {
                    self.traits.insert(name, i);
                }
            }
            let self_param = self.new_param("Self", vec![i]);
            self.trait_self.push(self_param);
            self.resolved.trait_self.push(self_param);
            if is_prelude(t.pos) {
                match name {
                    "Eq" => self.std.eq = i,
                    "Ord" => self.std.ord = i,
                    "ToStr" => self.std.to_str = i,
                    "Hash" => self.std.hash = i,
                    "Drop" => self.std.drop = Some(i),
                    _ => {}
                }
            }
            for (k, &m) in t.methods.iter().enumerate() {
                let f = &prog.fns[m];
                if t.methods[..k].iter().any(|&n| prog.fns[n].name == f.name) {
                    self.error("mod.unit.1", f.pos, format!("`{name}.{}` is declared more than once", f.name));
                }
            }
        }
        self.collect_supers();
    }

    /// Resolve `trait Ord Eq` (trait.super.1): each trait's supertraits,
    /// transitively. A trait that requires itself is an error.
    fn collect_supers(&mut self) {
        let prog = self.prog;
        let direct: Vec<Vec<usize>> = prog
            .traits
            .iter()
            .map(|t| t.supers.iter().filter_map(|(n, p)| self.trait_named(n, *p, "trait.super.1")).collect())
            .collect();
        for (i, t) in prog.traits.iter().enumerate() {
            let mut all: Vec<usize> = Vec::new();
            let mut todo = direct[i].clone();
            while let Some(s) = todo.pop() {
                if !all.contains(&s) {
                    all.push(s);
                    todo.extend(direct[s].iter().copied());
                }
            }
            if all.contains(&i) {
                self.error_help(
                    "trait.super.1",
                    t.pos,
                    format!("`{}` requires itself through its supertraits", t.name),
                    "remove one of the supertraits so they don't form a cycle",
                );
                all.retain(|&s| s != i);
            }
            self.supers.push(all);
        }
    }

    /// Whether a bound `b` provides trait `tr`: it is `tr` or requires it
    /// (gen.bound.2, trait.super.3).
    pub(super) fn provides(&self, b: usize, tr: usize) -> bool {
        b == tr || self.supers.get(b).is_some_and(|s| s.contains(&tr))
    }

    /// Resolve each `impl` block's trait and type, and register it
    /// (trait.impl.1, trait.impl.4, trait.impl.6, gen.method.2).
    pub(super) fn collect_impls(&mut self) {
        let prog = self.prog;
        for imp in &prog.impls {
            let tr = self.trait_named(&imp.trait_name, imp.trait_pos, "trait.impl.1");
            let target_name = imp.target.name.as_str();
            let (con, declared) = match target_name {
                "int" | "i64" => (Some(Con::Int), Vec::new()),
                "float" | "f64" => (Some(Con::Float), Vec::new()),
                "str" => (Some(Con::Str), Vec::new()),
                "char" => (Some(Con::Char), Vec::new()),
                "bool" => (Some(Con::Bool), Vec::new()),
                "List" => (Some(Con::List), vec![self.list_param]),
                n => match self.types.get(n).copied() {
                    Some(Def::Struct(s)) => (Some(Con::Struct(s)), self.struct_params[s].clone()),
                    Some(Def::Enum(e)) => (Some(Con::Enum(e)), self.enum_params[e].clone()),
                    None => {
                        if self.traits.contains_key(n) {
                            self.error_help(
                                "trait.impl.1",
                                imp.target.pos,
                                format!("`{n}` is a trait; `impl Trait for Type` names a type after `for`"),
                                "write the type that implements the trait",
                            );
                        } else {
                            self.unknown_type(n, imp.target.pos);
                        }
                        (None, Vec::new())
                    }
                },
            };
            let params = self.owner_params(&imp.target, &declared);
            let args: Vec<Ty> = params.iter().map(|&p| Ty::Param(p)).collect();
            let target = match con {
                Some(Con::Int) => Ty::Int,
                Some(Con::Float) => Ty::Float,
                Some(Con::Str) => Ty::Str,
                Some(Con::Char) => Ty::Char,
                Some(Con::Bool) => Ty::Bool,
                Some(Con::List) => Ty::List(Box::new(args[0].clone())),
                Some(Con::Struct(s)) => Ty::Struct(s, args),
                Some(Con::Enum(e)) => Ty::Enum(e, args),
                None => Ty::Err,
            };
            if tr == Some(self.std.hash) {
                self.error_help(
                    "trait.auto.5",
                    imp.pos,
                    "`Hash` can't be implemented: every type whose `==` is automatic is `Hash` already",
                    "remove this `impl`; custom hashing isn't designed yet (OQ-042)",
                );
                self.impl_info.push(ImplInfo { tr: None, target: Ty::Err, params: Vec::new() });
                continue;
            }
            if let (Some(t), Some(c)) = (tr, con) {
                let built_in_type = match c {
                    Con::Struct(s) => is_prelude(prog.structs[s].pos),
                    Con::Enum(e) => is_prelude(prog.enums[e].pos),
                    _ => true,
                };
                // The prelude's own impls (an error type's message, err.msg.1) are allowed.
                if is_prelude(prog.traits[t].pos) && built_in_type && !is_prelude(imp.pos) {
                    self.error(
                        "trait.impl.4",
                        imp.pos,
                        format!(
                            "`{}` and `{target_name}` are both built in, so this program can't implement one for the other",
                            imp.trait_name
                        ),
                    );
                } else if let Some(&other) = self.impl_of.get(&(t, c)) {
                    self.error_help(
                        "trait.impl.6",
                        imp.pos,
                        format!("`{target_name}` already implements `{}`", imp.trait_name),
                        format!("see line {}; a type implements a trait at most once", prog.impls[other].pos.line),
                    );
                } else {
                    self.impl_of.insert((t, c), self.impl_info.len());
                }
            }
            if !params.is_empty() {
                self.resolved.impl_types.insert(self.impl_info.len(), target.clone());
            }
            self.impl_info.push(ImplInfo { tr, target, params });
        }
    }

    /// The function index of trait `tr`'s method `name`.
    pub(super) fn trait_method(&self, tr: usize, name: &str) -> Option<usize> {
        self.prog.traits[tr].methods.iter().copied().find(|&m| self.prog.fns[m].name == name)
    }

    /// Whether `t` implements trait `tr` (trait.impl.1, trait.auto.1-2,
    /// trait.std.2-4, trait.any.5, gen.bound.2).
    pub(super) fn implements(&self, t: &Ty, tr: usize) -> bool {
        self.implements_in(t, tr, &mut Vec::new())
    }

    fn implements_in(&self, t: &Ty, tr: usize, seen: &mut Vec<(Con, usize)>) -> bool {
        // trait.auto.2: every value has a text form.
        if tr == self.std.to_str {
            return !matches!(t, Ty::Unit | Ty::Res(..));
        }
        let eq = self.std.eq;
        match t {
            Ty::Err | Ty::Never => true,
            Ty::Unit | Ty::Res(..) => false,
            // A bound `Ord` or `Hash` also provides `==` (trait.std.2, trait.auto.5).
            Ty::Param(p) => self.tparams[*p]
                .bounds
                .iter()
                .any(|&b| self.provides(b, tr) || (tr == eq && (b == self.std.ord || b == self.std.hash))),
            Ty::Any(a) => *a == tr && self.whole_trait_through_any(tr),
            // own.ref.6: references compare by identity, and aren't keys or ordered.
            Ty::Ref(_) => tr == eq,
            // type.num.14: sized numbers are `Eq` and `Ord`; every one but `f32` is `Hash`.
            Ty::Num(k) => tr == eq || tr == self.std.ord || (tr == self.std.hash && !k.is_float()),
            Ty::Union(_) => tr == eq || tr == self.std.hash,
            Ty::Opt(inner) => (tr == eq || tr == self.std.hash) && self.implements_in(inner, tr, seen),
            // type.map.4: maps and sets compare with `==`, and aren't keys.
            Ty::Map(k, v) => tr == eq && self.implements_in(k, eq, seen) && self.implements_in(v, eq, seen),
            Ty::Set(e) => tr == eq && self.implements_in(e, eq, seen),
            _ => {
                let Some(con) = con_of(t) else { return false };
                if let Some(&i) = self.impl_of.get(&(tr, con)) {
                    return self.impl_applies(i, t, con, seen);
                }
                if tr == eq {
                    return self.auto_eq(t, con, seen);
                }
                // trait.auto.5: `Hash` is automatic where `Eq` is automatic or
                // derived, except for floats.
                if tr == self.std.hash {
                    return !self.custom_eq(t) && !matches!(t, Ty::Float) && self.auto_hash(t, con, seen);
                }
                if tr == self.std.ord {
                    return match t {
                        Ty::Int | Ty::Float | Ty::Str | Ty::Char => true,
                        Ty::List(e) => self.implements_in(e, tr, seen),
                        _ => false,
                    };
                }
                false
            }
        }
    }

    /// Whether `impl` block `i` covers `t`: its type arguments satisfy the
    /// block's bounds (gen.method.2).
    fn impl_applies(&self, i: usize, t: &Ty, con: Con, seen: &mut Vec<(Con, usize)>) -> bool {
        let Some(tr) = self.impl_info[i].tr else { return true };
        if seen.contains(&(con, tr)) {
            return true;
        }
        seen.push((con, tr));
        let args: Vec<Ty> = match t {
            Ty::Struct(_, a) | Ty::Enum(_, a) => a.clone(),
            Ty::List(e) => vec![(**e).clone()],
            _ => Vec::new(),
        };
        let params = self.impl_info[i].params.clone();
        let ok = params
            .iter()
            .zip(&args)
            .all(|(&p, a)| self.tparams[p].bounds.iter().all(|&b| self.implements_in(a, b, seen)));
        seen.pop();
        ok
    }

    /// Whether `t` has an `impl Eq` with its own `fn eq` (not a derived one).
    fn custom_eq(&self, t: &Ty) -> bool {
        matches!(t, Ty::Struct(..) | Ty::Enum(..))
            && con_of(t)
                .and_then(|c| self.impl_of.get(&(self.std.eq, c)))
                .is_some_and(|&i| !self.prog.impls[i].methods.is_empty())
    }

    /// Why `t` isn't `Hash` (trait.auto.5), naming the part that isn't.
    pub(super) fn hash_cause(&self, t: &Ty) -> String {
        let hash = self.std.hash;
        match t {
            Ty::Float => "`float` isn't `Hash`, since NaN isn't equal to itself".into(),
            Ty::Num(k) if k.is_float() => "`f32` isn't `Hash`, since NaN isn't equal to itself".into(),
            Ty::Map(..) | Ty::Set(_) => "maps and sets aren't `Hash`".into(),
            Ty::Any(_) => "`any` values aren't `Hash`".into(),
            Ty::Param(p) => format!(
                "add the constraint where `{}` is declared, e.g. `[{} Hash]`",
                self.tparams[*p].name, self.tparams[*p].name
            ),
            Ty::List(e) | Ty::Opt(e) => self.hash_cause(e),
            Ty::Struct(_, args) | Ty::Enum(_, args) => {
                let name = self.type_name(t).unwrap_or_default();
                if self.custom_eq(t) {
                    return format!("`{name}` has its own `fn eq`, and custom hashing isn't designed yet (OQ-042)");
                }
                let fields = self.field_types_of(t, args);
                match fields.iter().find(|f| !self.implements(f, hash)) {
                    Some(f) => format!("`{name}` has a part of type {}: {}", self.show(f), self.hash_cause(f)),
                    None => format!("{} isn't `Hash`", self.show(t)),
                }
            }
            _ => format!("{} isn't `Hash`", self.show(t)),
        }
    }

    /// Structural hashing (trait.auto.5): every part is `Hash`.
    fn auto_hash(&self, t: &Ty, con: Con, seen: &mut Vec<(Con, usize)>) -> bool {
        let hash = self.std.hash;
        match t {
            Ty::Int | Ty::Bool | Ty::Str | Ty::Char => true,
            Ty::List(e) => self.implements_in(e, hash, seen),
            Ty::Struct(_, args) | Ty::Enum(_, args) => {
                if seen.contains(&(con, hash)) {
                    return true;
                }
                seen.push((con, hash));
                let ok = self.field_types_of(t, args).iter().all(|f| self.implements_in(f, hash, seen));
                seen.pop();
                ok
            }
            _ => false,
        }
    }

    /// Structural equality (trait.auto.1): every part supports `==`.
    fn auto_eq(&self, t: &Ty, con: Con, seen: &mut Vec<(Con, usize)>) -> bool {
        let eq = self.std.eq;
        match t {
            Ty::Int | Ty::Float | Ty::Bool | Ty::Str | Ty::Char => true,
            Ty::List(e) => self.implements_in(e, eq, seen),
            Ty::Struct(_, args) | Ty::Enum(_, args) => {
                if seen.contains(&(con, eq)) {
                    return true;
                }
                seen.push((con, eq));
                let ok = self.field_types_of(t, args).iter().all(|f| self.implements_in(f, eq, seen));
                seen.pop();
                ok
            }
            _ => false,
        }
    }

    /// The types of every field of a struct, or of every variant of an enum,
    /// with its type arguments filled in.
    fn field_types_of(&self, t: &Ty, args: &[Ty]) -> Vec<Ty> {
        let (params, fields): (&[usize], Vec<&Ty>) = match t {
            Ty::Struct(s, _) => {
                (&self.struct_params[*s], self.field_types.get(*s).map(|f| f.iter().collect()).unwrap_or_default())
            }
            Ty::Enum(e, _) => (
                &self.enum_params[*e],
                self.variant_types.get(*e).map(|v| v.iter().flatten().collect()).unwrap_or_default(),
            ),
            _ => return Vec::new(),
        };
        let binds: Vec<(usize, Ty)> = params.iter().copied().zip(args.iter().cloned()).collect();
        fields.into_iter().map(|f| subst(f, &binds)).collect()
    }

    /// Whether method `f` of trait `tr` can be called through `any`: `Self`
    /// appears only as its receiver (trait.any.4).
    pub(super) fn callable_through_any(&self, tr: usize, f: usize) -> bool {
        let p = self.trait_self[tr];
        let sig = &self.sigs[f];
        !mentions(&sig.ret, p) && sig.params.iter().all(|(_, t)| !mentions(t, p))
    }

    /// Whether every method of `tr` can be called through `any` (trait.any.5).
    fn whole_trait_through_any(&self, tr: usize) -> bool {
        self.prog.traits[tr].methods.iter().all(|&m| m < self.sigs.len() && self.callable_through_any(tr, m))
    }

    /// What `x.name(...)` can refer to when `x` has type `t` (trait.res.1).
    pub(super) fn candidates(&self, t: &Ty, name: &str) -> Vec<Cand> {
        let mut found = Vec::new();
        if super::builtin(t, name).is_some() {
            found.push(Cand::Builtin);
        }
        if let Some(i) = self.type_name(t).and_then(|tn| self.method_index(tn, name)) {
            found.push(Cand::Inherent(i));
        }
        for tr in 0..self.prog.traits.len() {
            if self.trait_method(tr, name).is_none() {
                continue;
            }
            // A built-in method of the same name takes precedence over a standard
            // trait's (`xs.max()`), and an inherent `eq` or `to_str` was already
            // reported (trait.auto.4, trait.res.2).
            let standard = [self.std.eq, self.std.ord, self.std.to_str, self.std.hash].contains(&tr);
            if !found.is_empty() && standard {
                continue;
            }
            let has = match t {
                Ty::Any(a) => *a == tr || tr == self.std.to_str,
                _ => self.implements(t, tr),
            };
            if has {
                found.push(Cand::Trait(tr));
            }
        }
        found
    }

    /// Help for a type that doesn't implement a trait it needs.
    pub(super) fn how_to_implement(&self, t: &Ty, tr: usize) -> Option<String> {
        let trait_name = &self.prog.traits[tr].name;
        match t {
            Ty::Param(p) => Some(format!(
                "add the constraint where `{}` is declared, e.g. `[{} {trait_name}]`",
                self.tparams[*p].name, self.tparams[*p].name
            )),
            _ if tr == self.std.hash => Some(self.hash_cause(t)),
            Ty::Struct(..) | Ty::Enum(..) if tr == self.std.eq => {
                Some(format!("{} has a field that can't be compared with `==`", self.show(t)))
            }
            Ty::Struct(s, _) if !is_prelude(self.prog.structs[*s].pos) => Some(self.impl_help(t, tr)),
            Ty::Enum(e, _) if !is_prelude(self.prog.enums[*e].pos) => Some(self.impl_help(t, tr)),
            _ => None,
        }
    }

    fn impl_help(&self, t: &Ty, tr: usize) -> String {
        let trait_name = &self.prog.traits[tr].name;
        let tn = self.type_name(t).unwrap_or_default();
        if tr == self.std.ord {
            format!("add `impl Ord for {tn}`; with no methods, it compares field by field")
        } else {
            format!("add `impl {trait_name} for {tn}` with the trait's methods indented below")
        }
    }

    /// `fn name(self, other Self) bool`, with `Self` replaced, for messages.
    fn sig_text(&self, f: usize, sig: &Sig) -> String {
        let decl = &self.prog.fns[f];
        let mut parts = Vec::new();
        if let Some(m) = sig.receiver {
            parts.push(if m { "mut self".to_string() } else { "self".to_string() });
        }
        for (p, (m, t)) in decl.params.iter().zip(&sig.params) {
            parts.push(format!("{}{} {}", if *m { "mut " } else { "" }, p.name, self.ty_name(t)));
        }
        let ret = match &sig.ret {
            Ty::Unit => String::new(),
            t => format!(" {}", self.ty_name(t)),
        };
        format!("fn {}({}){ret}", decl.name, parts.join(", "))
    }

    /// Check each `impl` block against its trait (trait.impl.3, trait.impl.5,
    /// trait.derive.1, trait.std.2).
    pub(super) fn check_impls(&mut self) {
        let prog = self.prog;
        for (i, imp) in prog.impls.iter().enumerate() {
            let Some(tr) = self.impl_info[i].tr else { continue };
            let target = self.impl_info[i].target.clone();
            let trait_decl = &prog.traits[tr];
            let binds = vec![(self.trait_self[tr], target.clone())];
            let mut defined = Vec::new();
            for &m in &imp.methods {
                let f = &prog.fns[m];
                if defined.contains(&f.name.as_str()) {
                    self.error("mod.unit.1", f.pos, format!("`{}` is defined more than once in this `impl`", f.name));
                    continue;
                }
                defined.push(f.name.as_str());
                let Some(t) = self.trait_method(tr, &f.name) else {
                    let names: Vec<&str> = trait_decl.methods.iter().map(|&k| prog.fns[k].name.as_str()).collect();
                    let mut d = Diagnostic::new(
                        "trait.impl.5",
                        f.pos,
                        format!("`{}` has no method `{}`", trait_decl.name, f.name),
                    );
                    d = match super::suggest(&f.name, &names) {
                        Some(s) => d.did_you_mean(&f.name, s),
                        None => d.help(format!(
                            "a method that isn't part of the trait is declared on its own: `fn {}.{}(...)`",
                            imp.target.name, f.name
                        )),
                    };
                    self.diags.push(d);
                    continue;
                };
                let want = self.sigs[t].clone();
                let want = Sig {
                    params: want.params.iter().map(|(m, ty)| (*m, subst(ty, &binds))).collect(),
                    ret: subst(&want.ret, &binds),
                    ..want
                };
                let got = self.sigs[m].clone();
                let matches = want.receiver == got.receiver
                    && want.params.len() == got.params.len()
                    && want.params.iter().zip(&got.params).all(|((wm, wt), (gm, gt))| wm == gm && same(wt, gt))
                    && self.same_result(&want.ret, &got.ret);
                if !matches {
                    let expected = self.sig_text(t, &want);
                    self.error_help(
                        "trait.impl.3",
                        f.pos,
                        format!("`{}` doesn't match its declaration in `{}`", f.name, trait_decl.name),
                        format!("write `{expected}`"),
                    );
                }
            }
            let built_in = is_prelude(trait_decl.pos);
            // ct.derive.2: a generator fills in an empty `impl` before checking
            // (`derive.rs`); one it couldn't fill is already reported.
            let by_generator = trait_decl.derive.is_some() && (imp.methods.is_empty() || trait_decl.methods.is_empty());
            if by_generator {
                // Nothing more to check.
            } else if built_in && imp.methods.is_empty() {
                self.check_derived(i, tr, &target);
            } else {
                let missing: Vec<usize> = trait_decl
                    .methods
                    .iter()
                    .copied()
                    .filter(|&m| prog.fns[m].body.is_none() && !defined.contains(&prog.fns[m].name.as_str()))
                    .collect();
                if !missing.is_empty() {
                    let names: Vec<&str> = missing.iter().map(|&m| prog.fns[m].name.as_str()).collect();
                    let shown: Vec<String> = missing
                        .iter()
                        .map(|&m| {
                            let s = self.sigs[m].clone();
                            let s = Sig {
                                params: s.params.iter().map(|(k, ty)| (*k, subst(ty, &binds))).collect(),
                                ret: subst(&s.ret, &binds),
                                ..s
                            };
                            self.sig_text(m, &s)
                        })
                        .collect();
                    let derive = if built_in { "; or remove every method to derive it" } else { "" };
                    self.error_help(
                        "trait.impl.3",
                        imp.pos,
                        format!("`impl {} for {}` is missing {}", trait_decl.name, imp.target.name, list_names(&names)),
                        format!(
                            "add {} to the block{derive}",
                            shown.iter().map(|s| format!("`{s}`")).collect::<Vec<_>>().join(", ")
                        ),
                    );
                }
            }
            // trait.std.2: a type that is `Ord` is also `Eq`.
            if tr == self.std.ord && !self.implements(&target, self.std.eq) {
                let msg = format!("{} implements `Ord`, so it must also support `==`", self.show(&target));
                self.error("trait.std.2", imp.pos, msg);
            }
            // trait.super.2: an `impl` of a trait needs its supertraits'.
            for s in self.supers[tr].clone() {
                if (tr == self.std.ord && s == self.std.eq) || self.implements(&target, s) {
                    continue;
                }
                let (tn, sn) = (&trait_decl.name, &prog.traits[s].name);
                self.error_help(
                    "trait.super.2",
                    imp.pos,
                    format!("{} implements `{tn}`, which requires `{sn}`, but it doesn't implement `{sn}`", self.show(&target)),
                    format!("add `impl {sn} for {}`", imp.target.name),
                );
            }
        }
    }

    /// Two result types are the same, including their error sets.
    fn same_result(&self, a: &Ty, b: &Ty) -> bool {
        match (a, b) {
            (Ty::Res(x, ea), Ty::Res(y, eb)) => {
                let (mut ea, mut eb) = (self.errs_of(ea), self.errs_of(eb));
                ea.sort_by_key(super::ty_key);
                eb.sort_by_key(super::ty_key);
                same(x, y) && ea == eb
            }
            _ => same(a, b),
        }
    }

    /// An empty `impl` of a standard trait derives it: every field's type
    /// must implement the trait (trait.derive.1).
    fn check_derived(&mut self, i: usize, tr: usize, target: &Ty) {
        let prog = self.prog;
        let imp = &prog.impls[i];
        let fields: Vec<(String, Ty)> = match target {
            Ty::Struct(s, args) => {
                let binds: Vec<(usize, Ty)> =
                    self.struct_params[*s].iter().copied().zip(args.iter().cloned()).collect();
                prog.structs[*s]
                    .fields
                    .iter()
                    .zip(&self.field_types[*s])
                    .map(|(f, t)| (f.name.clone(), subst(t, &binds)))
                    .collect()
            }
            Ty::Enum(e, args) => {
                let binds: Vec<(usize, Ty)> = self.enum_params[*e].iter().copied().zip(args.iter().cloned()).collect();
                prog.enums[*e]
                    .variants
                    .iter()
                    .zip(&self.variant_types[*e])
                    .flat_map(|(v, tys)| {
                        v.fields.iter().zip(tys).map(|(f, t)| (format!("{}.{}", v.name, f.name), subst(t, &binds)))
                    })
                    .collect::<Vec<_>>()
            }
            _ => return,
        };
        let trait_name = &prog.traits[tr].name;
        for (name, t) in fields {
            if !self.implements(&t, tr) {
                let msg = format!(
                    "`{trait_name}` can't be derived for `{}`: the field `{name}` is {}, which doesn't implement `{trait_name}`",
                    imp.target.name,
                    self.show(&t)
                );
                let help = self.how_to_implement(&t, tr).unwrap_or_else(|| {
                    format!("define the trait's methods in this `impl` instead of deriving `{trait_name}`")
                });
                self.error_help("trait.derive.1", imp.pos, msg, help);
            }
        }
    }
}
