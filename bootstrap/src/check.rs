//! Checker: names, types, places and control flow for the supported subset.
//!
//! Rules enforced: mod.main.1, mod.main.3, mod.unit.1, expr.name.1,
//! expr.bind.1-6, expr.block.2-3, expr.call.1-2, expr.if.1-2, expr.if.4,
//! expr.loop.1-5, expr.match.2-5, expr.op.2-3, type.bool.1, type.coll.2-6,
//! type.str.2-3, type.struct.3-6, type.enum.3, type.enum.5, trait.method.1-5,
//! trait.op.1, trait.op.3, own.param.1-2, own.excl.1-2, own.excl.5,
//! gram.generic.1, lex.str.2, type.opt.4-5, type.enum.6, type.str.4-5,
//! expr.loop.6, expr.match.6, err.decl.1-3, err.type.1-2, err.type.5-6,
//! err.prop.1-3, err.use.1, err.else.1-3, err.ctx.2, err.text.1.
//! All errors are collected in one pass (ai-author-wishlist 2.4).
//!
//! The checker also records which bare names are enum variants (type.enum.3),
//! since that depends on the expected type, which the interpreter doesn't track.

use std::collections::HashMap;

use crate::ast::{
    Arg, BinOp, Block, Cond, Else, Expr, Field, FnDecl, InterpPart, PathStep, Pattern, Program, Stmt, TypeKind,
    TypeRef, UnOp, place_path,
};
use crate::diag::{Diagnostic, Pos};
use crate::lexer::UNSUPPORTED;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ty {
    Int,
    Str,
    Bool,
    /// The result of an expression that produces no value (e.g. `print(...)`).
    Unit,
    /// A block that always leaves through `return`, `break` or `continue` (expr.block.3).
    Never,
    /// An error was already reported here; suppresses follow-on errors.
    Err,
    Struct(usize),
    Enum(usize),
    List(Box<Ty>),
    /// `T?` (type.opt.2)
    Opt(Box<Ty>),
    /// `T ! E`; `T` is `Unit` for a bare `!` (err.type.1-2).
    Res(Box<Ty>, Errs),
    /// One of several error types, e.g. the `err` of `else err` (err.type.5).
    Union(Vec<Ty>),
}

/// The error part of `T ! E`: an explicit set, or the inferred set of a
/// function, by index (err.type.2, err.type.6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Errs {
    Set(Vec<Ty>),
    Inferred(usize),
}

/// Sort key that keeps error sets in a canonical order.
fn ty_key(t: &Ty) -> (u8, usize) {
    match t {
        Ty::Struct(i) => (0, *i),
        Ty::Enum(i) => (1, *i),
        _ => (2, 0),
    }
}

/// Facts the interpreter needs from the checker.
#[derive(Debug, Default)]
pub struct Resolved {
    /// Bare enum variants, by the position of the name: (enum, variant).
    pub variants: HashMap<Pos, (usize, usize)>,
}

/// Names available without import (mod.main.3, ADR 0084).
const PRELUDE: &[&str] = &["print"];

const BUILTIN_TYPES: &[&str] = &["int", "i64", "str", "bool", "List"];

/// Type names the spec defines that the bootstrap doesn't support yet.
const SPEC_TYPES_NOT_YET: &[&str] =
    &["i8", "i16", "i32", "u8", "u16", "u32", "u64", "f32", "f64", "float", "char", "Map", "Set", "Option"];

/// Provisional list methods (type.coll.5-7, ADR 0116, ADR 0119).
const LIST_METHODS: &[&str] = &["len", "push", "swap"];

pub fn check(prog: &Program) -> (Vec<Diagnostic>, Resolved) {
    let mut c = Checker {
        prog,
        diags: Vec::new(),
        types: HashMap::new(),
        field_types: Vec::new(),
        variant_types: Vec::new(),
        fns: HashMap::new(),
        methods: HashMap::new(),
        sigs: Vec::new(),
        inferred: vec![Vec::new(); prog.fns.len()],
        resolved: Resolved::default(),
        locals: Vec::new(),
        ret: Ty::Unit,
        loops: 0,
    };
    c.collect_types();
    c.collect_fns();
    c.check_main();
    // err.type.6: inferred error sets are the least fixed point over all
    // functions, found by re-checking them silently until nothing changes.
    let inferring: Vec<usize> =
        (0..prog.fns.len()).filter(|i| matches!(&c.sigs[*i].ret, Ty::Res(_, Errs::Inferred(_)))).collect();
    if !inferring.is_empty() {
        for _ in 0..=prog.fns.len() {
            let before = c.inferred.clone();
            let saved = c.diags.len();
            for &i in &inferring {
                c.check_fn(i, &prog.fns[i]);
            }
            c.diags.truncate(saved);
            if c.inferred == before {
                break;
            }
        }
    }
    for (i, f) in prog.fns.iter().enumerate() {
        c.check_fn(i, f);
    }
    (c.diags, c.resolved)
}

/// A function's signature: receiver (`Some(true)` for `mut self`), parameters
/// (`true` for `mut`), and return type (`Unit` when there is none).
#[derive(Clone)]
struct Sig {
    receiver: Option<bool>,
    params: Vec<(bool, Ty)>,
    ret: Ty,
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Binding,
    Param,
    SelfParam,
    LoopVar,
}

struct Local {
    name: String,
    ty: Ty,
    mutable: bool,
    kind: Kind,
}

struct Checker<'p> {
    prog: &'p Program,
    diags: Vec<Diagnostic>,
    /// User-declared types by name; the first declaration wins.
    types: HashMap<&'p str, Ty>,
    /// Field types, parallel to `prog.structs[i].fields`.
    field_types: Vec<Vec<Ty>>,
    /// Field types of each enum's variants, parallel to `prog.enums`.
    variant_types: Vec<Vec<Vec<Ty>>>,
    /// Free functions: name to index in `prog.fns` and `sigs`.
    fns: HashMap<&'p str, usize>,
    /// Methods and associated functions: type name, then function name.
    methods: HashMap<&'p str, HashMap<&'p str, usize>>,
    sigs: Vec<Sig>,
    /// The inferred error set of each function with a `!T` result (err.type.2).
    inferred: Vec<Vec<Ty>>,
    resolved: Resolved,
    /// Locals of the function being checked, innermost last.
    locals: Vec<Local>,
    ret: Ty,
    loops: usize,
}

/// `a` and `b` are the same type, or an error already reported makes it moot.
fn same(a: &Ty, b: &Ty) -> bool {
    match (a, b) {
        (Ty::Err, _) | (_, Ty::Err) => true,
        (Ty::List(x), Ty::List(y)) => same(x, y),
        _ => a == b,
    }
}

fn list_names(names: &[&str]) -> String {
    let quoted: Vec<String> = names.iter().map(|n| format!("`{n}`")).collect();
    match quoted.len() {
        0 => String::new(),
        1 => quoted[0].clone(),
        n => format!("{} and {}", quoted[..n - 1].join(", "), quoted[n - 1]),
    }
}

impl<'p> Checker<'p> {
    fn error(&mut self, rule: &'static str, pos: Pos, msg: impl Into<String>) {
        self.diags.push(Diagnostic::new(rule, pos, msg));
    }

    fn error_help(&mut self, rule: &'static str, pos: Pos, msg: impl Into<String>, help: impl Into<String>) {
        self.diags.push(Diagnostic::new(rule, pos, msg).help(help));
    }

    fn unsupported(&mut self, pos: Pos, what: impl Into<String>) {
        self.diags.push(Diagnostic::new(UNSUPPORTED, pos, what));
    }

    // ---- Declarations ----

    fn collect_types(&mut self) {
        let prog = self.prog;
        let decls = (prog.structs.iter().enumerate().map(|(i, s)| (s.name.as_str(), s.pos, Ty::Struct(i))))
            .chain(prog.enums.iter().enumerate().map(|(i, e)| (e.name.as_str(), e.pos, Ty::Enum(i))));
        for (name, pos, ty) in decls {
            if BUILTIN_TYPES.contains(&name) || SPEC_TYPES_NOT_YET.contains(&name) {
                self.error("mod.unit.1", pos, format!("`{name}` is already a built-in type"));
            } else if self.types.contains_key(name) {
                self.error("mod.unit.1", pos, format!("`{name}` is declared more than once in this module"));
            } else {
                self.types.insert(name, ty);
            }
        }
        for e in &prog.enums {
            let mut variants = Vec::new();
            for (i, v) in e.variants.iter().enumerate() {
                if e.variants[..i].iter().any(|w| w.name == v.name) {
                    self.error("type.enum.5", v.pos, format!("variant `{}` is declared twice in `{}`", v.name, e.name));
                }
                variants.push(self.field_list(&v.fields, &format!("{}.{}", e.name, v.name)));
            }
            self.variant_types.push(variants);
        }
        for s in &prog.structs {
            let tys = self.field_list(&s.fields, &s.name);
            self.field_types.push(tys);
        }
    }

    /// Resolve the types of a field list, reporting repeated names (type.struct.5).
    fn field_list(&mut self, fields: &[Field], owner: &str) -> Vec<Ty> {
        let mut tys = Vec::new();
        for (i, f) in fields.iter().enumerate() {
            if fields[..i].iter().any(|g| g.name == f.name) {
                self.error("type.struct.5", f.pos, format!("field `{}` is declared twice in `{owner}`", f.name));
            }
            tys.push(self.resolve_type(&f.ty));
        }
        tys
    }

    fn collect_fns(&mut self) {
        let prog = self.prog;
        for (i, f) in prog.fns.iter().enumerate() {
            let params = f.params.iter().map(|p| (p.mutable, self.resolve_type(&p.ty))).collect();
            let mut ret = f.ret.as_ref().map_or(Ty::Unit, |t| self.resolve_ret(t, i));
            // A `main` with a wrong return type is reported once (mod.main.1);
            // its body is then checked as returning nothing.
            if f.owner.is_none() && f.name == "main" && !matches!(&ret, Ty::Res(ok, _) if **ok == Ty::Unit) {
                ret = Ty::Unit;
            }
            self.sigs.push(Sig { receiver: f.receiver.map(|r| r.mutable), params, ret });
            let name = f.name.as_str();
            match &f.owner {
                None if PRELUDE.contains(&name) => self.unsupported(
                    f.pos,
                    format!("redeclaring the prelude function `{name}` is not supported by the bootstrap"),
                ),
                None if self.fns.contains_key(name) || self.types.contains_key(name) => {
                    self.error("mod.unit.1", f.pos, format!("`{name}` is declared more than once in this module"));
                }
                None => {
                    self.fns.insert(name, i);
                }
                Some((owner, opos)) => {
                    if !self.types.contains_key(owner.as_str()) {
                        if BUILTIN_TYPES.contains(&owner.as_str()) || SPEC_TYPES_NOT_YET.contains(&owner.as_str()) {
                            self.error(
                                "trait.method.5",
                                *opos,
                                format!("methods of `{owner}` can only be declared where `{owner}` is declared"),
                            );
                        } else {
                            self.unknown_type(owner, *opos);
                        }
                        continue;
                    }
                    let table = self.methods.entry(owner.as_str()).or_default();
                    if table.contains_key(name) {
                        self.error("mod.unit.1", f.pos, format!("`{owner}.{name}` is declared more than once"));
                    } else {
                        table.insert(name, i);
                    }
                }
            }
        }
    }

    fn check_main(&mut self) {
        match self.prog.fns.iter().find(|f| f.name == "main" && f.owner.is_none()) {
            None => self.diags.push(
                Diagnostic::new("mod.main.1", Pos { line: 1, col: 1 }, "program has no `main` function")
                    .help("add:\n    fn main()\n        print(\"Hello, world!\")"),
            ),
            Some(f) => {
                if !f.params.is_empty() {
                    self.error("mod.main.1", f.params[0].pos, "`main` takes no parameters");
                }
                if let Some(ret) = &f.ret
                    && !matches!(ret.kind, TypeKind::Fallible { ok: None, .. })
                {
                    self.error_help(
                        "mod.main.1",
                        ret.pos,
                        "`main` returns nothing, or only an error",
                        "write `fn main()`, or `fn main() !` if it can fail",
                    );
                }
            }
        }
    }

    /// A function's return type, which may be fallible (err.type.1-2, err.decl.2-3).
    fn resolve_ret(&mut self, t: &TypeRef, fn_index: usize) -> Ty {
        let TypeKind::Fallible { ok, errors } = &t.kind else {
            return self.resolve_type(t);
        };
        let ok_ty = ok.as_ref().map_or(Ty::Unit, |o| self.resolve_type(o));
        if self.is_error(&ok_ty) {
            self.error_help(
                "err.decl.3",
                t.pos,
                format!("{} is an error type, so it can't also be the success value", self.show(&ok_ty)),
                "return it as an error, or declare the type without `error`",
            );
        }
        let Some(errors) = errors else {
            return Ty::Res(Box::new(ok_ty), Errs::Inferred(fn_index));
        };
        let mut set = Vec::new();
        for e in errors {
            let et = self.resolve_type(e);
            if et != Ty::Err && !self.is_error(&et) {
                self.error_help(
                    "err.decl.2",
                    e.pos,
                    format!("{} isn't an error type", self.show(&et)),
                    "declare it with `error struct` or `error enum`",
                );
            } else if et != Ty::Err && !set.contains(&et) {
                set.push(et);
            }
        }
        set.sort_by_key(ty_key);
        Ty::Res(Box::new(ok_ty), Errs::Set(set))
    }

    fn resolve_type(&mut self, t: &TypeRef) -> Ty {
        let (name, args) = match &t.kind {
            TypeKind::Named { name, args } => (name, args),
            TypeKind::Optional(inner) => return Ty::Opt(Box::new(self.resolve_type(inner))),
            TypeKind::Fallible { .. } => {
                self.unsupported(
                    t.pos,
                    "a type that can fail is only supported as a function's return type in the bootstrap (spec err.type.1)",
                );
                return Ty::Err;
            }
        };
        let ty = match name.as_str() {
            "int" | "i64" => Ty::Int,
            "str" => Ty::Str,
            "bool" => Ty::Bool,
            "List" => {
                if args.len() != 1 {
                    self.error_help(
                        "gram.generic.1",
                        t.pos,
                        format!("`List` takes 1 type argument, but {} given", was_were(args.len())),
                        "write e.g. `List[int]`",
                    );
                    return Ty::Err;
                }
                return Ty::List(Box::new(self.resolve_type(&args[0])));
            }
            n if SPEC_TYPES_NOT_YET.contains(&n) => {
                self.unsupported(t.pos, format!("type `{n}` is not supported by the bootstrap yet (spec 04-types)"));
                return Ty::Err;
            }
            n => match self.types.get(n) {
                Some(ty) => ty.clone(),
                None => {
                    self.unknown_type(n, t.pos);
                    return Ty::Err;
                }
            },
        };
        if !args.is_empty() {
            self.error("gram.generic.1", t.pos, format!("`{name}` takes no type arguments"));
        }
        ty
    }

    /// Whether a type was declared with `error` (err.decl.1).
    fn is_error(&self, t: &Ty) -> bool {
        match t {
            Ty::Struct(i) => self.prog.structs[*i].is_error,
            Ty::Enum(i) => self.prog.enums[*i].is_error,
            _ => false,
        }
    }

    fn errs_of(&self, e: &Errs) -> Vec<Ty> {
        match e {
            Errs::Set(v) => v.clone(),
            Errs::Inferred(j) => self.inferred[*j].clone(),
        }
    }

    /// Admit the errors `got` into the set `want`: an inferred set grows, and
    /// an explicit set must already contain them. Returns the missing ones.
    fn add_errors(&mut self, want: &Errs, got: &[Ty]) -> Vec<Ty> {
        match want {
            Errs::Inferred(j) => {
                for g in got {
                    if !self.inferred[*j].contains(g) {
                        self.inferred[*j].push(g.clone());
                        self.inferred[*j].sort_by_key(ty_key);
                    }
                }
                Vec::new()
            }
            Errs::Set(v) => got.iter().filter(|g| !v.contains(g)).cloned().collect(),
        }
    }

    /// Whether a value of type `got` is accepted where `want` is expected,
    /// including optional wrapping (type.opt.5), failure values (err.decl.3),
    /// and leaving blocks (expr.block.3). An inferred error set grows to admit
    /// the errors of `got` (err.type.2).
    fn accepts(&mut self, want: &Ty, got: &Ty) -> bool {
        match (want, got) {
            (_, Ty::Never | Ty::Err) | (Ty::Err, _) => true,
            (Ty::Opt(w), Ty::Opt(g)) => same(w, g),
            (Ty::Opt(w), g) => self.accepts(w, g),
            (Ty::Res(w, we), Ty::Res(g, ge)) => {
                let errs = self.errs_of(ge);
                same(w, g) && self.add_errors(we, &errs).is_empty()
            }
            (Ty::Res(_, we), Ty::Union(m)) => self.add_errors(we, m).is_empty(),
            (Ty::Res(_, we), g) if self.is_error(g) => {
                let we = we.clone();
                self.add_errors(&we, std::slice::from_ref(g)).is_empty()
            }
            (Ty::Res(w, _), g) => self.accepts(w, g),
            (Ty::Union(w), Ty::Union(g)) => g.iter().all(|t| w.contains(t)),
            (Ty::Union(w), g) => w.contains(g),
            _ => same(want, got),
        }
    }

    /// Report a value of type `got` where `want` was expected, under `rule`.
    /// An unhandled failure is reported as such (err.use.1).
    fn mismatch(&mut self, rule: &'static str, pos: Pos, want: &Ty, got: &Ty) {
        if !matches!(want, Ty::Res(..)) && self.reject_unhandled(got, pos) {
            return;
        }
        let msg = format!("expected {}, found {}", self.show(want), self.show(got));
        match (want, got) {
            (Ty::Res(_, we), _) => {
                let got_errs = match got {
                    Ty::Res(_, ge) => self.errs_of(ge),
                    Ty::Union(m) => m.clone(),
                    g if self.is_error(g) => vec![g.clone()],
                    _ => Vec::new(),
                };
                let missing: Vec<Ty> = match we {
                    Errs::Set(v) => got_errs.into_iter().filter(|g| !v.contains(g)).collect(),
                    Errs::Inferred(_) => Vec::new(),
                };
                if missing.is_empty() {
                    self.error(rule, pos, msg);
                } else {
                    let names: Vec<String> = missing.iter().map(|t| self.ty_name(t)).collect();
                    let names: Vec<&str> = names.iter().map(String::as_str).collect();
                    self.error_help(
                        rule,
                        pos,
                        format!("{msg}: the error type doesn't include {}", list_names(&names)),
                        "add them to the return type, e.g. `T ! A | B`, or use `!T` to infer the errors",
                    );
                }
            }
            (w, Ty::Opt(g)) if same(w, g) => self.error_help(
                rule,
                pos,
                msg,
                "this value may be absent; supply a fallback with `x := value else fallback`",
            ),
            _ => self.error(rule, pos, msg),
        }
    }

    /// Report a failure that isn't handled (err.use.1); true if `t` is one.
    fn reject_unhandled(&mut self, t: &Ty, pos: Pos) -> bool {
        if !matches!(t, Ty::Res(..)) {
            return false;
        }
        self.error_help(
            "err.use.1",
            pos,
            "this can fail, and the failure isn't handled",
            "pass the error on with `?`, or handle it with `x := ... else ...`",
        );
        true
    }

    /// The success type of a value that may be a failure.
    fn ok_type(t: &Ty) -> Ty {
        match t {
            Ty::Res(ok, _) => (**ok).clone(),
            t => t.clone(),
        }
    }

    fn unknown_type(&mut self, name: &str, pos: Pos) {
        let candidates: Vec<&str> = BUILTIN_TYPES.iter().copied().chain(self.types.keys().copied()).collect();
        let mut d = Diagnostic::new("expr.name.1", pos, format!("unknown type `{name}`"));
        if let Some(s) = suggest(name, &candidates) {
            d = d.help(format!("did you mean `{s}`?"));
        }
        self.diags.push(d);
    }

    fn ty_name(&self, t: &Ty) -> String {
        match t {
            Ty::Int => "int".into(),
            Ty::Str => "str".into(),
            Ty::Bool => "bool".into(),
            Ty::Unit | Ty::Never => "no value".into(),
            Ty::Err => "?".into(),
            Ty::Struct(i) => self.prog.structs[*i].name.clone(),
            Ty::Enum(i) => self.prog.enums[*i].name.clone(),
            Ty::List(e) => format!("List[{}]", self.ty_name(e)),
            Ty::Opt(t) => format!("{}?", self.ty_name(t)),
            Ty::Res(ok, errs) => {
                let set: Vec<String> = self.errs_of(errs).iter().map(|t| self.ty_name(t)).collect();
                let ok = if **ok == Ty::Unit { String::new() } else { format!("{} ", self.ty_name(ok)) };
                if set.is_empty() { format!("!{}", ok.trim_end()) } else { format!("{ok}! {}", set.join(" | ")) }
            }
            Ty::Union(m) => m.iter().map(|t| self.ty_name(t)).collect::<Vec<_>>().join(" | "),
        }
    }

    /// A type for messages: "`int`", or "no value".
    fn show(&self, t: &Ty) -> String {
        match t {
            Ty::Unit | Ty::Never => "no value".into(),
            _ => format!("`{}`", self.ty_name(t)),
        }
    }

    fn type_name(&self, t: &Ty) -> Option<&'p str> {
        match t {
            Ty::Struct(i) => Some(self.prog.structs[*i].name.as_str()),
            Ty::Enum(i) => Some(self.prog.enums[*i].name.as_str()),
            _ => None,
        }
    }

    fn method_index(&self, type_name: &str, name: &str) -> Option<usize> {
        self.methods.get(type_name).and_then(|m| m.get(name)).copied()
    }

    fn variant_index(&self, e: usize, name: &str) -> Option<usize> {
        self.prog.enums[e].variants.iter().position(|v| v.name == name)
    }

    // ---- Functions and blocks ----

    fn check_fn(&mut self, idx: usize, f: &'p FnDecl) {
        let sig = self.sigs[idx].clone();
        self.locals.clear();
        self.loops = 0;
        self.ret = sig.ret.clone();
        if let (Some(r), Some((owner, _))) = (&f.receiver, &f.owner) {
            let ty = self.types.get(owner.as_str()).cloned().unwrap_or(Ty::Err);
            self.locals.push(Local { name: "self".into(), ty, mutable: r.mutable, kind: Kind::SelfParam });
        }
        for (p, (_, ty)) in f.params.iter().zip(&sig.params) {
            if self.declare(&p.name, p.pos) {
                self.locals.push(Local { name: p.name.clone(), ty: ty.clone(), mutable: p.mutable, kind: Kind::Param });
            }
        }
        let need = !Self::returns_nothing(&sig.ret);
        let got = self.block(&f.body, need.then_some(&sig.ret), need);
        if need && !self.accepts(&sig.ret, &got) {
            let ret_pos = f.ret.as_ref().map_or(f.pos, |t| t.pos);
            let (at, msg) = match f.body.last() {
                Some(Stmt::Expr(e)) if got != Ty::Unit => (
                    e.pos(),
                    format!(
                        "`{}` returns {}, but its body ends with {}",
                        f.full_name(),
                        self.show(&sig.ret),
                        self.show(&got)
                    ),
                ),
                _ => (
                    ret_pos,
                    format!("`{}` must end with an expression of type {}", f.full_name(), self.show(&sig.ret)),
                ),
            };
            if matches!(got, Ty::Res(..) | Ty::Union(_)) || self.is_error(&got) {
                self.mismatch("expr.block.2", at, &sig.ret, &got);
            } else {
                self.error("expr.block.2", at, msg);
            }
        }
    }

    /// A function result with no success value: none at all, or a bare `!` or `! E`.
    fn returns_nothing(ret: &Ty) -> bool {
        match ret {
            Ty::Unit => true,
            Ty::Res(ok, _) => **ok == Ty::Unit,
            _ => false,
        }
    }

    /// Check a block. Its type is its last statement's (expr.block.1), or
    /// `Never` if a statement always leaves (expr.block.3).
    fn block(&mut self, stmts: &'p [crate::ast::Stmt], expect: Option<&Ty>, need: bool) -> Ty {
        let mark = self.locals.len();
        let mut ty = Ty::Unit;
        let mut leaves = false;
        for (i, s) in stmts.iter().enumerate() {
            let last = i + 1 == stmts.len();
            ty = self.stmt(s, if last { expect } else { None }, last && need);
            leaves |= ty == Ty::Never;
            // err.use.1: a failure can't be discarded.
            if (!last || !need) && matches!(s, Stmt::Expr(_)) && self.reject_unhandled(&ty, s.pos()) {
                ty = Ty::Err;
            }
        }
        self.locals.truncate(mark);
        if leaves { Ty::Never } else { ty }
    }

    fn stmt(&mut self, s: &'p Stmt, expect: Option<&Ty>, need: bool) -> Ty {
        match s {
            Stmt::Let { name, pos, mutable, value, otherwise } => {
                let vt = self.expr(value, None, true);
                let ty = match otherwise {
                    Some(o) => self.else_part(&vt, value.pos(), o),
                    None if self.reject_unhandled(&vt, value.pos()) => Self::ok_type(&vt),
                    None => vt,
                };
                if ty == Ty::Unit {
                    self.error(
                        "expr.bind.1",
                        value.pos(),
                        format!("`{name}` would be bound to an expression that produces no value"),
                    );
                }
                if self.declare(name, *pos) {
                    self.locals.push(Local { name: name.clone(), ty, mutable: *mutable, kind: Kind::Binding });
                }
                Ty::Unit
            }
            Stmt::Assign { target, op, value, pos } => {
                self.assign(target, *op, value, *pos);
                Ty::Unit
            }
            Stmt::While { cond, body, .. } => {
                self.condition(cond);
                self.loops += 1;
                self.block(body, None, false);
                self.loops -= 1;
                Ty::Unit
            }
            Stmt::For { var, var_pos, mutable, iter, body, .. } => {
                let elem = if *mutable { self.mut_iterable(iter, var, body) } else { self.iterable(iter) };
                let mark = self.locals.len();
                if self.declare(var, *var_pos) {
                    self.locals.push(Local { name: var.clone(), ty: elem, mutable: *mutable, kind: Kind::LoopVar });
                }
                self.loops += 1;
                self.block(body, None, false);
                self.loops -= 1;
                self.locals.truncate(mark);
                Ty::Unit
            }
            Stmt::Break(pos) | Stmt::Continue(pos) => {
                if self.loops == 0 {
                    let word = if matches!(s, Stmt::Break(_)) { "break" } else { "continue" };
                    self.error("expr.loop.5", *pos, format!("`{word}` outside a loop"));
                }
                Ty::Never
            }
            Stmt::Return(value, pos) => {
                self.return_stmt(value.as_ref(), *pos);
                Ty::Never
            }
            Stmt::Expr(e) => self.expr(e, expect, need),
        }
    }

    fn return_stmt(&mut self, value: Option<&'p Expr>, pos: Pos) {
        let ret = self.ret.clone();
        match value {
            None if !Self::returns_nothing(&ret) => {
                let msg = format!("this function returns {}, so `return` needs a value", self.show(&ret));
                self.error("expr.block.2", pos, msg);
            }
            None => {}
            Some(v) if ret == Ty::Unit => {
                self.expr(v, None, true);
                self.error_help(
                    "expr.block.2",
                    v.pos(),
                    "this function doesn't return a value",
                    "remove the value, or declare a return type after `)`",
                );
            }
            Some(v) => {
                let t = self.expr(v, Some(&ret), true);
                if !self.accepts(&ret, &t) {
                    self.mismatch("expr.block.2", v.pos(), &ret, &t);
                }
            }
        }
    }

    /// The `else` part of `x := e else ...`; returns the type bound to `x`
    /// (err.else.1-3).
    fn else_part(&mut self, vt: &Ty, pos: Pos, o: &'p Else) -> Ty {
        let (ok, errors) = match vt {
            Ty::Opt(t) => ((**t).clone(), None),
            Ty::Res(t, e) => ((**t).clone(), Some(self.errs_of(e))),
            Ty::Err => (Ty::Err, None),
            other => {
                let msg = format!(
                    "`else` handles a value that can be absent (`T?`) or fail (`T ! E`), but this is {}",
                    self.show(other)
                );
                self.error("err.else.1", pos, msg);
                (other.clone(), None)
            }
        };
        match o {
            Else::Value(d) => {
                let dt = self.expr(d, Some(&ok), true);
                if !self.accepts(&ok, &dt) {
                    self.mismatch("err.else.1", d.pos(), &ok, &dt);
                }
            }
            Else::Leave(stmt) => {
                self.stmt(stmt, None, false);
            }
            Else::Block { err, body, pos } => {
                let mark = self.locals.len();
                if let Some((name, npos)) = err {
                    match &errors {
                        Some(set) => {
                            let t = if set.len() == 1 { set[0].clone() } else { Ty::Union(set.clone()) };
                            if self.declare(name, *npos) {
                                self.locals.push(Local {
                                    name: name.clone(),
                                    ty: t,
                                    mutable: false,
                                    kind: Kind::Binding,
                                });
                            }
                        }
                        None if ok != Ty::Err => self.error_help(
                            "err.else.3",
                            *npos,
                            "only a failure has an error to bind; an optional is just absent",
                            "write `else` without a name",
                        ),
                        None => {}
                    }
                }
                let t = self.block(body, Some(&ok), true);
                self.locals.truncate(mark);
                if !self.accepts(&ok, &t) {
                    let msg = format!(
                        "this `else` block must leave (with `return`, `break` or `continue`) or end with {}",
                        self.show(&ok)
                    );
                    self.error("err.else.2", *pos, msg);
                }
            }
        }
        ok
    }

    /// Check a new local name against the no-shadowing rules; true if it can be added.
    fn declare(&mut self, name: &str, pos: Pos) -> bool {
        if let Some(l) = self.locals.iter().rev().find(|l| l.name == name) {
            let help = if l.mutable {
                format!("`{name}` is a `mut` variable; to change it, write `{name} = ...`")
            } else {
                "choose a different name; names can't be shadowed".into()
            };
            self.error_help("expr.bind.4", pos, format!("`{name}` is already declared in this function"), help);
            return false;
        }
        let what = if self.fns.contains_key(name) || PRELUDE.contains(&name) {
            "a function"
        } else if self.types.contains_key(name) || BUILTIN_TYPES.contains(&name) {
            "a type"
        } else {
            return true;
        };
        self.error_help(
            "expr.bind.5",
            pos,
            format!("`{name}` is already {what} name"),
            "choose a different name for the local",
        );
        false
    }

    fn local(&self, name: &str) -> Option<&Local> {
        self.locals.iter().rev().find(|l| l.name == name)
    }

    fn condition(&mut self, cond: &'p Expr) {
        let t = self.expr(cond, Some(&Ty::Bool), true);
        if self.reject_unhandled(&t, cond.pos()) {
            return;
        }
        if !self.accepts(&Ty::Bool, &t) {
            let msg = format!("a condition must be `bool`, found {}", self.show(&t));
            match t {
                Ty::Int => self.error_help("expr.if.2", cond.pos(), msg, "compare explicitly, e.g. `n != 0`"),
                _ => self.error("expr.if.2", cond.pos(), msg),
            }
        }
    }

    /// The element type of a `for` loop's iterable (expr.loop.4).
    fn iterable(&mut self, iter: &'p Expr) -> Ty {
        if let Expr::Range { lo, hi, .. } = iter {
            self.range_bounds(lo, hi);
            return Ty::Int;
        }
        match self.expr(iter, None, true) {
            Ty::List(t) => *t,
            Ty::Err => Ty::Err,
            t if self.reject_unhandled(&t, iter.pos()) => Ty::Err,
            t => {
                let msg = format!("`for` can't iterate over {}", self.show(&t));
                if t == Ty::Int {
                    self.error_help("expr.loop.4", iter.pos(), msg, "iterate over a range, e.g. `0..n`");
                } else {
                    self.error("expr.loop.4", iter.pos(), msg);
                }
                Ty::Err
            }
        }
    }

    /// `for mut x in e`: `e` is a mutable list place that the body doesn't
    /// otherwise use (expr.loop.6, own.excl.5). Returns the element type.
    fn mut_iterable(&mut self, iter: &'p Expr, var: &str, body: &'p [Stmt]) -> Ty {
        if matches!(iter, Expr::Range { .. }) {
            self.error_help(
                "expr.loop.6",
                iter.pos(),
                "`for mut` changes the elements of a list, and a range has none",
                "remove the `mut`",
            );
            return Ty::Int;
        }
        let t = self.expr(iter, None, true);
        let elem = match t {
            Ty::List(e) => *e,
            Ty::Err => return Ty::Err,
            other => {
                let msg = format!("`for mut` needs a list, found {}", self.show(&other));
                self.error("expr.loop.6", iter.pos(), msg);
                return Ty::Err;
            }
        };
        self.require_mutable(iter, Some("expr.loop.6"), "changed by `for mut`");
        if let Some(at) = self.overlapping_use(iter, body) {
            self.error_help(
                "own.excl.5",
                at,
                "the loop body can't use this while `for mut` is changing its elements",
                format!("change the current element through `{var}`"),
            );
        }
        elem
    }

    fn range_bounds(&mut self, lo: &'p Expr, hi: &'p Expr) {
        for bound in [lo, hi] {
            let t = self.expr(bound, Some(&Ty::Int), true);
            if !self.accepts(&Ty::Int, &t) {
                let msg = format!("range bounds must be `int`, found {}", self.show(&t));
                self.error("expr.loop.3", bound.pos(), msg);
            }
        }
    }

    // ---- Assignment and places ----

    fn assign(&mut self, target: &'p Expr, op: Option<BinOp>, value: &'p Expr, pos: Pos) {
        if let Expr::Name(n, npos) = target
            && self.local(n).is_none()
        {
            self.expr(value, None, true);
            if n == "self" {
                self.expr(target, None, true);
            } else {
                self.error_help(
                    "expr.bind.3",
                    *npos,
                    format!("`{n}` isn't declared"),
                    format!("declare it with `mut {n} := ...`"),
                );
            }
            return;
        }
        let rooted_in_local = place_path(target).is_some_and(|(root, _)| self.local(root).is_some());
        let tt = self.expr(target, None, true);
        if !rooted_in_local {
            if tt != Ty::Err {
                self.error(
                    "expr.bind.6",
                    target.pos(),
                    "only a variable, or a field or element of one, can be assigned",
                );
            }
            self.expr(value, None, true);
            return;
        }
        self.require_mutable(target, None, "assigned");
        let vt = self.expr(value, Some(&tt), true);
        match op {
            Some(op) => {
                self.operator(op, &tt, &vt, pos);
            }
            None if !self.accepts(&tt, &vt) => self.mismatch("expr.bind.6", value.pos(), &tt, &vt),
            None => {}
        }
    }

    /// Report if `e` isn't a mutable place (own.place.1). `rule` is the rule
    /// for this context; without one, the rule is chosen by the kind of root.
    fn require_mutable(&mut self, e: &'p Expr, rule: Option<&'static str>, action: &str) {
        let Some((root, _)) = place_path(e) else {
            let rule = rule.unwrap_or("expr.bind.6");
            self.error(rule, e.pos(), format!("only a variable, or a field or element of one, can be {action}"));
            return;
        };
        let Some(l) = self.local(root) else { return };
        if l.mutable {
            return;
        }
        let (default_rule, msg, help) = match l.kind {
            Kind::Binding => {
                ("expr.bind.3", format!("`{root}` is immutable"), format!("declare it with `mut {root} := ...`"))
            }
            Kind::Param => (
                "own.param.1",
                format!("the parameter `{root}` is read-only"),
                format!("declare it `mut {root} Type`, and pass it with `mut`"),
            ),
            Kind::SelfParam => (
                "trait.method.1",
                "`self` is read-only in this method".into(),
                "declare the method with `mut self`".into(),
            ),
            Kind::LoopVar => (
                "expr.loop.4",
                format!("the loop variable `{root}` is immutable"),
                "copy it into a `mut` binding first".into(),
            ),
        };
        self.error_help(rule.unwrap_or(default_rule), e.pos(), format!("{msg}, so it can't be {action}"), help);
    }

    /// Whether two argument places certainly overlap (own.excl.2). Overlaps
    /// that depend on index values are checked when the call runs (own.excl.3).
    fn certainly_overlap(&self, a: &Expr, b: &Expr) -> bool {
        let (Some((ra, pa)), Some((rb, pb))) = (place_path(a), place_path(b)) else { return false };
        if ra != rb || self.local(ra).is_none() {
            return false;
        }
        for (x, y) in pa.iter().zip(&pb) {
            match (x, y) {
                (PathStep::Field(f), PathStep::Field(g)) if f == g => {}
                (PathStep::Index(Expr::Int(m, _)), PathStep::Index(Expr::Int(n, _))) if m == n => {}
                (PathStep::Index(Expr::Name(m, _)), PathStep::Index(Expr::Name(n, _))) if m == n => {}
                _ => return false,
            }
        }
        true
    }

    /// Whether two places may overlap: same root, and no step that proves
    /// them apart (a different field, or a different constant index).
    fn may_overlap(&self, a: &Expr, b: &Expr) -> bool {
        let (Some((ra, pa)), Some((rb, pb))) = (place_path(a), place_path(b)) else { return false };
        if ra != rb || self.local(ra).is_none() {
            return false;
        }
        !pa.iter().zip(&pb).any(|step| match step {
            (PathStep::Field(f), PathStep::Field(g)) => f != g,
            (PathStep::Index(Expr::Int(m, _)), PathStep::Index(Expr::Int(n, _))) => m != n,
            _ => false,
        })
    }

    /// The first use in `body` of a place that may overlap `target` (own.excl.5).
    fn overlapping_use(&self, target: &Expr, body: &[Stmt]) -> Option<Pos> {
        body.iter().find_map(|s| self.overlap_in_stmt(target, s))
    }

    fn overlap_in_stmt(&self, target: &Expr, s: &Stmt) -> Option<Pos> {
        let e = |x: &Expr| self.overlap_in_expr(target, x);
        match s {
            Stmt::Let { value, otherwise, .. } => e(value).or_else(|| match otherwise {
                Some(Else::Value(d)) => e(d),
                Some(Else::Leave(st)) => self.overlap_in_stmt(target, st),
                Some(Else::Block { body, .. }) => self.overlapping_use(target, body),
                None => None,
            }),
            Stmt::Assign { target: t, value, .. } => e(t).or_else(|| e(value)),
            Stmt::While { cond, body, .. } => e(cond).or_else(|| self.overlapping_use(target, body)),
            Stmt::For { iter, body, .. } => e(iter).or_else(|| self.overlapping_use(target, body)),
            Stmt::Return(Some(v), _) | Stmt::Expr(v) => e(v),
            Stmt::Return(None, _) | Stmt::Break(_) | Stmt::Continue(_) => None,
        }
    }

    fn overlap_in_expr(&self, target: &Expr, x: &Expr) -> Option<Pos> {
        let e = |y: &Expr| self.overlap_in_expr(target, y);
        if let Some((_, path)) = place_path(x) {
            if self.may_overlap(target, x) {
                return Some(x.pos());
            }
            return path.iter().find_map(|step| match step {
                PathStep::Index(i) => e(i),
                PathStep::Field(_) => None,
            });
        }
        match x {
            Expr::Str(..) | Expr::Int(..) | Expr::Bool(..) | Expr::None(_) | Expr::Name(..) => None,
            Expr::Interp(parts, _) => parts.iter().find_map(|p| match p {
                InterpPart::Expr(y) => e(y),
                InterpPart::Lit(_) => None,
            }),
            Expr::List(items, _) => items.iter().find_map(e),
            Expr::Call { callee, args, .. } => e(callee).or_else(|| args.iter().find_map(|a| e(&a.value))),
            Expr::Method { recv, args, .. } => e(recv).or_else(|| args.iter().find_map(|a| e(&a.value))),
            Expr::Field { recv, .. } => e(recv),
            Expr::Index { recv, index, .. } => e(recv).or_else(|| e(index)),
            Expr::Unary { operand, .. } => e(operand),
            Expr::Binary { lhs, rhs, .. } => e(lhs).or_else(|| e(rhs)),
            Expr::Range { lo, hi, .. } => e(lo).or_else(|| e(hi)),
            Expr::If { branches, els, .. } => branches
                .iter()
                .find_map(|(c, b)| {
                    let cond = match c {
                        Cond::Expr(x) | Cond::Bind { value: x, .. } => x,
                    };
                    e(cond).or_else(|| self.overlapping_use(target, b))
                })
                .or_else(|| els.as_ref().and_then(|b| self.overlapping_use(target, b))),
            Expr::Match { scrutinee, arms, .. } => {
                e(scrutinee).or_else(|| arms.iter().find_map(|a| self.overlapping_use(target, &a.body)))
            }
            Expr::Try { expr, context, .. } => e(expr).or_else(|| context.as_deref().and_then(e)),
        }
    }

    // ---- Expressions ----

    /// The type of `e`. `expect` is the type the context wants, if known; it
    /// resolves bare variants and empty lists. `need` says whether the value
    /// is used (expr.if.4).
    fn expr(&mut self, e: &'p Expr, expect: Option<&Ty>, need: bool) -> Ty {
        match e {
            Expr::Str(..) => Ty::Str,
            Expr::Int(..) => Ty::Int,
            Expr::Bool(..) => Ty::Bool,
            Expr::None(pos) => self.none(*pos, expect),
            Expr::Try { expr, context, pos } => self.try_expr(expr, context.as_deref(), *pos),
            Expr::Interp(parts, _) => {
                for part in parts {
                    if let InterpPart::Expr(x) = part {
                        let t = self.expr(x, None, true);
                        self.text_form(&t, x.pos(), "lex.str.2");
                    }
                }
                Ty::Str
            }
            Expr::Name(n, pos) => self.name(n, *pos, expect),
            Expr::List(elems, pos) => self.list(elems, *pos, expect),
            Expr::Call { callee, args, .. } => self.call(callee, args, expect),
            Expr::Method { recv, name, name_pos, args } => self.method(recv, name, *name_pos, args),
            Expr::Field { recv, name, name_pos } => self.field(recv, name, *name_pos),
            Expr::Index { recv, index, .. } => self.index(recv, index),
            Expr::Unary { op, operand, pos } => self.unary(*op, operand, *pos),
            Expr::Binary { op, lhs, rhs, pos } => self.binary(*op, lhs, rhs, *pos),
            Expr::Range { lo, hi, pos, .. } => {
                self.range_bounds(lo, hi);
                self.unsupported(
                    *pos,
                    "a range outside `for` and list slicing is not supported by the bootstrap yet (spec expr.loop.3)",
                );
                Ty::Err
            }
            Expr::If { branches, els, else_err, pos } => {
                self.if_expr(branches, els.as_ref(), else_err.as_ref(), *pos, expect, need)
            }
            Expr::Match { scrutinee, arms, pos } => self.match_expr(scrutinee, arms, *pos, expect, need),
        }
    }

    /// Text forms exist for `int`, `bool` and `str` (type.str.3) and for error
    /// values (err.text.1).
    fn text_form(&mut self, t: &Ty, pos: Pos, no_value_rule: &'static str) {
        match t {
            Ty::Int | Ty::Str | Ty::Bool | Ty::Err | Ty::Never | Ty::Union(_) => {}
            t if self.is_error(t) => {}
            Ty::Unit => self.error(no_value_rule, pos, "this expression produces no value"),
            Ty::Res(..) => {
                self.reject_unhandled(t, pos);
            }
            Ty::Opt(_) => self.diags.push(
                Diagnostic::new(UNSUPPORTED, pos, "an optional has no text form yet (ADR 0088)")
                    .help("unwrap it first: `x := value else fallback`"),
            ),
            _ => {
                let name = self.ty_name(t);
                self.diags.push(
                    Diagnostic::new(
                        UNSUPPORTED,
                        pos,
                        format!("`{name}` has no text form yet; it needs the formatting trait (ADR 0088)"),
                    )
                    .help("turn it into a `str` first, e.g. with a method that returns one"),
                );
            }
        }
    }

    fn name(&mut self, n: &str, pos: Pos, expect: Option<&Ty>) -> Ty {
        if let Some(l) = self.local(n) {
            return l.ty.clone();
        }
        if n == "self" {
            self.error_help(
                "trait.method.1",
                pos,
                "`self` is only available in methods declared with `self`",
                "declare a method as `fn Type.name(self, ...)`",
            );
            return Ty::Err;
        }
        // type.enum.3: a bare variant where the expected enum type is known.
        if let Some((e, v)) = self.bare_variant(n, pos, expect) {
            if !self.prog.enums[e].variants[v].fields.is_empty() {
                let fields: Vec<String> =
                    self.prog.enums[e].variants[v].fields.iter().map(|f| format!("{}: ...", f.name)).collect();
                self.error_help(
                    "type.enum.6",
                    pos,
                    format!("the variant `{n}` carries data"),
                    format!("build it with `{n}({})`", fields.join(", ")),
                );
                return Ty::Err;
            }
            return Ty::Enum(e);
        }
        if !self.variant_owners(n).is_empty() {
            return Ty::Err;
        }
        if self.fns.contains_key(n) || PRELUDE.contains(&n) {
            self.unsupported(pos, format!("using the function `{n}` as a value is not supported by the bootstrap yet"));
            return Ty::Err;
        }
        if let Some(t) = self.types.get(n).cloned() {
            let help = match t {
                Ty::Struct(_) => format!("build a value with `{n}(field: value, ...)`"),
                _ => format!(
                    "use one of its variants, e.g. `{n}.{}`",
                    self.prog.enums[self.enum_of(&t)].variants[0].name
                ),
            };
            self.error_help("expr.name.1", pos, format!("`{n}` is a type, not a value"), help);
            return Ty::Err;
        }
        if BUILTIN_TYPES.contains(&n) || SPEC_TYPES_NOT_YET.contains(&n) {
            self.error("expr.name.1", pos, format!("`{n}` is a type, not a value"));
            return Ty::Err;
        }
        self.unknown_name(n, pos);
        Ty::Err
    }

    /// Enums whose variants may be written bare here (type.enum.3): the
    /// expected enum, the enums of an expected `T?` or `T ! E`, and, for a
    /// result whose errors are inferred, every error enum.
    fn expected_enums(&self, expect: Option<&Ty>) -> Vec<usize> {
        match expect {
            Some(Ty::Enum(e)) => vec![*e],
            Some(Ty::Opt(t)) => self.expected_enums(Some(t)),
            Some(Ty::Union(m)) => m.iter().filter_map(|t| if let Ty::Enum(e) = t { Some(*e) } else { None }).collect(),
            Some(Ty::Res(ok, errs)) => {
                let mut v = self.expected_enums(Some(ok));
                let errors: Vec<Ty> = match errs {
                    Errs::Set(set) => set.clone(),
                    Errs::Inferred(_) => {
                        (0..self.prog.enums.len()).filter(|e| self.prog.enums[*e].is_error).map(Ty::Enum).collect()
                    }
                };
                v.extend(errors.iter().filter_map(|t| if let Ty::Enum(e) = t { Some(*e) } else { None }));
                v
            }
            _ => Vec::new(),
        }
    }

    /// Resolve `n` as a bare variant (type.enum.3), recording it for the
    /// interpreter. Reports when it needs qualifying; `None` if it isn't one.
    fn bare_variant(&mut self, n: &str, pos: Pos, expect: Option<&Ty>) -> Option<(usize, usize)> {
        let found: Vec<(usize, usize)> =
            self.expected_enums(expect).into_iter().filter_map(|e| self.variant_index(e, n).map(|v| (e, v))).collect();
        if let [(e, v)] = found[..] {
            self.resolved.variants.insert(pos, (e, v));
            return Some((e, v));
        }
        let owners = if found.len() > 1 {
            found.iter().map(|(e, _)| format!("{}.{n}", self.prog.enums[*e].name)).collect()
        } else {
            self.variant_owners(n)
        };
        if !owners.is_empty() {
            let owners: Vec<&str> = owners.iter().map(String::as_str).collect();
            let msg = if found.len() > 1 {
                format!("`{n}` is a variant of more than one possible enum here")
            } else {
                format!("the type isn't known here, so the variant `{n}` needs its enum name")
            };
            self.error_help("type.enum.3", pos, msg, format!("write {}", list_names(&owners).replace(" and ", " or ")));
        }
        None
    }

    fn variant_owners(&self, n: &str) -> Vec<String> {
        self.prog
            .enums
            .iter()
            .filter(|e| e.variants.iter().any(|v| v.name == n))
            .map(|e| format!("{}.{n}", e.name))
            .collect()
    }

    /// `none` needs a known optional type (type.opt.4).
    fn none(&mut self, pos: Pos, expect: Option<&Ty>) -> Ty {
        match expect {
            Some(t @ Ty::Opt(_)) => t.clone(),
            Some(Ty::Res(ok, _)) if matches!(**ok, Ty::Opt(_)) => (**ok).clone(),
            _ => {
                self.error_help(
                    "type.opt.4",
                    pos,
                    "the type of `none` isn't known here",
                    "use `none` where a `T?` is expected, e.g. as an argument or a return value",
                );
                Ty::Err
            }
        }
    }

    /// `e?` and `e? "context"` (err.prop.1-3, err.ctx.2).
    fn try_expr(&mut self, inner: &'p Expr, context: Option<&'p Expr>, pos: Pos) -> Ty {
        let t = self.expr(inner, None, true);
        if let Some(c) = context {
            let ct = self.expr(c, Some(&Ty::Str), true);
            if !matches!(t, Ty::Res(..) | Ty::Err) {
                self.error("err.ctx.2", c.pos(), "only an error can carry context; this value can't fail");
            } else if !self.accepts(&Ty::Str, &ct) {
                self.mismatch("err.ctx.2", c.pos(), &Ty::Str, &ct);
            }
        }
        let ret = self.ret.clone();
        match t {
            Ty::Opt(v) => {
                if !matches!(ret, Ty::Opt(_)) {
                    self.error_help(
                        "err.prop.2",
                        pos,
                        "`?` on an optional only works in a function that returns an optional",
                        "supply a fallback or leave: `x := value else return ...`",
                    );
                }
                *v
            }
            Ty::Res(ok, errs) => {
                let errors = self.errs_of(&errs);
                match &ret {
                    Ty::Res(_, want) => {
                        let missing = self.add_errors(want, &errors);
                        if !missing.is_empty() {
                            let names: Vec<String> = missing.iter().map(|t| self.ty_name(t)).collect();
                            let names: Vec<&str> = names.iter().map(String::as_str).collect();
                            self.error_help(
                                "err.prop.3",
                                pos,
                                format!("`?` passes on {}, which this function's error type doesn't include", list_names(&names)),
                                "add them to the return type (`T ! A | B`), use `!T` to infer them, or handle them with `else`",
                            );
                        }
                    }
                    _ => self.error_help(
                        "err.prop.1",
                        pos,
                        "`?` passes the error on, but this function can't fail",
                        "declare a return type with `!` (e.g. `!int`, or `fn main() !`), or handle it with `else`",
                    ),
                }
                *ok
            }
            Ty::Err => Ty::Err,
            other => {
                let msg = format!("`?` needs a value that can be absent or fail, but this is {}", self.show(&other));
                self.error("err.prop.1", pos, msg);
                other
            }
        }
    }

    fn enum_of(&self, t: &Ty) -> usize {
        match t {
            Ty::Enum(e) => *e,
            _ => unreachable!("an enum type"),
        }
    }

    fn list(&mut self, elems: &'p [Expr], pos: Pos, expect: Option<&Ty>) -> Ty {
        let hint = match expect {
            Some(Ty::List(t)) => Some((**t).clone()),
            _ => None,
        };
        if elems.is_empty() {
            return match hint {
                Some(t) => Ty::List(Box::new(t)),
                None => {
                    self.error_help(
                        "type.coll.3",
                        pos,
                        "the element type of this empty list isn't known",
                        "use `[]` where a `List[...]` is expected, e.g. as an argument or struct field",
                    );
                    Ty::Err
                }
            };
        }
        let mut first: Option<Ty> = None;
        for e in elems {
            let want = first.clone().or_else(|| hint.clone());
            let t = self.expr(e, want.as_ref(), true);
            if self.reject_unhandled(&t, e.pos()) {
                continue;
            }
            match (&t, &first) {
                (Ty::Unit, _) => self.error("type.coll.2", e.pos(), "a list element must be a value"),
                (Ty::Never, _) => {}
                (_, None) => first = Some(t),
                (_, Some(f)) if !same(&t, f) => {
                    let msg = format!(
                        "list elements must all have one type: this is {}, the first is {}",
                        self.show(&t),
                        self.show(f)
                    );
                    self.error("type.coll.2", e.pos(), msg);
                }
                _ => {}
            }
        }
        Ty::List(Box::new(first.or(hint).unwrap_or(Ty::Err)))
    }

    /// Check arguments without a signature to check them against.
    fn args_only(&mut self, args: &'p [Arg]) {
        for a in args {
            self.expr(&a.value, None, true);
        }
    }

    fn call(&mut self, callee: &'p Expr, args: &'p [Arg], expect: Option<&Ty>) -> Ty {
        let Expr::Name(n, npos) = callee else {
            self.expr(callee, None, true);
            self.args_only(args);
            self.unsupported(callee.pos(), "calling the result of an expression is not supported by the bootstrap yet");
            return Ty::Err;
        };
        let npos = *npos;
        if self.local(n).is_some() {
            self.args_only(args);
            self.error("expr.call.1", npos, format!("`{n}` is a value, not a function"));
            return Ty::Err;
        }
        if n == "print" {
            return self.print_call(args, npos);
        }
        match self.types.get(n.as_str()).cloned() {
            Some(Ty::Struct(s)) => return self.struct_literal(s, args, npos),
            Some(t) => {
                self.args_only(args);
                let e = self.enum_of(&t);
                let example = &self.prog.enums[e].variants[0].name;
                self.error_help(
                    "expr.name.1",
                    npos,
                    format!("`{n}` is an enum, not a function"),
                    format!("use one of its variants, e.g. `{n}.{example}`"),
                );
                return Ty::Err;
            }
            None => {}
        }
        if let Some(&i) = self.fns.get(n.as_str()) {
            let sig = self.sigs[i].clone();
            self.check_args(n, &sig, args, None, npos);
            return sig.ret;
        }
        if let Some((e, v)) = self.bare_variant(n, npos, expect) {
            return self.variant_literal(e, v, args, npos);
        }
        self.args_only(args);
        if self.variant_owners(n).is_empty() {
            self.unknown_name(n, npos);
        }
        Ty::Err
    }

    /// `Variant(field: value, ...)` (type.enum.6).
    fn variant_literal(&mut self, e: usize, v: usize, args: &'p [Arg], pos: Pos) -> Ty {
        let prog = self.prog;
        let variant = &prog.enums[e].variants[v];
        if variant.fields.is_empty() {
            self.args_only(args);
            self.error_help(
                "type.enum.6",
                pos,
                format!("the variant `{}` carries no data", variant.name),
                format!("write `{}` without parentheses", variant.name),
            );
        } else {
            let types = self.variant_types[e][v].clone();
            let label = format!("{}.{}", prog.enums[e].name, variant.name);
            self.fields_literal(&label, &variant.fields, &types, args, pos);
        }
        Ty::Enum(e)
    }

    /// Check a call's arguments against a signature (expr.call.1-2,
    /// own.param.2, own.excl.1-2). `recv` is a method's receiver.
    fn check_args(&mut self, what: &str, sig: &Sig, args: &'p [Arg], recv: Option<&'p Expr>, pos: Pos) {
        for a in args {
            if let Some((_, lpos)) = &a.label {
                self.unsupported(*lpos, "named arguments are not supported by the bootstrap yet (ADR 0058)");
            }
        }
        if args.len() != sig.params.len() {
            self.args_only(args);
            self.error(
                "expr.call.1",
                pos,
                format!("`{what}` takes {}, but {} given", count(sig.params.len(), "argument"), was_were(args.len())),
            );
            return;
        }
        for (a, (pmut, pty)) in args.iter().zip(&sig.params) {
            let t = self.expr(&a.value, Some(pty), true);
            if *pmut && !a.mutable {
                self.error_help(
                    "own.param.2",
                    a.value.pos(),
                    "this parameter is `mut`, so the argument must be marked `mut`",
                    "write `mut` before the argument",
                );
            } else if *pmut {
                self.require_mutable(&a.value, Some("own.param.2"), "passed `mut`");
            } else if a.mutable {
                self.error_help("own.param.2", a.value.pos(), "this parameter isn't `mut`", "remove the `mut`");
            }
            if !self.accepts(pty, &t) {
                self.mismatch("expr.call.2", a.value.pos(), pty, &t);
            }
        }
        // own.excl.1-2: two `mut` places must not overlap. A read-only argument
        // that overlaps a `mut` one is fine: it is passed as a copy (own.excl.4).
        let mut items: Vec<(&'p Expr, bool)> = Vec::new();
        if let Some(r) = recv {
            items.push((r, sig.receiver == Some(true)));
        }
        items.extend(args.iter().zip(&sig.params).map(|(a, (pmut, _))| (&a.value, *pmut && a.mutable)));
        for j in 0..items.len() {
            for i in 0..j {
                if items[i].1 && items[j].1 && self.certainly_overlap(items[i].0, items[j].0) {
                    let help = if matches!(items[j].0, Expr::Index { .. }) {
                        "to exchange two list elements, use `xs.swap(i, j)`"
                    } else {
                        "pass `mut` only once; the function already sees the change through it"
                    };
                    self.error_help(
                        "own.excl.2",
                        items[j].0.pos(),
                        "this `mut` argument overlaps another `mut` argument of the same call",
                        help,
                    );
                }
            }
        }
    }

    fn print_call(&mut self, args: &'p [Arg], pos: Pos) -> Ty {
        for a in args {
            if let Some((_, lpos)) = &a.label {
                self.unsupported(*lpos, "named arguments are not supported by the bootstrap yet (ADR 0058)");
            }
            if a.mutable {
                self.error_help("own.param.2", a.value.pos(), "`print`'s parameter isn't `mut`", "remove the `mut`");
            }
        }
        if args.len() != 1 {
            self.args_only(args);
            self.error(
                "mod.main.3",
                pos,
                format!("`print` takes exactly one argument, but {} given", was_were(args.len())),
            );
        } else {
            let t = self.expr(&args[0].value, None, true);
            if t == Ty::Unit {
                self.error("mod.main.3", args[0].value.pos(), "this expression produces no value to print");
            } else {
                self.text_form(&t, args[0].value.pos(), "mod.main.3");
            }
        }
        Ty::Unit
    }

    /// `Type(field: value, ...)` (type.struct.3-5).
    fn struct_literal(&mut self, s: usize, args: &'p [Arg], pos: Pos) -> Ty {
        let prog = self.prog;
        let decl = &prog.structs[s];
        let types = self.field_types[s].clone();
        self.fields_literal(&decl.name, &decl.fields, &types, args, pos);
        Ty::Struct(s)
    }

    /// The `(field: value, ...)` of a struct literal or a variant with data
    /// (type.struct.3-5, type.enum.6).
    fn fields_literal(&mut self, type_label: &str, fields: &'p [Field], types: &[Ty], args: &'p [Arg], pos: Pos) {
        let mut given = vec![false; fields.len()];
        for (k, a) in args.iter().enumerate() {
            if a.mutable {
                self.error_help(
                    "own.param.2",
                    a.value.pos(),
                    "`mut` can't mark a struct field's value",
                    "remove the `mut`",
                );
            }
            let Some((label, lpos)) = &a.label else {
                self.expr(&a.value, None, true);
                let help = match fields.get(k) {
                    Some(f) => format!("write `{}: ...`", f.name),
                    None => "write `field: value`".into(),
                };
                self.error_help("type.struct.3", a.value.pos(), "fields are given by name", help);
                continue;
            };
            let Some(i) = fields.iter().position(|f| f.name == *label) else {
                self.expr(&a.value, None, true);
                let names: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();
                let mut d = Diagnostic::new("type.struct.4", *lpos, format!("`{type_label}` has no field `{label}`"));
                if let Some(s) = suggest(label, &names) {
                    d = d.help(format!("did you mean `{s}`?"));
                }
                self.diags.push(d);
                continue;
            };
            if given[i] {
                self.error("type.struct.4", *lpos, format!("field `{label}` is given more than once"));
            }
            given[i] = true;
            let ft = types[i].clone();
            let t = self.expr(&a.value, Some(&ft), true);
            if self.reject_unhandled(&t, a.value.pos()) {
                continue;
            }
            if !self.accepts(&ft, &t) {
                let msg = format!("field `{label}` is {}, but this is {}", self.show(&ft), self.show(&t));
                self.error("type.struct.5", a.value.pos(), msg);
            }
        }
        let missing: Vec<&str> =
            fields.iter().zip(&given).filter(|(_, g)| !**g).map(|(f, _)| f.name.as_str()).collect();
        // Unnamed arguments were already reported; don't also call their fields missing.
        if !missing.is_empty() && args.iter().all(|a| a.label.is_some()) {
            let msg = format!(
                "`{type_label}(...)` is missing {} {}",
                if missing.len() == 1 { "field" } else { "fields" },
                list_names(&missing)
            );
            self.error_help("type.struct.4", pos, msg, "every field must be given; there are no defaults yet");
        }
    }

    fn method(&mut self, recv: &'p Expr, name: &'p str, pos: Pos, args: &'p [Arg]) -> Ty {
        // `Type.name(args)`: an associated function (trait.method.4).
        if let Expr::Name(tn, _) = recv
            && self.local(tn).is_none()
            && let Some(t) = self.types.get(tn.as_str()).cloned()
        {
            return self.associated(tn, &t, name, pos, args);
        }
        let rt = self.expr(recv, None, true);
        if self.reject_unhandled(&rt, recv.pos()) {
            self.args_only(args);
            return Ty::Err;
        }
        if let Ty::List(elem) = &rt {
            return self.list_method(recv, elem, name, pos, args);
        }
        if rt == Ty::Str {
            return self.str_method(recv, name, pos, args);
        }
        let Some(tn) = self.type_name(&rt) else {
            self.args_only(args);
            if rt != Ty::Err {
                let msg = format!("{} has no method `{name}`", self.show(&rt));
                self.error("expr.name.1", pos, msg);
            }
            return Ty::Err;
        };
        let Some(i) = self.method_index(tn, name) else {
            self.args_only(args);
            let is_field = matches!(rt, Ty::Struct(s) if self.prog.structs[s].fields.iter().any(|f| f.name == name));
            if is_field {
                self.error_help(
                    "expr.name.1",
                    pos,
                    format!("`{name}` is a field of `{tn}`, not a method"),
                    "remove the `()`",
                );
            } else {
                let names: Vec<&str> = self.methods.get(tn).map(|m| m.keys().copied().collect()).unwrap_or_default();
                let mut d = Diagnostic::new("expr.name.1", pos, format!("`{tn}` has no method `{name}`"));
                if let Some(s) = suggest(name, &names) {
                    d = d.help(format!("did you mean `{s}`?"));
                }
                self.diags.push(d);
            }
            return Ty::Err;
        };
        let sig = self.sigs[i].clone();
        match sig.receiver {
            None => {
                self.args_only(args);
                self.error_help(
                    "trait.method.4",
                    pos,
                    format!("`{name}` is an associated function, not a method"),
                    format!("call it as `{tn}.{name}(...)`"),
                );
            }
            Some(mutable) => {
                if mutable {
                    self.require_mutable(recv, Some("trait.method.3"), "the receiver of a `mut self` method");
                }
                self.check_args(name, &sig, args, Some(recv), pos);
            }
        }
        sig.ret
    }

    fn associated(&mut self, tn: &'p str, t: &Ty, name: &'p str, pos: Pos, args: &'p [Arg]) -> Ty {
        if let Ty::Enum(e) = t
            && let Some(v) = self.variant_index(*e, name)
        {
            return self.variant_literal(*e, v, args, pos);
        }
        let Some(i) = self.method_index(tn, name) else {
            self.args_only(args);
            self.error("expr.name.1", pos, format!("`{tn}` has no associated function `{name}`"));
            return Ty::Err;
        };
        let sig = self.sigs[i].clone();
        if sig.receiver.is_some() {
            self.args_only(args);
            self.error_help(
                "trait.method.4",
                pos,
                format!("`{name}` is a method, so it's called on a value"),
                format!("call it as `x.{name}(...)` where `x` is a `{tn}`"),
            );
        } else {
            self.check_args(&format!("{tn}.{name}"), &sig, args, None, pos);
        }
        sig.ret
    }

    /// `xs.len()`, `xs.push(v)` and `xs.swap(i, j)` (type.coll.5-7).
    fn list_method(&mut self, recv: &'p Expr, elem: &Ty, name: &str, pos: Pos, args: &'p [Arg]) -> Ty {
        match name {
            "len" => {
                let sig = Sig { receiver: Some(false), params: Vec::new(), ret: Ty::Int };
                self.check_args("len", &sig, args, Some(recv), pos);
                Ty::Int
            }
            "push" => {
                self.require_mutable(recv, Some("trait.method.3"), "the receiver of `push`");
                let sig = Sig { receiver: Some(true), params: vec![(false, elem.clone())], ret: Ty::Unit };
                self.check_args("push", &sig, args, Some(recv), pos);
                Ty::Unit
            }
            "swap" => {
                self.require_mutable(recv, Some("trait.method.3"), "the receiver of `swap`");
                let sig = Sig { receiver: Some(true), params: vec![(false, Ty::Int), (false, Ty::Int)], ret: Ty::Unit };
                self.check_args("swap", &sig, args, Some(recv), pos);
                Ty::Unit
            }
            _ => {
                self.args_only(args);
                let mut d = Diagnostic::new("expr.name.1", pos, format!("`List` has no method `{name}`"));
                if let Some(s) = suggest(name, LIST_METHODS) {
                    d = d.help(format!("did you mean `{s}`?"));
                }
                self.diags.push(d);
                Ty::Err
            }
        }
    }

    /// `s.split(sep)` and `s.to_int()` (type.str.4-5).
    fn str_method(&mut self, recv: &'p Expr, name: &str, pos: Pos, args: &'p [Arg]) -> Ty {
        let (params, ret) = match name {
            "split" => (vec![(false, Ty::Str)], Ty::List(Box::new(Ty::Str))),
            "to_int" => (Vec::new(), Ty::Opt(Box::new(Ty::Int))),
            _ => {
                self.args_only(args);
                let mut d = Diagnostic::new("expr.name.1", pos, format!("`str` has no method `{name}`"));
                if let Some(s) = suggest(name, &["split", "to_int"]) {
                    d = d.help(format!("did you mean `{s}`?"));
                }
                self.diags.push(d);
                return Ty::Err;
            }
        };
        let sig = Sig { receiver: Some(false), params, ret: ret.clone() };
        self.check_args(name, &sig, args, Some(recv), pos);
        ret
    }

    fn field(&mut self, recv: &'p Expr, name: &str, pos: Pos) -> Ty {
        // `Enum.Variant` (type.enum.3).
        if let Expr::Name(tn, _) = recv
            && self.local(tn).is_none()
            && let Some(t) = self.types.get(tn.as_str()).cloned()
        {
            if let Ty::Enum(e) = t {
                if let Some(v) = self.variant_index(e, name) {
                    let fields = &self.prog.enums[e].variants[v].fields;
                    if !fields.is_empty() {
                        let shown: Vec<String> = fields.iter().map(|f| format!("{}: ...", f.name)).collect();
                        self.error_help(
                            "type.enum.6",
                            pos,
                            format!("the variant `{name}` carries data"),
                            format!("build it with `{tn}.{name}({})`", shown.join(", ")),
                        );
                        return Ty::Err;
                    }
                    return t;
                }
                let names: Vec<&str> = self.prog.enums[e].variants.iter().map(|v| v.name.as_str()).collect();
                let mut d = Diagnostic::new("expr.name.1", pos, format!("`{tn}` has no variant `{name}`"));
                if let Some(s) = suggest(name, &names) {
                    d = d.help(format!("did you mean `{s}`?"));
                }
                self.diags.push(d);
            } else {
                self.error("type.struct.6", pos, format!("`{tn}` is a type; fields are read from values"));
            }
            return Ty::Err;
        }
        let rt = self.expr(recv, None, true);
        if self.reject_unhandled(&rt, recv.pos()) {
            return Ty::Err;
        }
        let is_method = match &rt {
            Ty::List(_) => LIST_METHODS.contains(&name),
            t => self.type_name(t).is_some_and(|tn| self.method_index(tn, name).is_some()),
        };
        if is_method {
            self.error_help(
                "expr.op.3",
                pos,
                format!("`{name}` is a method, so calling it needs `()`"),
                format!("write `.{name}()`"),
            );
            return Ty::Err;
        }
        match &rt {
            Ty::Struct(s) => {
                let decl = &self.prog.structs[*s];
                if let Some(i) = decl.fields.iter().position(|f| f.name == name) {
                    return self.field_types[*s][i].clone();
                }
                let names: Vec<&str> = decl.fields.iter().map(|f| f.name.as_str()).collect();
                let mut d = Diagnostic::new("type.struct.6", pos, format!("`{}` has no field `{name}`", decl.name));
                if let Some(s) = suggest(name, &names) {
                    d = d.help(format!("did you mean `{s}`?"));
                }
                self.diags.push(d);
                Ty::Err
            }
            Ty::Err => Ty::Err,
            t => {
                let msg = format!("{} has no fields", self.show(t));
                self.error("type.struct.6", pos, msg);
                Ty::Err
            }
        }
    }

    /// `xs[i]` and `xs[a..b]` (type.coll.4, type.str.2).
    fn index(&mut self, recv: &'p Expr, index: &'p Expr) -> Ty {
        let rt = self.expr(recv, None, true);
        if self.reject_unhandled(&rt, recv.pos()) {
            return Ty::Err;
        }
        let slice = matches!(index, Expr::Range { .. });
        if let Expr::Range { lo, hi, .. } = index {
            self.range_bounds(lo, hi);
        } else {
            let it = self.expr(index, Some(&Ty::Int), true);
            if !self.accepts(&Ty::Int, &it) {
                let msg = format!("a list index must be `int`, found {}", self.show(&it));
                self.error("type.coll.4", index.pos(), msg);
            }
        }
        match rt {
            Ty::List(t) if slice => Ty::List(t),
            Ty::List(t) => *t,
            Ty::Err => Ty::Err,
            Ty::Str => {
                self.error("type.str.2", recv.pos(), "a `str` can't be indexed");
                Ty::Err
            }
            t => {
                let msg = format!("only lists can be indexed, not {}", self.show(&t));
                self.error("type.coll.4", recv.pos(), msg);
                Ty::Err
            }
        }
    }

    fn unary(&mut self, op: UnOp, operand: &'p Expr, pos: Pos) -> Ty {
        let (want, rule, what) = match op {
            UnOp::Neg => (Ty::Int, "trait.op.1", "unary `-`"),
            UnOp::Not => (Ty::Bool, "type.bool.1", "`not`"),
        };
        let t = self.expr(operand, Some(&want), true);
        if self.reject_unhandled(&t, operand.pos()) {
            return Ty::Err;
        }
        if self.accepts(&want, &t) {
            return want;
        }
        let msg = format!("{what} needs {}, found {}", self.show(&want), self.show(&t));
        self.error(rule, pos, msg);
        Ty::Err
    }

    fn binary(&mut self, op: BinOp, lhs: &'p Expr, rhs: &'p Expr, pos: Pos) -> Ty {
        if matches!(op, BinOp::And | BinOp::Or) {
            for side in [lhs, rhs] {
                let t = self.expr(side, Some(&Ty::Bool), true);
                if self.reject_unhandled(&t, side.pos()) {
                    continue;
                }
                if !self.accepts(&Ty::Bool, &t) {
                    let msg = format!("`{}` needs `bool` operands, found {}", op.symbol(), self.show(&t));
                    self.error("type.bool.1", side.pos(), msg);
                }
            }
            return Ty::Bool;
        }
        let lt = self.expr(lhs, None, true);
        let rt = self.expr(rhs, Some(&lt), true);
        self.operator(op, &lt, &rt, pos)
    }

    /// Which types support which operators (type.num.5, type.str.3, trait.op.1, trait.op.3).
    fn operator(&mut self, op: BinOp, lt: &Ty, rt: &Ty, pos: Pos) -> Ty {
        let sym = op.symbol();
        let result = if op.is_arithmetic() { Ty::Int } else { Ty::Bool };
        if matches!(lt, Ty::Err | Ty::Never) || matches!(rt, Ty::Err | Ty::Never) {
            return if op.is_arithmetic() { Ty::Err } else { Ty::Bool };
        }
        if self.reject_unhandled(lt, pos) || self.reject_unhandled(rt, pos) {
            return Ty::Err;
        }
        if *lt == Ty::Unit || *rt == Ty::Unit {
            self.error("trait.op.1", pos, format!("an operand of `{sym}` produces no value"));
            return Ty::Err;
        }
        if lt != rt {
            let msg = format!("`{sym}` needs operands of one type, found {} and {}", self.show(lt), self.show(rt));
            if op == BinOp::Add && (*lt == Ty::Str || *rt == Ty::Str) {
                self.error_help("trait.op.3", pos, msg, "build text with interpolation: \"{a}{b}\"");
            } else {
                self.error("trait.op.3", pos, msg);
            }
            return if op.is_arithmetic() { Ty::Err } else { Ty::Bool };
        }
        let ordering = matches!(op, BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge);
        match lt {
            Ty::Int => return result,
            Ty::Str | Ty::Bool if matches!(op, BinOp::Eq | BinOp::Ne) => return Ty::Bool,
            Ty::Str if op == BinOp::Add => self.diags.push(
                Diagnostic::new(UNSUPPORTED, pos, "joining strings with `+` is not decided yet (spec trait.op.1)")
                    .help("build text with interpolation: \"{a}{b}\""),
            ),
            Ty::Str if ordering => self.unsupported(pos, "ordering `str` values is not decided yet (spec trait.op.1)"),
            Ty::Struct(_) | Ty::Enum(_) | Ty::List(_) | Ty::Opt(_) | Ty::Union(_)
                if matches!(op, BinOp::Eq | BinOp::Ne) =>
            {
                let name = self.ty_name(lt);
                let mut d = Diagnostic::new(
                    UNSUPPORTED,
                    pos,
                    format!(
                        "comparing `{name}` values with `{sym}` is not supported by the bootstrap yet (spec trait.op.1)"
                    ),
                );
                if matches!(lt, Ty::Enum(_)) {
                    d = d.help("use `match`");
                } else if matches!(lt, Ty::Opt(_)) {
                    d = d.help("use `else` to handle `none`: `x := value else fallback`");
                }
                self.diags.push(d);
            }
            t => {
                let msg = format!("`{sym}` is not defined for {}", self.show(t));
                self.error("trait.op.1", pos, msg);
            }
        }
        if op.is_arithmetic() { Ty::Err } else { Ty::Bool }
    }

    // ---- `if` and `match` ----

    fn if_expr(
        &mut self,
        branches: &'p [(Cond, Block)],
        els: Option<&'p Block>,
        else_err: Option<&'p (String, Pos)>,
        pos: Pos,
        expect: Option<&Ty>,
        need: bool,
    ) -> Ty {
        if need && els.is_none() {
            self.error_help(
                "expr.if.4",
                pos,
                "this `if` is used as a value, so it needs an `else`",
                "add an `else` branch",
            );
        }
        let mut agree = Agree::new(expect, need, "branch", "`if`");
        let mut binds_failure = false;
        let mut errors = None;
        for (cond, body) in branches {
            let mark = self.locals.len();
            match cond {
                Cond::Expr(c) => self.condition(c),
                Cond::Bind { name, pos: npos, value } => {
                    let (ty, errs) = self.binding_condition(name, value);
                    binds_failure |= errs.is_some();
                    errors = errs;
                    if self.declare(name, *npos) {
                        self.locals.push(Local { name: name.clone(), ty, mutable: false, kind: Kind::Binding });
                    }
                }
            }
            agree.block(self, body, pos);
            self.locals.truncate(mark);
        }
        // err.use.2: a failure is never ignored silently.
        if binds_failure && els.is_none() {
            self.error_help(
                "err.use.2",
                pos,
                "this `if` ignores the failure; an `if` on a result that can fail needs an `else`",
                "add `else err` and handle the error",
            );
        }
        if let Some(body) = els {
            let mark = self.locals.len();
            if let Some((name, npos)) = else_err {
                match (&errors, branches.len()) {
                    (Some(set), 1) => {
                        let t = if set.len() == 1 { set[0].clone() } else { Ty::Union(set.clone()) };
                        if self.declare(name, *npos) {
                            self.locals.push(Local { name: name.clone(), ty: t, mutable: false, kind: Kind::Binding });
                        }
                    }
                    _ => self.error_help(
                        "expr.if.6",
                        *npos,
                        "`else err` needs an `if` with one condition that binds a result that can fail",
                        "remove the name, or write `if x := ...` on a `T ! E` value with no `elif`",
                    ),
                }
            }
            agree.block(self, body, pos);
            self.locals.truncate(mark);
        }
        if need && els.is_none() { Ty::Err } else { agree.result() }
    }

    /// The type bound by `if name := value`, and the errors it can fail with
    /// when `value` is a `T ! E` (expr.if.5).
    fn binding_condition(&mut self, name: &str, value: &'p Expr) -> (Ty, Option<Vec<Ty>>) {
        match self.expr(value, None, true) {
            Ty::Opt(t) => (*t, None),
            Ty::Res(t, e) => (*t, Some(self.errs_of(&e))),
            Ty::Err => (Ty::Err, None),
            other => {
                let msg = format!(
                    "`if {name} := ...` needs a value that can be absent (`T?`) or fail (`T ! E`), but this is {}",
                    self.show(&other)
                );
                self.error_help("expr.if.5", value.pos(), msg, "compare it instead, e.g. `if x == ...`");
                (other, None)
            }
        }
    }

    fn match_expr(
        &mut self,
        scrutinee: &'p Expr,
        arms: &'p [crate::ast::Arm],
        pos: Pos,
        expect: Option<&Ty>,
        need: bool,
    ) -> Ty {
        let st = self.expr(scrutinee, None, true);
        let e = match st {
            Ty::Enum(e) => Some(e),
            Ty::Err => None,
            t if self.reject_unhandled(&t, scrutinee.pos()) => None,
            Ty::Union(_) => {
                self.unsupported(
                    scrutinee.pos(),
                    "matching on a set of error types is not designed yet (OQ-029); match on one error type",
                );
                None
            }
            t => {
                let msg = format!(
                    "matching on {} needs literal patterns, which are not supported by the bootstrap yet (spec expr.match.4)",
                    self.show(&t)
                );
                self.unsupported(scrutinee.pos(), msg);
                None
            }
        };
        let mut covered = vec![false; e.map_or(0, |e| self.prog.enums[e].variants.len())];
        let mut wildcard = false;
        let mut agree = Agree::new(expect, need, "arm", "`match`");
        for arm in arms {
            let bindings = match (e, &arm.pat) {
                (Some(e), _) => self.pattern(e, &arm.pat, &mut covered, &mut wildcard),
                // The scrutinee was already reported: bind names quietly to
                // avoid follow-on errors.
                (None, Pattern::Variant { bindings: Some(names), .. }) => {
                    names.iter().map(|(n, p)| (n.clone(), *p, Ty::Err)).collect()
                }
                (None, _) => Vec::new(),
            };
            let mark = self.locals.len();
            for (name, npos, ty) in bindings {
                if name != "_" && self.declare(&name, npos) {
                    self.locals.push(Local { name, ty, mutable: false, kind: Kind::Binding });
                }
            }
            agree.block(self, &arm.body, pos);
            self.locals.truncate(mark);
        }
        if let Some(e) = e {
            let missing: Vec<&str> = (self.prog.enums[e].variants.iter())
                .zip(&covered)
                .filter(|(_, c)| !**c)
                .map(|(v, _)| v.name.as_str())
                .collect();
            if !wildcard && !missing.is_empty() {
                self.error_help(
                    "expr.match.2",
                    pos,
                    format!("this `match` doesn't cover {}", list_names(&missing)),
                    "add an arm for each, or a `_` arm",
                );
            }
        }
        agree.result()
    }

    /// Check one pattern against the scrutinee's enum (expr.match.3-6), and
    /// return the names it binds with their types.
    fn pattern(
        &mut self,
        e: usize,
        pat: &Pattern,
        covered: &mut [bool],
        wildcard: &mut bool,
    ) -> Vec<(String, Pos, Ty)> {
        let prog = self.prog;
        let decl = &prog.enums[e];
        match pat {
            Pattern::Wildcard(pos) => {
                if *wildcard || covered.iter().all(|c| *c) {
                    self.error("expr.match.5", *pos, "this arm can never match; earlier arms cover every case");
                    return Vec::new();
                }
                let hidden: Vec<&str> = decl
                    .variants
                    .iter()
                    .zip(covered.iter())
                    .filter(|(_, c)| !**c)
                    .map(|(v, _)| v.name.as_str())
                    .collect();
                self.diags.push(
                    Diagnostic::warning(
                        "expr.match.3",
                        *pos,
                        format!("`_` covers {}, so a variant added later won't be flagged here", list_names(&hidden)),
                    )
                    .help("list the variants instead of `_`"),
                );
                *wildcard = true;
                Vec::new()
            }
            Pattern::Variant { qualifier, name, bindings, pos } => {
                if let Some(q) = qualifier
                    && *q != decl.name
                {
                    self.error("expr.match.4", *pos, format!("`{q}.{name}` isn't a variant of `{}`", decl.name));
                    return Vec::new();
                }
                let Some(v) = self.variant_index(e, name) else {
                    let names: Vec<&str> = decl.variants.iter().map(|v| v.name.as_str()).collect();
                    let mut d =
                        Diagnostic::new("expr.name.1", *pos, format!("`{}` has no variant `{name}`", decl.name));
                    if let Some(s) = suggest(name, &names) {
                        d = d.help(format!("did you mean `{s}`?"));
                    }
                    self.diags.push(d);
                    return Vec::new();
                };
                if *wildcard || covered[v] {
                    self.error("expr.match.5", *pos, format!("this arm can never match; `{name}` is already covered"));
                } else {
                    covered[v] = true;
                }
                // expr.match.6: one binding (or `_`) per field, in order.
                let fields = &decl.variants[v].fields;
                let given = bindings.as_deref().unwrap_or(&[]);
                if fields.len() != given.len() {
                    let shown: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();
                    let help = if fields.is_empty() {
                        format!("write `{name}` without parentheses")
                    } else {
                        format!("write `{name}({})`, using `_` for fields you don't need", shown.join(", "))
                    };
                    self.error_help(
                        "expr.match.6",
                        *pos,
                        format!("`{name}` has {}, but the pattern gives {}", count(fields.len(), "field"), given.len()),
                        help,
                    );
                    return Vec::new();
                }
                let types = &self.variant_types[e][v];
                given.iter().zip(types).map(|((n, p), t)| (n.clone(), *p, t.clone())).collect()
            }
        }
    }

    fn unknown_name(&mut self, name: &str, pos: Pos) {
        let candidates: Vec<&str> = (self.locals.iter().map(|l| l.name.as_str()))
            .chain(self.fns.keys().copied())
            .chain(PRELUDE.iter().copied())
            .chain(self.types.keys().copied())
            .collect();
        let mut d = Diagnostic::new("expr.name.1", pos, format!("unknown name `{name}`"));
        if let Some(s) = suggest(name, &candidates) {
            d = d.help(format!("did you mean `{s}`?"));
        }
        self.diags.push(d);
    }
}

/// Checks that the branches of an `if` or the arms of a `match` agree on a
/// type when the value is used (expr.if.4).
struct Agree {
    expect: Option<Ty>,
    need: bool,
    first: Option<Ty>,
    failed: bool,
    part: &'static str,
    construct: &'static str,
}

impl Agree {
    fn new(expect: Option<&Ty>, need: bool, part: &'static str, construct: &'static str) -> Self {
        Agree { expect: expect.cloned(), need, first: None, failed: false, part, construct }
    }

    fn block<'p>(&mut self, c: &mut Checker<'p>, body: &'p Block, pos: Pos) {
        let hint = self.expect.clone().or_else(|| self.first.clone());
        let t = c.block(body, hint.as_ref(), self.need);
        if !self.need {
            return;
        }
        let at = body.last().map_or(pos, Stmt::pos);
        // With a known expected type, each branch must fit it.
        if let Some(want) = self.expect.clone()
            && t != Ty::Unit
        {
            if !c.accepts(&want, &t) {
                self.failed = true;
                if !c.reject_unhandled(&t, at) {
                    let msg = format!("this {} is {}, but {} is expected", self.part, c.show(&t), c.show(&want));
                    c.error("expr.if.4", at, msg);
                }
            } else if t != Ty::Never {
                self.first = Some(want);
            }
            return;
        }
        match (&t, &self.first) {
            (Ty::Never, _) => {}
            (Ty::Err, _) => self.failed = true,
            (Ty::Unit, _) => {
                self.failed = true;
                let (part, construct) = (self.part, self.construct);
                c.error(
                    "expr.if.4",
                    at,
                    format!("this {part} produces no value, but the {construct} is used as a value"),
                );
            }
            (_, None) => self.first = Some(t),
            (_, Some(f)) if !same(&t, f) => {
                self.failed = true;
                let msg = format!("this {} is {}, but the first {} is {}", self.part, c.show(&t), self.part, c.show(f));
                c.error("expr.if.4", at, msg);
            }
            _ => {}
        }
    }

    fn result(self) -> Ty {
        if !self.need {
            Ty::Unit
        } else if let Some(t) = self.first {
            t
        } else if self.failed {
            Ty::Err
        } else {
            Ty::Never
        }
    }
}

fn count(n: usize, noun: &str) -> String {
    if n == 1 { format!("1 {noun}") } else { format!("{n} {noun}s") }
}

fn was_were(n: usize) -> String {
    if n == 1 { "1 was".into() } else { format!("{n} were") }
}

/// Closest candidate by edit distance, if it's close enough to be a likely typo.
fn suggest<'a>(name: &str, candidates: &[&'a str]) -> Option<&'a str> {
    let limit = (name.chars().count() / 3).max(1);
    candidates.iter().map(|c| (edit_distance(name, c), *c)).filter(|(d, _)| *d <= limit).min().map(|(_, c)| c)
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        prev = cur;
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer::lex, parser::parse};

    fn diags(src: &str) -> Vec<Diagnostic> {
        let (tokens, ld) = lex(src);
        assert!(ld.is_empty(), "{ld:?}");
        let (prog, pd) = parse(tokens);
        assert!(pd.is_empty(), "{pd:?}");
        check(&prog).0
    }

    fn rules(src: &str) -> Vec<&'static str> {
        let mut r: Vec<_> = diags(src).into_iter().map(|d| d.rule).collect();
        r.sort();
        r
    }

    const GRADES: &str = "enum Grade\n    A\n    B\n    C\n";

    #[test]
    fn hello_world_is_clean() {
        assert!(rules("fn main()\n    print(\"Hello, world!\")\n").is_empty());
    }

    #[test]
    fn missing_main() {
        assert_eq!(rules("fn start()\n    print(1)\n"), vec!["mod.main.1"]);
    }

    #[test]
    fn unknown_name_suggests() {
        let d = diags("fn main()\n    prnt(\"x\")\n");
        assert_eq!(d[0].rule, "expr.name.1");
        assert_eq!(d[0].help.as_deref(), Some("did you mean `print`?"));
    }

    #[test]
    fn no_shadowing() {
        assert_eq!(rules("fn main()\n    x := 1\n    x := 2\n"), vec!["expr.bind.4"]);
        assert_eq!(rules("fn main()\n    main := 1\n"), vec!["expr.bind.5"]);
        assert_eq!(rules("fn main()\n    x := 1\n    for i in 0..3\n        x := i\n"), vec!["expr.bind.4"]);
        // Sibling blocks may reuse a name.
        assert!(rules("fn main()\n    if true\n        x := 1\n    else\n        x := 2\n").is_empty());
    }

    #[test]
    fn calls_check_arity_and_types() {
        let src = "fn greet(name str) str\n    name\nfn main()\n    print(greet(1))\n    print(greet())\n";
        assert_eq!(rules(src), vec!["expr.call.1", "expr.call.2"]);
    }

    #[test]
    fn return_type_must_match() {
        assert_eq!(rules("fn f() str\n    1\nfn main()\n    print(f())\n"), vec!["expr.block.2"]);
        assert!(rules("fn f() int\n    if true\n        return 1\n    2\nfn main()\n    print(f())\n").is_empty());
    }

    #[test]
    fn print_needs_one_value() {
        assert_eq!(rules("fn main()\n    print(print(1))\n"), vec!["mod.main.3"]);
        assert_eq!(rules("fn main()\n    print(1, 2)\n"), vec!["mod.main.3"]);
    }

    #[test]
    fn bare_variants_need_an_expected_type() {
        let ok = format!(
            "{GRADES}fn f() Grade\n    if true\n        A\n    else\n        B\nfn main()\n    g := Grade.C\n    h := f()\n"
        );
        assert!(rules(&ok).is_empty());
        let bad = format!("{GRADES}fn main()\n    g := A\n");
        assert_eq!(rules(&bad), vec!["type.enum.3"]);
    }

    #[test]
    fn match_exhaustiveness_and_reachability() {
        let m = |arms: &str| rules(&format!("{GRADES}fn main()\n    g := Grade.A\n    match g\n{arms}"));
        assert!(m("        A => print(1)\n        B => print(2)\n        C => print(3)\n").is_empty());
        assert_eq!(m("        A => print(1)\n        B => print(2)\n"), vec!["expr.match.2"]);
        assert_eq!(
            m("        A => print(1)\n        A => print(2)\n        _ => print(3)\n"),
            vec!["expr.match.3", "expr.match.5"]
        );
        assert_eq!(m("        _ => print(1)\n        B => print(2)\n"), vec!["expr.match.3", "expr.match.5"]);
    }

    #[test]
    fn if_used_as_value_needs_else_and_agreement() {
        assert_eq!(rules("fn main()\n    x := if true\n        1\n"), vec!["expr.if.4"]);
        assert_eq!(rules("fn main()\n    x := if true\n        1\n    else\n        \"a\"\n"), vec!["expr.if.4"]);
        assert_eq!(rules("fn main()\n    if 1\n        print(1)\n"), vec!["expr.if.2"]);
    }

    #[test]
    fn assignment_needs_a_mutable_place() {
        assert_eq!(rules("fn main()\n    x := 1\n    x = 2\n"), vec!["expr.bind.3"]);
        assert_eq!(rules("fn main()\n    y = 2\n"), vec!["expr.bind.3"]);
        assert_eq!(rules("fn f(p int)\n    p = 1\nfn main()\n    f(1)\n"), vec!["own.param.1"]);
        assert_eq!(rules("fn main()\n    mut x := 1\n    x = \"a\"\n"), vec!["expr.bind.6"]);
        assert!(rules("fn main()\n    mut xs := [1]\n    xs[0] += 1\n    xs.push(2)\n").is_empty());
    }

    #[test]
    fn exclusivity_is_checked_statically_when_certain() {
        let src = "fn f(mut a int, mut b int)\n    a = b\nfn main()\n    mut xs := [1, 2]\n    i := 0\n    f(mut xs[0], mut xs[0])\n    f(mut xs[0], mut xs[1])\n    f(mut xs[i], mut xs[i])\n";
        assert_eq!(rules(src), vec!["own.excl.2", "own.excl.2"]);
    }

    #[test]
    fn read_only_arguments_may_overlap_mut_ones() {
        // own.excl.4: they are passed as copies.
        let src = "fn f(mut a int, b int)\n    a = b\nfn main()\n    mut xs := [1, 2]\n    f(mut xs[0], xs[0])\n    xs.push(xs[0])\n    xs.swap(0, xs[1])\n";
        assert!(rules(src).is_empty());
    }

    #[test]
    fn operators_need_matching_types() {
        assert_eq!(rules("fn main()\n    x := 1 + \"a\"\n"), vec!["trait.op.3"]);
        assert_eq!(rules("fn main()\n    x := true * false\n"), vec!["trait.op.1"]);
        assert_eq!(rules("fn main()\n    x := 1 and true\n"), vec!["type.bool.1"]);
    }

    #[test]
    fn redeclaring_a_mut_variable_suggests_assignment() {
        let d = diags("fn main()\n    mut best := 1\n    for i in 0..3\n        best := i\n");
        assert_eq!(d[0].rule, "expr.bind.4");
        assert_eq!(d[0].help.as_deref(), Some("`best` is a `mut` variable; to change it, write `best = ...`"));
    }

    #[test]
    fn inferred_error_sets_reach_a_fixed_point() {
        let src = "error enum E\n    Bad\nfn a(n int) !int\n    if n == 0\n        return E.Bad\n    b(n - 1)?\nfn b(n int) !int\n    a(n)?\nfn only(n int) int ! E\n    b(n)?\nfn main()\n    x := only(1) else 0\n";
        assert!(rules(src).is_empty(), "{:?}", rules(src));
    }

    #[test]
    fn edit_distance_basics() {
        assert_eq!(edit_distance("prnt", "print"), 1);
        assert_eq!(edit_distance("", "abc"), 3);
    }
}
