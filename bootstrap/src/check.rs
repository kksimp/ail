//! Checker: names, types, places and control flow for the supported subset.
//!
//! Rules enforced: mod.main.1, mod.main.3, mod.unit.1, expr.name.1,
//! expr.bind.1-6, expr.block.2-3, expr.call.1-2, expr.cmp.1, expr.if.1-2,
//! expr.if.4-6, expr.hole.1, expr.loop.1-6, expr.match.2-6, expr.op.2-3, type.bool.1,
//! type.coll.2-7, type.str.2-5, type.struct.3-6, type.enum.3, type.enum.5-6,
//! type.opt.4-5, trait.method.1-5, trait.decl.1-2, trait.impl.1-6,
//! trait.self.1, trait.res.1, trait.op.1, trait.op.3, trait.any.1-5,
//! trait.std.1-4, trait.derive.1, trait.auto.1-4, gen.decl.1-2,
//! gen.bound.1-4, gen.method.1-3, gen.infer.1-2, gen.anon.1, own.param.1-2,
//! own.excl.1-2, own.excl.5, gram.generic.1, lex.str.2, err.decl.1-3,
//! err.type.1-2, err.type.5-6, err.prop.1-3, err.use.1-2, err.else.1-3,
//! err.ctx.2, err.text.1. All errors are collected in one pass
//! (ai-author-wishlist 2.4).
//!
//! The checker also records facts the interpreter needs: which bare names are
//! enum variants (type.enum.3), which method calls go through a trait, and
//! which `impl` implements each trait for each type.

mod absent;
mod builtins;
pub mod conc;
mod contracts;
mod comprehension;
pub mod facts;
mod keyfn;
mod stdlib;
pub mod effects;
pub mod parallel;
mod moves;
mod ownership;
mod privacy;
mod testing;
mod traits;
mod ui;

use std::collections::HashMap;

use builtins::builtin;
use traits::{Cand, bind, bound, subst, subst_unknown};

use crate::ast::{
    Arg, ArgKind, BinOp, Block, Cond, Else, Expr, Field, FnDecl, GenericParam, InterpPart, Literal, Owner, PathStep,
    Pattern, Program, Stmt, TypeKind, TypeRef, UnOp, Within, field_slots, is_prelude, place_path,
};
use crate::fasthash::{FastMap, FastSet};
use crate::diag::{Diagnostic, Edit, Fix, Pos};
use crate::lexer::UNSUPPORTED;
use crate::num::Num;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ty {
    Int,
    /// A sized numeric type other than `int` and `float` (type.num.1).
    Num(Num),
    /// `float`, `f64` (type.num.1-2)
    Float,
    Str,
    /// `char` (type.str.1)
    Char,
    Bool,
    /// The result of an expression that produces no value (e.g. `print(...)`).
    Unit,
    /// A block that always leaves through `return`, `break` or `continue` (expr.block.3).
    Never,
    /// An error was already reported here, or a type isn't known; suppresses
    /// follow-on errors.
    Err,
    /// A struct and its type arguments (gen.decl.1).
    Struct(usize, Vec<Ty>),
    Enum(usize, Vec<Ty>),
    List(Box<Ty>),
    /// `Map[K, V]` and `Set[T]` (type.coll.1, type.map.1)
    Map(Box<Ty>, Box<Ty>),
    Set(Box<Ty>),
    /// `T?` (type.opt.2)
    Opt(Box<Ty>),
    /// `T ! E`; `T` is `Unit` for a bare `!` (err.type.1-2).
    Res(Box<Ty>, Errs),
    /// One of several error types, e.g. the `err` of `else err` (err.type.5).
    Union(Vec<Ty>),
    /// A type parameter, by id (gen.decl.1), including a trait's `Self`.
    Param(usize),
    /// `any Trait` (trait.any.1).
    Any(usize),
    /// `ref T` (own.ref.1).
    Ref(Box<Ty>),
    /// `Task[T]`, `TaskGroup` (with `Unit`), `Channel[T]` and `Mutex[T]`
    /// (conc.task.1, conc.share.2-3, ADRs 0186-0189).
    Conc(conc::Conc, Box<Ty>),
}

/// The error part of `T ! E`: an explicit set, or the inferred set of a
/// function, by index (err.type.2, err.type.6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Errs {
    Set(Vec<Ty>),
    Inferred(usize),
}

/// A type constructor: what an `impl` block is for (trait.impl.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Con {
    Int,
    Float,
    Str,
    Char,
    Bool,
    List,
    Struct(usize),
    Enum(usize),
}

/// A user-declared type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Def {
    Struct(usize),
    Enum(usize),
}

/// Sort key that keeps error sets in a canonical order.
fn ty_key(t: &Ty) -> (u8, usize) {
    match t {
        Ty::Struct(i, _) => (0, *i),
        Ty::Enum(i, _) => (1, *i),
        _ => (2, 0),
    }
}

/// The standard traits (trait.std.1-3, type.map.2) and `enum Ordering`, by index.
#[derive(Debug, Default, Clone, Copy)]
pub struct Std {
    pub eq: usize,
    pub ord: usize,
    pub to_str: usize,
    pub hash: usize,
    pub ordering: usize,
    /// `Drop`, the cleanup trait of unique types (own.uniq.3, ADR 0191).
    pub drop: Option<usize>,
}

/// A method call that goes through a trait.
#[derive(Debug, Clone, Copy)]
pub enum Call {
    /// `x.f(...)`, dispatched on the value of `x`.
    Trait(usize),
    /// `Trait.f(x, ...)`, dispatched on the value of the first argument (trait.res.1).
    Qualified(usize),
}

/// Facts the interpreter needs from the checker.
#[derive(Debug, Default)]
pub struct Resolved {
    /// Bare enum variants, by the position of the name: (enum, variant).
    pub variants: FastMap<Pos, (usize, usize)>,
    /// Method calls through a trait, by the position of the method name.
    pub calls: FastMap<Pos, Call>,
    /// The `impl` block for each trait and type constructor (trait.impl.6).
    pub impls: FastMap<(usize, Con), usize>,
    /// Empty `{}` literals that are sets rather than maps (type.coll.3).
    pub empty_sets: FastSet<Pos>,
    /// `e?` on an optional, which passes on `none` (err.prop.2); every other
    /// `?` passes on only a failure, so a successful `none` stays a value.
    pub optional_try: FastSet<Pos>,
    /// `e?` on an optional in a function whose errors are inferred, which
    /// fails with the prelude's `Absent` on `none` (err.prop.5, ADR 0279).
    pub absent_try: FastSet<Pos>,
    /// `e?` on an optional inside the value of an `else`, which goes to that
    /// `else` instead of leaving the function (err.prop.4, ADR 0252).
    pub else_try: FastSet<Pos>,
    /// Integer literals that are floats because a float is expected (type.num.10).
    pub float_literals: FastSet<Pos>,
    /// Literals typed by context as a sized numeric type (type.num.13).
    pub sized_literals: FastMap<Pos, Num>,
    /// The function each method or associated function call runs, by the
    /// position of its name, when that is known statically (eff.infer.3).
    pub targets: HashMap<Pos, usize>,
    pub std: Std,
    /// Calls to the standard modules, by the position of the name (std.use.2).
    pub std_calls: HashMap<Pos, crate::stdlib::StdCall>,
    /// The declaration a name, call, method, field or variant refers to, by
    /// the use's position, as an `ail api` symbol id (api.sym.1, ADR 0151).
    /// Trait calls are in `calls` and bare variants in `variants` instead.
    pub symbols: HashMap<Pos, String>,
    /// Methods called on a task, group, channel or mutex handle, by the
    /// position of the name: they synchronise with other tasks (conc.par.2).
    pub handle_calls: FastSet<Pos>,
    /// Names whose use moves the binding's value out (own.uniq.2,
    /// own.move.2), by position, with the rule that makes it a move. The
    /// interpreter takes the value out of the binding there.
    pub moves: FastMap<Pos, &'static str>,
    /// The root positions of places written through a `ref` (own.ref.4),
    /// which give a function `mutates` (eff.infer.2).
    pub ref_writes: FastSet<Pos>,
    /// Types of locals and expressions, recorded for `ail api` only (api.loc.1).
    pub facts: facts::Facts,
}

/// Names available without import (mod.main.3, ADR 0084).
const PRELUDE: &[&str] = &["print", "unreachable", "assert", "assert_eq"];

const BUILTIN_TYPES: &[&str] = &[
    "int", "i64", "i8", "i16", "i32", "u8", "u16", "u32", "u64", "float", "f64", "f32", "str", "char", "bool", "List",
    "Map", "Set", "Task", "TaskGroup", "Channel", "Mutex",
];

/// Type names the spec defines that the bootstrap doesn't support yet.
const SPEC_TYPES_NOT_YET: &[&str] = &["i8", "i16", "i32", "u8", "u16", "u32", "u64", "f32", "Option"];

pub fn check(prog: &Program) -> (Vec<Diagnostic>, Resolved) {
    check_with(prog, false)
}

/// `check`, also recording the types of locals and expressions (api.loc.1).
pub fn check_recording(prog: &Program) -> (Vec<Diagnostic>, Resolved) {
    check_with(prog, true)
}

fn check_with(prog: &Program, record: bool) -> (Vec<Diagnostic>, Resolved) {
    let mut c = Checker {
        prog,
        diags: Vec::new(),
        types: HashMap::new(),
        traits: HashMap::new(),
        tparams: Vec::new(),
        struct_params: Vec::new(),
        enum_params: Vec::new(),
        list_param: 0,
        trait_self: Vec::new(),
        impl_info: Vec::new(),
        impl_of: HashMap::new(),
        std: Std::default(),
        field_types: Vec::new(),
        variant_types: Vec::new(),
        fns: HashMap::new(),
        methods: HashMap::new(),
        sigs: Vec::new(),
        fn_scopes: Vec::new(),
        fn_self: Vec::new(),
        inferred: vec![Vec::new(); prog.fns.len()],
        consts: HashMap::new(),
        const_types: vec![None; prog.consts.len()],
        const_busy: vec![false; prog.consts.len()],
        resolved: Resolved::default(),
        scope: Vec::new(),
        self_ty: None,
        deferred: Some(Vec::new()),
        locals: Vec::new(),
        pending: Vec::new(),
        ret: Ty::Unit,
        loops: 0,
        record: false,
        anchored: 0,
        anchors: HashMap::new(),
        bound: HashMap::new(),
        catching: 0,
        caught: 0,
    };
    c.list_param = c.new_param("T", Vec::new());
    c.collect_traits();
    c.collect_types();
    c.collect_impls();
    c.resolve_fields();
    c.check_unique_fields();
    c.collect_fns();
    c.collect_consts();
    c.check_uses();
    c.check_deferred_bounds();
    c.check_main();
    c.check_attrs();
    c.check_impls();
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
    c.record = record;
    for (i, f) in prog.fns.iter().enumerate() {
        c.check_fn(i, f);
    }
    c.record = false;
    c.resolved.impls = c.impl_of.iter().map(|(k, v)| (*k, *v)).collect();
    c.resolved.std = c.std;
    c.check_drop_impls();
    c.diags.extend(moves::check(prog, &c.resolved));
    c.diags.extend(effects::check_pure(prog, &c.resolved));
    c.diags.extend(conc::check_handles(prog));
    c.diags.extend(effects::check_contracts(prog, &c.resolved));
    c.diags.extend(effects::check_constants(prog, &c.resolved));
    c.diags.extend(parallel::check_par(prog, &c.resolved));
    (c.diags, c.resolved)
}

/// A function's signature: receiver (`Some(true)` for `mut self`), parameters
/// (`true` for `mut`), return type (`Unit` when there is none), and the type
/// parameters it is generic over. Those are, in order: `owner` that come from
/// the type it belongs to (or a trait's `Self`), `declared` of its own, and
/// any anonymous ones (gen.anon.1).
#[derive(Clone)]
struct Sig {
    receiver: Option<bool>,
    params: Vec<(bool, Ty)>,
    /// Which parameters are `move` (own.move.1); empty when none are.
    moves: Vec<bool>,
    ret: Ty,
    generics: Vec<usize>,
    owner: usize,
    declared: usize,
}

impl Sig {
    fn plain(receiver: Option<bool>, params: Vec<(bool, Ty)>, ret: Ty) -> Self {
        Sig { receiver, params, moves: Vec::new(), ret, generics: Vec::new(), owner: 0, declared: 0 }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Binding,
    Param,
    /// A `move` parameter, which owns its value (own.move.1).
    MoveParam,
    SelfParam,
    LoopVar,
}

/// A local declared from `[]` or `{}`, typed by its first use (type.coll.13).
struct Pending {
    name: String,
    /// The literal, and whether it is `{}` (a map or a set).
    lit: Pos,
    brace: bool,
    /// Its index in `locals`.
    index: usize,
}

struct Local {
    name: String,
    ty: Ty,
    mutable: bool,
    kind: Kind,
    /// Where a `:=` binding's name is written, for fixes (fix.edit.1).
    decl: Option<Pos>,
}

/// A type parameter: its name and the traits it must implement (gen.bound.1).
struct TParam {
    name: String,
    bounds: Vec<usize>,
}

/// An `impl` block: its trait and the implementing type with the block's
/// type parameters (`None` and `Err` after an error).
struct ImplInfo {
    tr: Option<usize>,
    target: Ty,
    params: Vec<usize>,
}

struct Checker<'p> {
    prog: &'p Program,
    diags: Vec<Diagnostic>,
    /// User-declared types by name; the first declaration wins.
    types: HashMap<&'p str, Def>,
    traits: HashMap<&'p str, usize>,
    /// Type parameters by id.
    tparams: Vec<TParam>,
    /// The type parameters of each struct and enum, parallel to `prog`.
    struct_params: Vec<Vec<usize>>,
    enum_params: Vec<Vec<usize>>,
    /// The parameter of the built-in `List[T]`.
    list_param: usize,
    /// Each trait's `Self` parameter (trait.self.1).
    trait_self: Vec<usize>,
    impl_info: Vec<ImplInfo>,
    impl_of: HashMap<(usize, Con), usize>,
    std: Std,
    /// Field types, parallel to `prog.structs[i].fields`, in terms of the
    /// struct's type parameters.
    field_types: Vec<Vec<Ty>>,
    /// Field types of each enum's variants, parallel to `prog.enums`.
    variant_types: Vec<Vec<Vec<Ty>>>,
    /// Free functions: name to index in `prog.fns` and `sigs`.
    fns: HashMap<&'p str, usize>,
    /// Methods and associated functions: type name, then function name.
    methods: HashMap<&'p str, HashMap<&'p str, usize>>,
    sigs: Vec<Sig>,
    /// Per function: the type parameters in scope, and `Self`.
    fn_scopes: Vec<Vec<(String, usize)>>,
    fn_self: Vec<Option<Ty>>,
    /// The inferred error set of each function with a `!T` result (err.type.2).
    inferred: Vec<Vec<Ty>>,
    /// Constants by name, their types once known, and which are being typed
    /// (to report cycles) (mod.const.1-3).
    consts: HashMap<&'p str, usize>,
    const_types: Vec<Option<Ty>>,
    const_busy: Vec<bool>,
    resolved: Resolved,
    /// Type parameter names in scope, and what `Self` means here.
    scope: Vec<(String, usize)>,
    self_ty: Option<Ty>,
    /// Bounds to check once every declaration is known (gen.bound.3):
    /// the type, the trait, where, and what requires it.
    deferred: Option<Vec<(Ty, usize, Pos, String)>>,
    /// Locals of the function being checked, innermost last.
    locals: Vec<Local>,
    /// Locals declared from `[]` or `{}` whose type isn't known yet (type.coll.13).
    pending: Vec<Pending>,
    ret: Ty,
    loops: usize,
    /// Record `resolved.facts` (api.loc.1); on for the final pass only.
    record: bool,
    /// The locals below this index already have an anchor (`anchors`).
    anchored: usize,
    /// The anchor of each local without a declaration, by index in `locals`.
    anchors: HashMap<usize, Pos>,
    /// Where each local's name is written, by index in `locals`.
    bound: HashMap<usize, Pos>,
    /// How many `else` values enclose the expression being checked, and how
    /// many optional `?` they caught (err.prop.4).
    catching: u32,
    caught: u32,
}

/// `a` and `b` are the same type, or an error already reported makes it moot.
fn same(a: &Ty, b: &Ty) -> bool {
    match (a, b) {
        (Ty::Err, _) | (_, Ty::Err) => true,
        (Ty::List(x), Ty::List(y)) | (Ty::Opt(x), Ty::Opt(y)) | (Ty::Set(x), Ty::Set(y)) => same(x, y),
        (Ty::Map(k, v), Ty::Map(l, w)) => same(k, l) && same(v, w),
        (Ty::Conc(a, x), Ty::Conc(b, y)) => a == b && same(x, y),
        (Ty::Struct(s, xs), Ty::Struct(t, ys)) | (Ty::Enum(s, xs), Ty::Enum(t, ys)) => {
            s == t && xs.len() == ys.len() && xs.iter().zip(ys).all(|(x, y)| same(x, y))
        }
        _ => a == b,
    }
}

/// Whether `t` contains an unknown part.
fn has_err(t: &Ty) -> bool {
    match t {
        Ty::Err => true,
        Ty::List(x) | Ty::Opt(x) | Ty::Set(x) | Ty::Conc(_, x) => has_err(x),
        Ty::Map(k, v) => has_err(k) || has_err(v),
        Ty::Res(x, _) => has_err(x),
        Ty::Struct(_, xs) | Ty::Enum(_, xs) | Ty::Union(xs) => xs.iter().any(has_err),
        _ => false,
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

    fn new_param(&mut self, name: &str, bounds: Vec<usize>) -> usize {
        self.tparams.push(TParam { name: name.to_string(), bounds });
        self.tparams.len() - 1
    }

    fn scope_of(&self, ids: &[usize]) -> Vec<(String, usize)> {
        ids.iter().map(|&id| (self.tparams[id].name.clone(), id)).collect()
    }

    fn def_params(&self, d: Def) -> &[usize] {
        match d {
            Def::Struct(s) => &self.struct_params[s],
            Def::Enum(e) => &self.enum_params[e],
        }
    }

    fn def_name(&self, d: Def) -> &'p str {
        match d {
            Def::Struct(s) => &self.prog.structs[s].name,
            Def::Enum(e) => &self.prog.enums[e].name,
        }
    }

    fn def_pos(&self, d: Def) -> Pos {
        match d {
            Def::Struct(s) => self.prog.structs[s].pos,
            Def::Enum(e) => self.prog.enums[e].pos,
        }
    }

    fn def_ty(d: Def, args: Vec<Ty>) -> Ty {
        match d {
            Def::Struct(s) => Ty::Struct(s, args),
            Def::Enum(e) => Ty::Enum(e, args),
        }
    }

    fn collect_types(&mut self) {
        let prog = self.prog;
        let decls = (prog.structs.iter().enumerate().map(|(i, s)| (s.name.as_str(), s.pos, Def::Struct(i))))
            .chain(prog.enums.iter().enumerate().map(|(i, e)| (e.name.as_str(), e.pos, Def::Enum(i))));
        for (name, pos, def) in decls {
            let built_in = self.types.get(name).is_some_and(|d| is_prelude(self.def_pos(*d)));
            if BUILTIN_TYPES.contains(&name) || SPEC_TYPES_NOT_YET.contains(&name) || built_in {
                self.error("mod.unit.1", pos, format!("`{name}` is already a built-in type"));
            } else if self.types.contains_key(name) || self.traits.contains_key(name) {
                let mut d =
                    Diagnostic::new("mod.unit.1", pos, format!("`{name}` is declared more than once in this module"));
                if let Some(first) = self.types.get(name) {
                    d = d.note(self.def_pos(*first), format!("`{name}` is first declared here"));
                }
                self.diags.push(d);
            } else {
                self.types.insert(name, def);
            }
        }
        if let Some(i) = prog.enums.iter().position(|e| e.name == "Ordering" && is_prelude(e.pos)) {
            self.std.ordering = i;
        }
        // Type parameters (gen.decl.1-2).
        for s in &prog.structs {
            let ids = self.declare_generics(&s.generics);
            self.struct_params.push(ids);
        }
        for e in &prog.enums {
            let ids = self.declare_generics(&e.generics);
            self.enum_params.push(ids);
        }
    }

    /// Create the type parameters of a declaration, with their bounds
    /// (gen.decl.2, gen.bound.1).
    fn declare_generics(&mut self, gs: &[GenericParam]) -> Vec<usize> {
        let mut ids = Vec::new();
        for (k, g) in gs.iter().enumerate() {
            let name = g.name.as_str();
            if gs[..k].iter().any(|h| h.name == g.name) || self.scope.iter().any(|(n, _)| n == name) {
                self.error("gen.decl.2", g.pos, format!("the type parameter `{name}` is declared twice"));
            } else if self.types.contains_key(name) || BUILTIN_TYPES.contains(&name) || name == "Self" {
                self.error_help(
                    "gen.decl.2",
                    g.pos,
                    format!("`{name}` is already a type"),
                    "choose another name for the type parameter, e.g. `T`",
                );
            } else if self.traits.contains_key(name) {
                self.error_help(
                    "gen.decl.2",
                    g.pos,
                    format!("`{name}` is a trait, so it can't name a type parameter"),
                    format!("to require the trait, write `[T {name}]`"),
                );
            }
            let bounds = self.resolve_bounds(&g.bounds);
            ids.push(self.new_param(name, bounds));
        }
        ids
    }

    fn resolve_bounds(&mut self, bounds: &[(String, Pos)]) -> Vec<usize> {
        bounds.iter().filter_map(|(n, p)| self.trait_named(n, *p, "gen.bound.1")).collect()
    }

    /// The trait called `n`, reporting when there's none.
    fn trait_named(&mut self, n: &str, pos: Pos, rule: &'static str) -> Option<usize> {
        if let Some(&t) = self.traits.get(n) {
            return Some(t);
        }
        if self.types.contains_key(n) || BUILTIN_TYPES.contains(&n) {
            self.error_help(
                rule,
                pos,
                format!("`{n}` is a type, not a trait"),
                "a constraint or `impl` names a trait, e.g. `Ord`",
            );
            return None;
        }
        let names: Vec<&str> = self.traits.keys().copied().collect();
        let mut d = Diagnostic::new("expr.name.1", pos, format!("unknown trait `{n}`"));
        if let Some(s) = suggest(n, &names) {
            d = d.did_you_mean(n, s);
        }
        self.diags.push(d);
        None
    }

    fn resolve_fields(&mut self) {
        let prog = self.prog;
        for (i, e) in prog.enums.iter().enumerate() {
            self.scope = self.scope_of(&self.enum_params[i].clone());
            let mut variants = Vec::new();
            for (k, v) in e.variants.iter().enumerate() {
                if e.variants[..k].iter().any(|w| w.name == v.name) {
                    self.error("type.enum.5", v.pos, format!("variant `{}` is declared twice in `{}`", v.name, e.name));
                }
                variants.push(self.field_list(&v.fields, &format!("{}.{}", e.name, v.name)));
            }
            self.variant_types.push(variants);
        }
        for (i, s) in prog.structs.iter().enumerate() {
            self.scope = self.scope_of(&self.struct_params[i].clone());
            let tys = self.field_list(&s.fields, &s.name);
            self.field_types.push(tys);
        }
        self.scope.clear();
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

    /// The type parameters of a method's or `impl`'s owner: copies of the
    /// type's declared ones, with any bounds the head adds (gen.method.1-2).
    fn owner_params(&mut self, owner: &Owner, declared: &[usize]) -> Vec<usize> {
        let names: Vec<String> = declared.iter().map(|&d| self.tparams[d].name.clone()).collect();
        let matching = owner
            .params
            .as_ref()
            .is_none_or(|given| given.len() == names.len() && given.iter().zip(&names).all(|(g, n)| g.name == *n));
        if !matching {
            let shown = if names.is_empty() {
                format!("`{}` has no type parameters", owner.name)
            } else {
                format!(
                    "`{}` has the type parameters {}",
                    owner.name,
                    list_names(&names.iter().map(String::as_str).collect::<Vec<_>>())
                )
            };
            self.error_help(
                "gen.method.2",
                owner.pos,
                format!("{shown}; a method or `impl` names them in order, to add constraints"),
                format!("write `{}` alone, or e.g. `{}[{} Ord]`", owner.name, owner.name, names.join(", ")),
            );
        }
        let mut ids = Vec::new();
        for (k, &d) in declared.iter().enumerate() {
            let mut bounds = self.tparams[d].bounds.clone();
            if matching && let Some(given) = &owner.params {
                for b in self.resolve_bounds(&given[k].bounds) {
                    if !bounds.contains(&b) {
                        bounds.push(b);
                    }
                }
            }
            // With the count right but a name wrong, keep the name as written
            // so its uses don't cascade.
            let name = match &owner.params {
                Some(given) if given.len() == names.len() => given[k].name.clone(),
                _ => names[k].clone(),
            };
            ids.push(self.new_param(&name, bounds));
        }
        ids
    }

    fn collect_fns(&mut self) {
        let prog = self.prog;
        for (i, f) in prog.fns.iter().enumerate() {
            // The owner's type parameters and `Self` (gen.method.1-2, trait.self.1).
            let (owner_ids, self_ty) = match f.within {
                Within::Trait(t) => (vec![self.trait_self[t]], Some(Ty::Param(self.trait_self[t]))),
                Within::Impl(k) => (self.impl_info[k].params.clone(), Some(self.impl_info[k].target.clone())),
                Within::TopLevel => match f.owner.as_ref().and_then(|o| self.types.get(o.name.as_str()).copied()) {
                    Some(def) => {
                        let declared = self.def_params(def).to_vec();
                        let ids = self.owner_params(f.owner.as_ref().expect("an owner"), &declared);
                        let args = ids.iter().map(|&p| Ty::Param(p)).collect();
                        (ids, Some(Self::def_ty(def, args)))
                    }
                    None => (Vec::new(), None),
                },
            };
            self.scope = self.scope_of(&owner_ids);
            self.self_ty = self_ty.clone();
            let own = self.declare_generics(&f.generics);
            let own_scope = self.scope_of(&own);
            self.scope.extend(own_scope);
            let mut generics = owner_ids.clone();
            generics.extend(&own);
            // A trait as a parameter's type declares an anonymous type parameter (gen.anon.1).
            let mut params = Vec::new();
            for p in &f.params {
                let ty = match &p.ty.kind {
                    TypeKind::Named { name, args }
                        if args.is_empty()
                            && self.traits.contains_key(name.as_str())
                            && !self.scope.iter().any(|(n, _)| n == name) =>
                    {
                        let id = self.new_param(name, vec![self.traits[name.as_str()]]);
                        generics.push(id);
                        Ty::Param(id)
                    }
                    _ => self.resolve_type(&p.ty),
                };
                params.push((p.mutable, ty));
            }
            let mut ret = f.ret.as_ref().map_or(Ty::Unit, |t| self.resolve_ret(t, i));
            // test.attr.4: a test can always fail, so `?` works in it.
            if f.ret.is_none() && f.is_test() {
                ret = Ty::Res(Box::new(Ty::Unit), Errs::Inferred(i));
            }
            // A `main` with a wrong return type is reported once (mod.main.1);
            // its body is then checked as returning nothing.
            if f.top_level()
                && f.owner.is_none()
                && f.name == "main"
                && !matches!(&ret, Ty::Res(ok, _) if **ok == Ty::Unit)
            {
                ret = Ty::Unit;
            }
            if let Within::Trait(_) = f.within {
                if f.receiver.is_none() {
                    self.unsupported(f.pos, "a trait function without `self` is not designed yet (OQ-039)");
                }
                if !f.generics.is_empty() {
                    self.unsupported(
                        f.pos,
                        "type parameters on a trait's methods are not supported by the bootstrap yet",
                    );
                }
                if matches!(ret, Ty::Res(_, Errs::Inferred(_))) {
                    self.unsupported(
                        f.ret.as_ref().map_or(f.pos, |t| t.pos),
                        "a trait method lists its errors (`T ! E`); the bootstrap can't infer them across implementations",
                    );
                }
            }
            self.fn_scopes.push(std::mem::take(&mut self.scope));
            self.fn_self.push(self_ty);
            self.self_ty = None;
            let sig = Sig {
                receiver: f.receiver.map(|r| r.mutable),
                params,
                moves: if f.params.iter().any(|p| p.moved) { f.params.iter().map(|p| p.moved).collect() } else { Vec::new() },
                ret,
                generics,
                owner: owner_ids.len(),
                declared: own.len(),
            };
            self.sigs.push(sig);
            if f.top_level() {
                self.register_fn(i, f);
            }
        }
    }

    /// Make a top-level function callable by name (mod.unit.1, trait.method.5, trait.auto.4).
    fn register_fn(&mut self, i: usize, f: &'p FnDecl) {
        let name = f.name.as_str();
        match &f.owner {
            None if PRELUDE.contains(&name) => self.unsupported(
                f.pos,
                format!("redeclaring the prelude function `{name}` is not supported by the bootstrap"),
            ),
            None if self.fns.contains_key(name) || self.types.contains_key(name) || self.traits.contains_key(name) => {
                let mut d =
                    Diagnostic::new("mod.unit.1", f.pos, format!("`{name}` is declared more than once in this module"));
                if let Some(first) = self.fns.get(name).map(|j| self.prog.fns[*j].pos).or_else(|| self.types.get(name).map(|d| self.def_pos(*d))) {
                    d = d.note(first, format!("`{name}` is first declared here"));
                }
                self.diags.push(d);
            }
            None => {
                self.fns.insert(name, i);
            }
            Some(owner) => {
                let o = owner.name.as_str();
                if !self.types.contains_key(o) {
                    if BUILTIN_TYPES.contains(&o) || SPEC_TYPES_NOT_YET.contains(&o) {
                        self.error(
                            "trait.method.5",
                            owner.pos,
                            format!("methods of `{o}` can only be declared where `{o}` is declared"),
                        );
                    } else if self.traits.contains_key(o) {
                        self.error_help(
                            "trait.impl.1",
                            owner.pos,
                            format!("`{o}` is a trait; its methods are declared in its `trait` block"),
                            format!(
                                "to implement it for a type, write `impl {o} for Type` with the methods indented below"
                            ),
                        );
                    } else {
                        self.unknown_type(o, owner.pos);
                    }
                    return;
                }
                // trait.auto.4: `eq` and `to_str` belong to `Eq` and `ToStr`.
                for (method, tr) in [("eq", "Eq"), ("to_str", "ToStr")] {
                    if name == method {
                        let signature =
                            if method == "eq" { "fn eq(self, other Self) bool" } else { "fn to_str(self) str" };
                        self.error_help(
                            "trait.auto.4",
                            f.pos,
                            format!("`{method}` is the method of the standard trait `{tr}`, which every type has"),
                            format!("to replace `{o}`'s, write `impl {tr} for {o}` with `{signature}` indented below"),
                        );
                    }
                }
                let table = self.methods.entry(o).or_default();
                if let Some(first) = table.get(name).map(|j| self.prog.fns[*j].pos) {
                    let d = Diagnostic::new("mod.unit.1", f.pos, format!("`{o}.{name}` is declared more than once"))
                        .note(first, format!("`{o}.{name}` is first declared here"));
                    self.diags.push(d);
                } else {
                    table.insert(name, i);
                }
            }
        }
    }

    /// Top-level constants (mod.const.1-3): register their names, check that
    /// each value is a constant expression, and type them in dependency order.
    fn collect_consts(&mut self) {
        let prog = self.prog;
        for (i, c) in prog.consts.iter().enumerate() {
            let name = c.name.as_str();
            let taken = self.fns.contains_key(name)
                || self.types.contains_key(name)
                || self.traits.contains_key(name)
                || self.consts.contains_key(name)
                || PRELUDE.contains(&name)
                || BUILTIN_TYPES.contains(&name);
            if taken {
                let mut d =
                    Diagnostic::new("mod.unit.1", c.pos, format!("`{name}` is declared more than once in this module"));
                let first = (self.consts.get(name).map(|j| prog.consts[*j].pos))
                    .or_else(|| self.fns.get(name).map(|j| prog.fns[*j].pos))
                    .or_else(|| self.types.get(name).map(|d| self.def_pos(*d)));
                if let Some(first) = first {
                    d = d.note(first, format!("`{name}` is first declared here"));
                }
                self.diags.push(d);
            } else {
                self.consts.insert(name, i);
            }
            self.constant_expr(&c.value);
        }
        for i in 0..prog.consts.len() {
            self.const_type(i);
        }
    }

    /// The type of constant `i`, typing it first if needed (mod.const.3).
    fn const_type(&mut self, i: usize) -> Ty {
        if let Some(t) = &self.const_types[i] {
            return t.clone();
        }
        let c = &self.prog.consts[i];
        if self.const_busy[i] {
            self.error_help(
                "mod.const.3",
                c.pos,
                format!("the value of `{}` depends on itself", c.name),
                "constants can refer to other constants, but not in a cycle",
            );
            self.const_types[i] = Some(Ty::Err);
            return Ty::Err;
        }
        self.const_busy[i] = true;
        let saved = (std::mem::take(&mut self.locals), std::mem::take(&mut self.scope), self.self_ty.take());
        let t = self.expr(&c.value, None, true);
        (self.locals, self.scope, self.self_ty) = saved;
        self.const_busy[i] = false;
        if t == Ty::Unit {
            self.error("mod.const.2", c.value.pos(), format!("`{}` would be a constant with no value", c.name));
        }
        if matches!(t, Ty::Res(..)) {
            self.error_help(
                "ct.const.3",
                c.value.pos(),
                format!("`{}` would hold a result that may fail; a constant can't hold a failure", c.name),
                "supply a fallback with `else`: `NAME := parse(text) else 0`",
            );
        }
        let t = self.const_types[i].clone().unwrap_or(t);
        self.const_types[i] = Some(t.clone());
        t
    }

    /// Report the parts of a constant's value that the bootstrap can't
    /// compute. Calls, method calls, field access, indexing and ranges are
    /// allowed (ct.const.1, ADR 0225); whether what they call is
    /// compile-time-safe is checked after effect inference (ct.const.2).
    /// `if`, `match`, `?` and `else` with a block are left to a function.
    fn constant_expr(&mut self, e: &Expr) {
        let unsupported = |c: &mut Self, at: Pos, what: &str, rule: &str| {
            c.diags.push(
                Diagnostic::new(
                    UNSUPPORTED,
                    at,
                    format!("the bootstrap can't compute {what} in a constant yet ({rule})"),
                )
                .help("move the logic into a function and call it: `NAME := build()`"),
            );
        };
        match e {
            Expr::Str(..) | Expr::Int(..) | Expr::Float(..) | Expr::Char(..) | Expr::Bool(..) | Expr::Name(..) => {}
            Expr::None(_) | Expr::Hole(_) | Expr::TypeArgs { .. } => {}
            Expr::Interp(parts, _) => {
                for p in parts {
                    if let InterpPart::Expr(x) | InterpPart::Fmt(x, _) = p {
                        self.constant_expr(x);
                    }
                }
            }
            Expr::List(items, _) | Expr::Set(items, _) => items.iter().for_each(|x| self.constant_expr(x)),
            Expr::Map(pairs, _) => pairs.iter().for_each(|(k, v)| {
                self.constant_expr(k);
                self.constant_expr(v);
            }),
            Expr::Unary { operand, .. } => self.constant_expr(operand),
            // A constant is shared by the whole program, so it can't hold a cell.
            Expr::Ref { pos, .. } => self.error_help(
                "mod.const.2",
                *pos,
                "a constant's value can't contain a `ref` cell".to_string(),
                "a constant is shared by the whole program; make the cell inside a function",
            ),
            Expr::Binary { lhs, rhs, .. } => {
                self.constant_expr(lhs);
                self.constant_expr(rhs);
            }
            Expr::Field { recv, .. } => self.constant_expr(recv),
            Expr::Call { callee, args, .. } => {
                self.constant_expr(callee);
                args.iter().for_each(|a| self.constant_expr(&a.value));
            }
            Expr::Method { recv, args, .. } => {
                self.constant_expr(recv);
                args.iter().for_each(|a| self.constant_expr(&a.value));
            }
            Expr::Index { recv, index, .. } => {
                self.constant_expr(recv);
                self.constant_expr(index);
            }
            Expr::Range { lo, hi, .. } => {
                lo.iter().chain(hi.iter()).for_each(|x| self.constant_expr(x));
            }
            Expr::Else { value, otherwise, pos } => match &**otherwise {
                Else::Value(d) => {
                    self.constant_expr(value);
                    self.constant_expr(d);
                }
                _ => unsupported(self, *pos, "`else` with a block or a jump", "ct.const.1"),
            },
            Expr::If { pos, .. } => unsupported(self, *pos, "`if`", "ct.const.1"),
            Expr::Match { pos, .. } => unsupported(self, *pos, "`match`", "ct.const.1"),
            Expr::Try { pos, .. } => unsupported(self, *pos, "`?`", "ct.const.3"),
            Expr::Comp { .. } => {}
        }
    }

    fn check_main(&mut self) {
        match self.prog.fns.iter().find(|f| f.name == "main" && f.owner.is_none() && f.top_level()) {
            None => self.diags.push(
                Diagnostic::new("mod.main.1", Pos { line: 1, col: 1 }, "program has no `main` function")
                    .help("add:\n    fn main()\n        print(\"Hello, world!\")"),
            ),
            Some(f) => {
                if let Some(g) = f.generics.first() {
                    self.error("mod.main.1", g.pos, "`main` can't have type parameters");
                }
                if !f.params.is_empty() {
                    self.error("mod.main.1", f.params[0].pos, "`main` takes no parameters");
                }
                let i = self.fns.get("main").copied();
                let fails_only = i.is_some_and(|i| matches!(&self.sigs[i].ret, Ty::Res(ok, _) if **ok == Ty::Unit));
                if let Some(ret) = &f.ret
                    && !fails_only
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
        // err.type.7: an error type can't be a success value (err.decl.3), so
        // `! E` is a result with no success value that fails with `E`.
        if errors.is_none() && self.is_error(&ok_ty) {
            return Ty::Res(Box::new(Ty::Unit), Errs::Set(vec![ok_ty]));
        }
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
            TypeKind::Ref(inner) => {
                let inner_ty = self.resolve_type(inner);
                self.no_unique(&inner_ty, inner.pos, "the contents of a `ref`");
                return Ty::Ref(Box::new(inner_ty));
            }
            TypeKind::Fallible { .. } => {
                self.unsupported(
                    t.pos,
                    "a type that can fail is only supported as a function's return type in the bootstrap (spec err.type.1)",
                );
                return Ty::Err;
            }
            TypeKind::Any(tr) => {
                return match self.trait_named(tr, t.pos, "trait.any.1") {
                    Some(i) => Ty::Any(i),
                    None => Ty::Err,
                };
            }
        };
        if let Some(&(_, id)) = self.scope.iter().rev().find(|(n, _)| n == name) {
            if !args.is_empty() {
                self.error("gen.decl.1", t.pos, format!("the type parameter `{name}` takes no type arguments"));
            }
            return Ty::Param(id);
        }
        if let Some(t) = self.std_type(name, args, t.pos) {
            return t;
        }
        let ty = match name.as_str() {
            "int" | "i64" => Ty::Int,
            "float" | "f64" => Ty::Float,
            n if Num::from_name(n).is_some() => Ty::Num(Num::from_name(n).expect("a sized type")),
            "str" => Ty::Str,
            "char" => Ty::Char,
            "bool" => Ty::Bool,
            "Self" => match self.self_ty.clone() {
                Some(t) => t,
                None => {
                    self.error_help(
                        "trait.self.1",
                        t.pos,
                        "`Self` is only a type in a trait, a method, or an `impl` block",
                        "write the type's name",
                    );
                    return Ty::Err;
                }
            },
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
                let elem = self.resolve_type(&args[0]);
                self.no_unique(&elem, args[0].pos, "a list element");
                return Ty::List(Box::new(elem));
            }
            "Map" | "Set" => {
                let want = if name == "Map" { 2 } else { 1 };
                if args.len() != want {
                    let example = if want == 2 { "Map[str, int]" } else { "Set[str]" };
                    self.error_help(
                        "gram.generic.1",
                        t.pos,
                        format!("`{name}` takes {}, but {} given", count(want, "type argument"), was_were(args.len())),
                        format!("write e.g. `{example}`"),
                    );
                    return Ty::Err;
                }
                let first = self.resolve_type(&args[0]);
                let ty = if want == 2 {
                    let second = self.resolve_type(&args[1]);
                    self.no_unique(&second, args[1].pos, "a map value");
                    Ty::Map(Box::new(first.clone()), Box::new(second))
                } else {
                    Ty::Set(Box::new(first.clone()))
                };
                self.require_key(&first, t.pos);
                return ty;
            }
            n if conc::TYPE_NAMES.contains(&n) => return self.conc_type(n, args, t.pos),
            n if SPEC_TYPES_NOT_YET.contains(&n) => {
                self.unsupported(t.pos, format!("type `{n}` is not supported by the bootstrap yet (spec 04-types)"));
                return Ty::Err;
            }
            n if self.traits.contains_key(n) => {
                self.error_help(
                    "trait.any.2",
                    t.pos,
                    format!("`{n}` is a trait, not a type"),
                    format!(
                        "for a value of any type implementing it, write `any {n}`; or use a type parameter `[T {n}]`"
                    ),
                );
                return Ty::Err;
            }
            n => match self.types.get(n).copied() {
                Some(def) => return self.generic_type(def, args, t.pos),
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

    /// `Name[args]` for a declared type, checking the count and the bounds of
    /// its arguments (gen.decl.1, gen.bound.3).
    fn generic_type(&mut self, def: Def, args: &[TypeRef], pos: Pos) -> Ty {
        let params = self.def_params(def).to_vec();
        let name = self.def_name(def);
        if args.len() != params.len() {
            if params.is_empty() {
                self.error("gram.generic.1", pos, format!("`{name}` takes no type arguments"));
            } else {
                let names: Vec<&str> = params.iter().map(|&p| self.tparams[p].name.as_str()).collect();
                self.error_help(
                    "gen.decl.1",
                    pos,
                    format!(
                        "`{name}` takes {}, but {} given",
                        count(params.len(), "type argument"),
                        was_were(args.len())
                    ),
                    format!("write `{name}[{}]` with types in place of the parameters", names.join(", ")),
                );
            }
            return Ty::Err;
        }
        let arg_pos: Vec<Pos> = args.iter().map(|a| a.pos).collect();
        let args: Vec<Ty> = args.iter().map(|a| self.resolve_type(a)).collect();
        for (a, at) in args.iter().zip(arg_pos) {
            self.no_unique(a, at, "a type argument");
        }
        for (&p, a) in params.iter().zip(&args) {
            for b in self.tparams[p].bounds.clone() {
                let what = format!("`{name}`'s parameter `{}`", self.tparams[p].name);
                self.require(a, b, pos, what);
            }
        }
        Self::def_ty(def, args)
    }

    /// Report if `t` doesn't implement trait `tr`, which `what` requires
    /// (gen.bound.3). While declarations are being collected, the check waits.
    fn require(&mut self, t: &Ty, tr: usize, pos: Pos, what: String) {
        if let Some(d) = &mut self.deferred {
            d.push((t.clone(), tr, pos, what));
            return;
        }
        if self.implements(t, tr) {
            return;
        }
        let trait_name = &self.prog.traits[tr].name;
        let msg = format!("{what} must implement `{trait_name}`, but {} doesn't", self.show(t));
        let help = self.how_to_implement(t, tr);
        match help {
            Some(h) => self.error_help("gen.bound.3", pos, msg, h),
            None => self.error("gen.bound.3", pos, msg),
        }
    }

    fn check_deferred_bounds(&mut self) {
        for (t, tr, pos, what) in self.deferred.take().unwrap_or_default() {
            self.require(&t, tr, pos, what);
        }
    }

    /// Whether a type was declared with `error` (err.decl.1).
    fn is_error(&self, t: &Ty) -> bool {
        match t {
            Ty::Struct(i, _) => self.prog.structs[*i].is_error,
            Ty::Enum(i, _) => self.prog.enums[*i].is_error,
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
    /// including optional wrapping (type.opt.5), conversion to `any`
    /// (trait.any.3), failure values (err.decl.3), and leaving blocks
    /// (expr.block.3). An inferred error set grows to admit the errors of
    /// `got` (err.type.2).
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
            (Ty::Any(a), Ty::Any(b)) => a == b,
            (Ty::Any(a), g) => !matches!(g, Ty::Unit | Ty::Res(..)) && self.implements(g, *a),
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
            (Ty::Opt(w), Ty::Opt(_)) if matches!(**w, Ty::Opt(_)) => self.error_help(
                rule,
                pos,
                msg,
                "an optional doesn't convert to a nested optional, since its `none` would be ambiguous (type.opt.7)",
            ),
            (w, Ty::Opt(g)) if same(w, g) => self.error_help(
                rule,
                pos,
                msg,
                "this value may be absent; supply a fallback with `x := value else fallback`",
            ),
            // expr.name.2: conversions other languages make implicitly.
            (Ty::Int, Ty::Float) => self.error_help(
                rule,
                pos,
                msg,
                "no implicit conversion: `x.to_int()` truncates toward zero; `x.round().to_int()` rounds",
            ),
            (Ty::Float, Ty::Int) => self.error_help(rule, pos, msg, "no implicit conversion: write `n.to_float()`"),
            (Ty::Num(k), Ty::Int) | (Ty::Num(k), Ty::Float) => {
                let help = format!("no implicit conversion: write `x.to_{}()`", k.name());
                self.error_help(rule, pos, msg, help)
            }
            (Ty::Int | Ty::Float, Ty::Num(_)) => {
                let name = if *want == Ty::Int { "to_int" } else { "to_float" };
                self.error_help(rule, pos, msg, format!("no implicit conversion: write `x.{name}()`"))
            }
            (Ty::Str, Ty::Char) => self.error_help(rule, pos, msg, "make a one-character string with \"{c}\""),
            (Ty::Char, Ty::Str) => self.error_help(rule, pos, msg, "a character is written in single quotes: 'a'"),
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

    /// fix.edit.3: offer `?` after a statement-level fallible call when this
    /// function's error type already admits its errors. Adds the fix to the
    /// `err.use.1` diagnostic just reported.
    fn try_fix(&mut self, t: &Ty, e: &Expr) {
        let Ty::Res(_, errs) = t else { return };
        let errors = self.errs_of(errs);
        let admits = match &self.ret {
            Ty::Res(_, Errs::Inferred(_)) => true,
            Ty::Res(_, Errs::Set(want)) => errors.iter().all(|g| want.contains(g)),
            _ => false,
        };
        let Some(d) = self.diags.last_mut().filter(|d| d.rule == "err.use.1") else { return };
        if admits && matches!(e, Expr::Call { .. } | Expr::Method { .. }) {
            d.fixes.push(Fix {
                title: "pass the error on with `?`".into(),
                edits: vec![Edit::at_line_end(e.pos(), "?")],
            });
        }
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
            d = d.did_you_mean(name, s);
        } else if name.len() == 1 && name.chars().all(|c| c.is_ascii_uppercase()) {
            d = d.help(format!("to make it a type parameter, declare it after the name: `fn name[{name}](...)`"));
        }
        self.diags.push(d);
    }

    fn ty_name(&self, t: &Ty) -> String {
        let with_args = |name: &str, args: &[Ty]| {
            if args.is_empty() {
                name.to_string()
            } else {
                format!("{name}[{}]", args.iter().map(|a| self.ty_name(a)).collect::<Vec<_>>().join(", "))
            }
        };
        match t {
            Ty::Int => "int".into(),
            Ty::Float => "float".into(),
            Ty::Num(n) => n.name().into(),
            Ty::Str => "str".into(),
            Ty::Char => "char".into(),
            Ty::Bool => "bool".into(),
            Ty::Unit | Ty::Never => "no value".into(),
            Ty::Err => "?".into(),
            Ty::Struct(i, args) => with_args(&self.prog.structs[*i].name, args),
            Ty::Enum(i, args) => with_args(&self.prog.enums[*i].name, args),
            Ty::List(e) => format!("List[{}]", self.ty_name(e)),
            Ty::Map(k, v) => format!("Map[{}, {}]", self.ty_name(k), self.ty_name(v)),
            Ty::Set(e) => format!("Set[{}]", self.ty_name(e)),
            Ty::Opt(t) => format!("{}?", self.ty_name(t)),
            Ty::Res(ok, errs) => {
                let set: Vec<String> = self.errs_of(errs).iter().map(|t| self.ty_name(t)).collect();
                let ok = if **ok == Ty::Unit { String::new() } else { format!("{} ", self.ty_name(ok)) };
                if set.is_empty() { format!("!{}", ok.trim_end()) } else { format!("{ok}! {}", set.join(" | ")) }
            }
            Ty::Union(m) => m.iter().map(|t| self.ty_name(t)).collect::<Vec<_>>().join(" | "),
            Ty::Param(p) => self.tparams[*p].name.clone(),
            Ty::Any(tr) => format!("any {}", self.prog.traits[*tr].name),
            Ty::Ref(t) => format!("ref {}", self.ty_name(t)),
            Ty::Conc(conc::Conc::Group, _) => "TaskGroup".into(),
            Ty::Conc(k, t) => format!("{}[{}]", k.name(), self.ty_name(t)),
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
            Ty::Struct(i, _) => Some(self.prog.structs[*i].name.as_str()),
            Ty::Enum(i, _) => Some(self.prog.enums[*i].name.as_str()),
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
        let Some(body) = &f.body else { return };
        let sig = self.sigs[idx].clone();
        self.scope = self.fn_scopes[idx].clone();
        self.self_ty = self.fn_self[idx].clone();
        self.locals.clear();
        self.pending.clear();
        self.anchored = 0;
        self.anchors.clear();
        self.loops = 0;
        self.catching = 0;
        self.ret = sig.ret.clone();
        if f.receiver.is_some() {
            let ty = self.self_ty.clone().unwrap_or(Ty::Err);
            let mutable = f.receiver.is_some_and(|r| r.mutable);
            self.locals.push(Local { name: "self".into(), ty, mutable, kind: Kind::SelfParam, decl: None });
        }
        for (p, (_, ty)) in f.params.iter().zip(&sig.params) {
            if self.declare(&p.name, p.pos) {
                let kind = if p.moved { Kind::MoveParam } else { Kind::Param };
                self.locals.push(Local { name: p.name.clone(), ty: ty.clone(), mutable: p.mutable, kind, decl: None });
            }
        }
        self.check_contracts(f, &sig.ret);
        let need = !Self::returns_nothing(&sig.ret);
        let got = self.block(body, need.then_some(&sig.ret), need);
        if need && !self.accepts(&sig.ret, &got) {
            let ret_pos = f.ret.as_ref().map_or(f.pos, |t| t.pos);
            let (at, msg) = match body.last() {
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
        self.record_locals(0);
        self.scope.clear();
        self.self_ty = None;
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
        self.anchor_locals(mark, stmts);
        let mut ty = Ty::Unit;
        let mut leaves = false;
        for (i, s) in stmts.iter().enumerate() {
            let last = i + 1 == stmts.len();
            ty = self.stmt(s, if last { expect } else { None }, last && need);
            leaves |= ty == Ty::Never;
            // own.uniq.2: a block's value leaves it.
            if last
                && need
                && let Stmt::Expr(e) = s
            {
                self.consume(e, &ty);
            }
            // err.use.1: a failure can't be discarded.
            if (!last || !need) && matches!(s, Stmt::Expr(_)) && self.reject_unhandled(&ty, s.pos()) {
                if let Stmt::Expr(e) = s {
                    self.try_fix(&ty, e);
                }
                ty = Ty::Err;
            }
        }
        self.end_scope(mark);
        if leaves { Ty::Never } else { ty }
    }

    fn stmt(&mut self, s: &'p Stmt, expect: Option<&Ty>, need: bool) -> Ty {
        match s {
            Stmt::Let { name, pos, mutable, value, otherwise } => {
                // type.coll.13: `x := []` or `{}` is typed by its first use.
                let empty = match value {
                    Expr::List(xs, _) | Expr::Set(xs, _) => xs.is_empty(),
                    Expr::Map(xs, _) => xs.is_empty(),
                    _ => false,
                };
                if empty && otherwise.is_none() {
                    if self.declare(name, *pos) {
                        let brace = !matches!(value, Expr::List(..));
                        let index = self.locals.len();
                        self.locals.push(Local {
                            name: name.clone(),
                            ty: Ty::Err,
                            mutable: *mutable,
                            kind: Kind::Binding,
                            decl: Some(*pos),
                        });
                        self.pending.push(Pending { name: name.clone(), lit: value.pos(), brace, index });
                    }
                    return Ty::Unit;
                }
                let vt = if otherwise.is_some() { self.else_value(value, otherwise.as_ref()) } else { self.expr(value, None, true) };
                self.consume(value, &vt);
                let ty = match otherwise {
                    Some(o) => self.else_part(&vt, value, o),
                    None if self.reject_unhandled(&vt, value.pos()) => {
                        self.try_fix(&vt, value);
                        Self::ok_type(&vt)
                    }
                    None => vt,
                };
                if ty == Ty::Unit {
                    let msg = format!("`{name}` would be bound to an expression that produces no value");
                    match value {
                        Expr::Method { name: m, .. } if m == "sort" => self.error_help(
                            "expr.bind.1",
                            value.pos(),
                            msg,
                            "`sort()` changes the list in place; `xs.sorted()` gives a new sorted list",
                        ),
                        _ => self.error("expr.bind.1", value.pos(), msg),
                    }
                }
                if self.declare(name, *pos) {
                    self.locals.push(Local { name: name.clone(), ty, mutable: *mutable, kind: Kind::Binding, decl: Some(*pos) });
                } else if *mutable
                    && let Some(d) = self.diags.last_mut()
                    && let Some(f) = d.fixes.first_mut()
                {
                    // `mut x := e` again becomes `x = e`, so the `mut` goes too.
                    f.edits.insert(0, Edit::replace(Pos { line: pos.line, col: 1 }, "mut ", ""));
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
                // expr.loop.7: `while true` with no `break` of its own never finishes.
                if matches!(cond, Expr::Bool(true, _)) && !breaks(body) { Ty::Never } else { Ty::Unit }
            }
            Stmt::Handle { value, otherwise } => {
                self.handle(value, otherwise);
                Ty::Unit
            }
            Stmt::For { var, var_pos, mutable, iter, body, .. } => {
                let elem = if *mutable { self.mut_iterable(iter, var, body) } else { self.iterable(iter) };
                let mark = self.locals.len();
                if self.declare(var, *var_pos) {
                    self.locals.push(Local { name: var.clone(), ty: elem, mutable: *mutable, kind: Kind::LoopVar, decl: None });
                }
                self.loops += 1;
                self.block(body, None, false);
                self.loops -= 1;
                self.end_scope(mark);
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
            Stmt::Defer(inner, pos) => {
                self.defer_stmt(inner, *pos);
                Ty::Unit
            }
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
                self.consume(v, &t);
                if !self.accepts(&ret, &t) {
                    self.mismatch("expr.block.2", v.pos(), &ret, &t);
                }
            }
        }
    }

    /// The value of `e else ...`. An optional `?` inside it goes to the
    /// `else` (err.prop.4), which makes the value optional if it wasn't.
    fn else_value(&mut self, value: &'p Expr, o: Option<&Else>) -> Ty {
        let before = self.caught;
        self.catching += 1;
        let vt = self.expr(value, None, true);
        self.catching -= 1;
        if self.caught > before
            && matches!(vt, Ty::Res(..))
            && let Some(Else::Block { err: Some((name, pos)), .. }) = o
        {
            self.error_help(
                "err.prop.4",
                *pos,
                format!("`{name}` would have no error when a `?` inside the value finds `none`"),
                "handle the optional separately: `x := opt else ...`, then the result",
            );
        }
        match vt {
            Ty::Opt(_) | Ty::Res(..) | Ty::Err => vt,
            t if self.caught > before => Ty::Opt(Box::new(t)),
            t => t,
        }
    }

    /// `f(x) else ...` for a result with no success value (err.else.4).
    fn handle(&mut self, value: &'p Expr, otherwise: &'p Else) {
        let vt = self.expr(value, None, true);
        match &vt {
            Ty::Res(ok, _) if **ok == Ty::Unit => {
                self.else_part(&vt, value, otherwise);
            }
            Ty::Err => {}
            Ty::Res(..) | Ty::Opt(_) => {
                let what = if matches!(vt, Ty::Opt(_)) { "may be absent" } else { "can fail" };
                self.error_help(
                    "err.else.4",
                    value.pos(),
                    format!(
                        "this {what}, and has a value ({}); `else` on its own handles only results with no value",
                        self.show(&vt)
                    ),
                    "bind the value: `x := ... else ...`",
                );
            }
            _ => {
                let msg = format!("`else` handles a failure, but this is {} and can't fail", self.show(&vt));
                self.error_help("err.else.4", value.pos(), msg, "remove the `else`");
            }
        }
    }

    /// The `else` part of `x := e else ...`; returns the type bound to `x`
    /// (err.else.1-3).
    fn else_part(&mut self, vt: &Ty, value: &Expr, o: &'p Else) -> Ty {
        let pos = value.pos();
        let (ok, errors) = match vt {
            Ty::Opt(t) => ((**t).clone(), None),
            Ty::Res(t, e) => ((**t).clone(), Some(self.errs_of(e))),
            Ty::Err => (Ty::Err, None),
            other => {
                let msg = format!(
                    "`else` handles a value that can be absent (`T?`) or fail (`T ! E`), but this is {}",
                    self.show(other)
                );
                match Self::never_absent_help(value) {
                    Some(h) => self.error_help("err.else.1", pos, msg, h),
                    None => self.error("err.else.1", pos, msg),
                }
                (other.clone(), None)
            }
        };
        match o {
            Else::Value(d) if ok == Ty::Unit => {
                self.expr(d, None, true);
                self.error_help(
                    "err.else.4",
                    d.pos(),
                    "this result has no value, so there's nothing to fall back to",
                    "write `else return`, or `else err` with a block that handles the error",
                );
            }
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
                                    decl: None,
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
                // err.else.4: with no value to supply, the block may just end.
                let unit = ok == Ty::Unit;
                let t = if unit { self.block(body, None, false) } else { self.block(body, Some(&ok), true) };
                self.end_scope(mark);
                if !unit && !self.accepts(&ok, &t) {
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
        if self.record {
            self.bound.insert(self.locals.len(), pos);
        }
        if let Some(l) = self.locals.iter().rev().find(|l| l.name == name) {
            let mut d = Diagnostic::new("expr.bind.4", pos, format!("`{name}` is already declared in this function"));
            if let Some(first) = l.decl {
                d = d.note(first, format!("`{name}` is first declared here"));
            }
            d = if l.mutable {
                d.help(format!("`{name}` is a `mut` variable; to change it, write `{name} = ...`"))
                    .fix(format!("assign to `{name}` with `=`"), vec![Edit::replace(pos, ":=", "=")])
            } else {
                d.help("choose a different name; names can't be shadowed")
            };
            self.diags.push(d);
            return false;
        }
        let what = if self.fns.contains_key(name) || PRELUDE.contains(&name) {
            "a function"
        } else if self.types.contains_key(name) || BUILTIN_TYPES.contains(&name) {
            "a type"
        } else if self.consts.contains_key(name) {
            "a constant"
        } else if crate::stdlib::imported(self.prog, name).is_some() {
            // Only an explicit `use std.NAME`: a standard module in scope
            // without one is hidden by the local instead (std.use.4).
            "an imported module"
        } else {
            return true;
        };
        let mut d = Diagnostic::new("expr.bind.5", pos, format!("`{name}` is already {what} name"))
            .help("choose a different name for the local");
        let first = (self.fns.get(name).map(|j| self.prog.fns[*j].pos))
            .or_else(|| self.types.get(name).map(|d| self.def_pos(*d)))
            .or_else(|| self.consts.get(name).map(|j| self.prog.consts[*j].pos));
        if let Some(first) = first.filter(|p| !is_prelude(*p)) {
            d = d.note(first, format!("`{name}` is declared here"));
        }
        self.diags.push(d);
        false
    }

    /// The pending entry of local `n`, if its type isn't known yet (type.coll.13).
    fn pending_of(&self, n: &str) -> Option<usize> {
        self.pending.iter().position(|p| p.name == n && self.locals.get(p.index).is_some_and(|l| l.name == n))
    }

    /// Give pending local `k` its type, now known.
    fn settle(&mut self, k: usize, ty: Ty) {
        let p = self.pending.remove(k);
        if matches!(ty, Ty::Set(_)) {
            self.resolved.empty_sets.insert(p.lit);
        }
        self.locals[p.index].ty = ty;
    }

    /// Report a use of pending local `k` before its type is known.
    fn unsettled(&mut self, k: usize, at: Pos) {
        let p = self.pending.remove(k);
        let ctor = if p.brace { "`Map[K, V]()` or `Set[T]()`" } else { "`List[T]()`" };
        let first =
            if p.brace { format!("`{0}.add(x)` or `{0}[k] = v`", p.name) } else { format!("`{}.push(x)`", p.name) };
        self.error_help(
            "type.coll.13",
            at,
            format!("the type of `{}` isn't known yet", p.name),
            format!("its first use should fix it, such as {first}; or declare it with {ctor}"),
        );
        self.locals[p.index].ty = Ty::Err;
    }

    /// The type of `e`, without reporting anything: to fix a pending local's
    /// type before `e` is checked for real.
    fn quiet_type(&mut self, e: &'p Expr) -> Ty {
        let n = self.diags.len();
        let t = self.expr(e, None, true);
        self.diags.truncate(n);
        t
    }

    /// Leave a block: its locals go out of scope, and one never typed is an
    /// error (type.coll.13).
    fn end_scope(&mut self, mark: usize) {
        while let Some(k) = self.pending.iter().position(|p| p.index >= mark) {
            let p = self.pending.remove(k);
            let ctor = if p.brace { "`Map[K, V]()` or `Set[T]()`" } else { "`List[T]()`" };
            self.error_help(
                "type.coll.13",
                p.lit,
                format!("nothing fixes the type of `{}`", p.name),
                format!("declare it with {ctor}"),
            );
        }
        self.record_locals(mark);
        self.locals.truncate(mark);
    }

    /// Record the locals from `mark` up, which are about to go out of scope
    /// (api.loc.1).
    fn record_locals(&mut self, mark: usize) {
        if self.record {
            for i in mark..self.locals.len() {
                let l = &self.locals[i];
                let Some(anchor) = l.decl.or_else(|| self.anchors.get(&i).copied()) else { continue };
                let kind = match l.kind {
                    Kind::Param => "param",
                    Kind::MoveParam => "move param",
                    Kind::SelfParam => "self",
                    Kind::LoopVar => "loop",
                    Kind::Binding => "binding",
                };
                let ty = if matches!(l.ty, Ty::Err) { "?".to_string() } else { self.ty_name(&l.ty) };
                let fact = facts::LocalFact { name: l.name.clone(), ty, mutable: l.mutable, kind, at: self.bound.get(&i).copied(), decl: l.decl, anchor };
                self.resolved.facts.locals.push(fact);
            }
        }
        self.anchored = self.anchored.min(mark);
        self.anchors.retain(|i, _| *i < mark);
        self.bound.retain(|i, _| *i < mark);
    }

    /// Locals below `mark` with no declaration of their own (parameters, loop
    /// variables, pattern names) are in scope from the first statement of the
    /// block that starts here (api.loc.2).
    fn anchor_locals(&mut self, mark: usize, stmts: &[crate::ast::Stmt]) {
        if self.record {
            if let Some(first) = stmts.first() {
                for i in self.anchored..mark {
                    self.anchors.entry(i).or_insert(first.pos());
                }
            }
            self.anchored = self.anchored.max(mark);
        }
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
        if let Expr::Range { lo, hi, pos, .. } = iter {
            self.loop_range(lo.as_deref(), hi.as_deref(), *pos);
            return Ty::Int;
        }
        match self.expr(iter, None, true) {
            Ty::List(t) | Ty::Set(t) => *t,
            Ty::Err => Ty::Err,
            t if self.reject_unhandled(&t, iter.pos()) => Ty::Err,
            Ty::Map(..) => {
                self.error_help(
                    "type.map.5",
                    iter.pos(),
                    "`for` iterates a map's keys or values explicitly",
                    "write `for k in m.keys()`, or `m.values()`",
                );
                Ty::Err
            }
            Ty::Str => {
                self.error_help("expr.loop.4", iter.pos(), "`for` can't iterate over a `str`", "iterate `s.chars()`");
                Ty::Err
            }
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

    /// The bounds of a `for` range, which needs both (expr.loop.3).
    fn loop_range(&mut self, lo: Option<&'p Expr>, hi: Option<&'p Expr>, pos: Pos) {
        if lo.is_none() || hi.is_none() {
            self.error_help(
                "expr.loop.3",
                pos,
                "a `for` range needs both bounds",
                "write `a..b`; open ranges are only for slices, `xs[a..]`",
            );
        }
        self.range_bounds(lo, hi);
    }

    fn range_bounds(&mut self, lo: Option<&'p Expr>, hi: Option<&'p Expr>) {
        for bound in [lo, hi].into_iter().flatten() {
            let t = self.expr(bound, Some(&Ty::Int), true);
            if !self.accepts(&Ty::Int, &t) {
                let msg = format!("range bounds must be `int`, found {}", self.show(&t));
                self.error("expr.loop.3", bound.pos(), msg);
            }
        }
    }

    // ---- Assignment and places ----

    fn assign(&mut self, target: &'p Expr, op: Option<BinOp>, value: &'p Expr, pos: Pos) {
        // type.coll.13: `m[k] = v`, `m[k] += n` or `x = e` can fix a pending
        // local's type; `op=` starts a missing key at zero (ADR 0251, ADR 0280).
        if let Expr::Index { recv, index, .. } = target
            && let Expr::Name(n, _) = &**recv
            && let Some(k) = self.pending_of(n)
            && self.pending[k].brace
        {
            let (kt, vt) = (self.quiet_type(index), self.quiet_type(value));
            if op.is_none() || matches!(vt, Ty::Int | Ty::Float) {
                self.settle(k, Ty::Map(Box::new(kt), Box::new(vt)));
            }
        }
        if op.is_none() {
            match target {
                Expr::Name(n, _) => {
                    if let Some(k) = self.pending_of(n) {
                        let vt = self.quiet_type(value);
                        match (&vt, self.pending[k].brace) {
                            (Ty::List(_), false) | (Ty::Map(..) | Ty::Set(_), true) => self.settle(k, vt),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        if let Expr::Name(n, npos) = target
            && self.local(n).is_none()
        {
            self.expr(value, None, true);
            if n == "self" {
                self.expr(target, None, true);
            } else {
                let mut d = Diagnostic::new("expr.bind.3", *npos, format!("`{n}` isn't declared"))
                    .help(format!("declare it with `mut {n} := ...`"));
                if op.is_none() {
                    let edits = vec![
                        Edit::insert(*npos, "mut "),
                        Edit::replace(pos, "=", ":="),
                    ];
                    d = d.fix(format!("declare `{n}` with `mut {n} :=`"), edits);
                }
                self.diags.push(d);
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
        self.consume(value, &vt);
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
        // own.ref.4: a `ref` cell's contents are mutable through any copy.
        if self.writes_through_ref(e, rule == Some("trait.method.3")) {
            self.resolved.ref_writes.insert(e.pos());
            return;
        }
        let Some((root, _)) = place_path(e) else {
            let rule = rule.unwrap_or("expr.bind.6");
            self.error(rule, e.pos(), format!("only a variable, or a field or element of one, can be {action}"));
            return;
        };
        let Some(l) = self.local(root) else { return };
        if l.mutable {
            return;
        }
        let decl = l.decl;
        let (default_rule, msg, help) = match l.kind {
            Kind::Binding => {
                ("expr.bind.3", format!("`{root}` is immutable"), format!("declare it with `mut {root} := ...`"))
            }
            Kind::Param | Kind::MoveParam => (
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
        let mut d = Diagnostic::new(rule.unwrap_or(default_rule), e.pos(), format!("{msg}, so it can't be {action}")).help(help);
        // fix.edit.1: a `:=` binding that is changed later becomes `mut`.
        if let Some(decl) = decl {
            d = d.fix(format!("declare `{root}` with `mut`"), vec![Edit::insert(decl, "mut ")]);
        }
        self.diags.push(d);
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
            Stmt::Handle { value, otherwise } => e(value).or_else(|| match otherwise {
                Else::Value(d) => e(d),
                Else::Leave(st) => self.overlap_in_stmt(target, st),
                Else::Block { body, .. } => self.overlapping_use(target, body),
            }),
            Stmt::Return(None, _) | Stmt::Break(_) | Stmt::Continue(_) => None,
            Stmt::Defer(st, _) => self.overlap_in_stmt(target, st),
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
            Expr::Str(..)
            | Expr::Int(..)
            | Expr::Float(..)
            | Expr::Char(..)
            | Expr::Bool(..)
            | Expr::None(_)
            | Expr::Hole(_)
            | Expr::Name(..)
            | Expr::TypeArgs { .. } => None,
            Expr::Interp(parts, _) => parts.iter().find_map(|p| match p {
                InterpPart::Expr(y) | InterpPart::Fmt(y, _) => e(y),
                InterpPart::Lit(_) => None,
            }),
            Expr::List(items, _) => items.iter().find_map(e),
            Expr::Call { callee, args, .. } => e(callee).or_else(|| args.iter().find_map(|a| e(&a.value))),
            Expr::Method { recv, args, .. } => e(recv).or_else(|| args.iter().find_map(|a| e(&a.value))),
            Expr::Field { recv, .. } => e(recv),
            Expr::Index { recv, index, .. } => e(recv).or_else(|| e(index)),
            Expr::Unary { operand, .. } | Expr::Ref { value: operand, .. } => e(operand),
            Expr::Binary { lhs, rhs, .. } => e(lhs).or_else(|| e(rhs)),
            Expr::Range { lo, hi, .. } => lo.as_deref().and_then(e).or_else(|| hi.as_deref().and_then(e)),
            Expr::Map(pairs, _) => pairs.iter().find_map(|(k, v)| e(k).or_else(|| e(v))),
            Expr::Set(items, _) => items.iter().find_map(e),
            Expr::Comp { elem, value, clauses, .. } => clauses
                .iter()
                .find_map(|c| e(&c.iter).or_else(|| c.cond.as_ref().and_then(e)))
                .or_else(|| e(elem))
                .or_else(|| value.as_deref().and_then(e)),
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
            Expr::Else { value, otherwise, .. } => e(value).or_else(|| match &**otherwise {
                Else::Value(d) => e(d),
                _ => None,
            }),
        }
    }

    // ---- Expressions ----

    /// The type of `e`. `expect` is the type the context wants, if known; it
    /// resolves bare variants, empty lists and type arguments. `need` says
    /// whether the value is used (expr.if.4).
    fn expr(&mut self, e: &'p Expr, expect: Option<&Ty>, need: bool) -> Ty {
        let t = self.expr_of(e, expect, need);
        if self.record {
            let text = if matches!(t, Ty::Err) { "?".to_string() } else { self.ty_name(&t) };
            if let Expr::Method { name_pos, .. } | Expr::Field { name_pos, .. } = e {
                self.resolved.facts.exprs.push((*name_pos, text.clone()));
            }
            self.resolved.facts.exprs.push((e.pos(), text));
        }
        t
    }

    fn expr_of(&mut self, e: &'p Expr, expect: Option<&Ty>, need: bool) -> Ty {
        match e {
            Expr::Str(..) => Ty::Str,
            // type.num.10: an integer literal where a float is expected is a float.
            Expr::Int(_, pos) if matches!(expect, Some(Ty::Float)) => {
                self.resolved.float_literals.insert(*pos);
                Ty::Float
            }
            // type.num.13: a literal is typed by the sized type the context expects.
            Expr::Int(v, pos) if matches!(expect, Some(Ty::Num(_))) => {
                let Some(Ty::Num(k)) = expect else { unreachable!() };
                self.sized_literal(*v, false, *pos, *k);
                Ty::Num(*k)
            }
            Expr::Float(_, pos) if matches!(expect, Some(Ty::Num(Num::F32))) => {
                self.resolved.sized_literals.insert(*pos, Num::F32);
                Ty::Num(Num::F32)
            }
            Expr::Int(..) => Ty::Int,
            Expr::Float(..) => Ty::Float,
            Expr::Char(..) => Ty::Char,
            Expr::Bool(..) => Ty::Bool,
            Expr::None(pos) => self.none(*pos, expect),
            Expr::Hole(pos) => self.hole(*pos, expect),
            Expr::Try { expr, context, pos } => self.try_expr(expr, context.as_deref(), *pos),
            // err.else.6: the value, or the fallback when it's absent or failed.
            Expr::Else { value, otherwise, .. } => {
                let vt = self.else_value(value, Some(otherwise));
                self.else_part(&vt, value, otherwise)
            }
            Expr::Interp(parts, _) => {
                for part in parts {
                    match part {
                        InterpPart::Expr(x) => {
                            let t = self.expr(x, None, true);
                            self.text_form(&t, x.pos(), "lex.str.2");
                        }
                        InterpPart::Fmt(x, spec) => {
                            let t = self.expr(x, None, true);
                            self.text_form(&t, x.pos(), "lex.str.2");
                            self.format_spec(&t, spec, x.pos());
                        }
                        InterpPart::Lit(_) => {}
                    }
                }
                Ty::Str
            }
            Expr::Name(n, pos) => self.name(n, *pos, expect),
            Expr::TypeArgs { name, pos, .. } => {
                let what = if self.types.contains_key(name.as_str()) { "a type" } else { "a function" };
                self.error_help(
                    "expr.name.1",
                    *pos,
                    format!("`{name}[...]` is {what}, not a value"),
                    "call it, e.g. `f[int](x)`, or call an associated function, e.g. `Stack[int].new()`",
                );
                Ty::Err
            }
            Expr::List(elems, pos) => {
                let t = self.list(elems, *pos, expect);
                if let Ty::List(e) = &t {
                    self.no_unique(e, *pos, "a list element");
                }
                t
            }
            Expr::Call { callee, args, .. } => self.call(callee, args, expect),
            Expr::Method { recv, name, name_pos, args } => self.method(recv, name, *name_pos, args, expect),
            Expr::Field { recv, name, name_pos } => self.field(recv, name, *name_pos, expect),
            Expr::Index { recv, index, .. } => self.index(recv, index),
            Expr::Unary { op: UnOp::Neg, operand, .. }
                if matches!(expect, Some(Ty::Float)) && matches!(**operand, Expr::Int(..)) =>
            {
                self.float_literal(operand);
                Ty::Float
            }
            Expr::Unary { op: UnOp::Neg, operand, .. }
                if matches!(expect, Some(Ty::Num(_))) && matches!(**operand, Expr::Int(..) | Expr::Float(..)) =>
            {
                let Some(Ty::Num(k)) = expect else { unreachable!() };
                match **operand {
                    Expr::Int(v, p) => self.sized_literal(v, true, p, *k),
                    Expr::Float(_, p) if *k == Num::F32 => {
                        self.resolved.sized_literals.insert(p, Num::F32);
                    }
                    _ => {}
                }
                Ty::Num(*k)
            }
            Expr::Unary { op, operand, pos } => self.unary(*op, operand, *pos),
            Expr::Binary { op, lhs, rhs, pos } => self.binary(*op, lhs, rhs, *pos),
            Expr::Map(pairs, pos) => self.map_literal(pairs, *pos, expect),
            Expr::Set(items, pos) => self.set_literal(items, *pos, expect),
            Expr::Comp { kind, elem, value, clauses, .. } => {
                self.comprehension(*kind, elem, value.as_deref(), clauses, expect)
            }
            Expr::Range { lo, hi, pos, .. } => {
                self.range_bounds(lo.as_deref(), hi.as_deref());
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
            Expr::Ref { value, pos } => self.ref_expr(value, *pos, expect),
        }
    }

    /// Every value has a text form (trait.auto.2); "no value" and an
    /// unhandled failure don't.
    /// `{x:spec}` (lex.str.7, type.str.11): `.N` needs a float, `x` an int.
    fn format_spec(&mut self, t: &Ty, spec: &crate::lexer::FormatSpec, pos: Pos) {
        if spec.precision.is_some() && !matches!(t, Ty::Float | Ty::Num(Num::F32) | Ty::Err) {
            self.error_help(
                "type.str.11",
                pos,
                "`.N` in a format specification needs a `float`",
                "round a float with `{x:.2}`; to cut an int or text, convert it first",
            );
        }
        if spec.hex && !matches!(t, Ty::Int | Ty::Err) && !matches!(t, Ty::Num(k) if !k.is_float()) {
            self.error_help(
                "type.str.11",
                pos,
                "`x` in a format specification needs an `int`",
                "drop the `x`, or convert the value to an `int` first",
            );
        }
    }

    fn text_form(&mut self, t: &Ty, pos: Pos, no_value_rule: &'static str) {
        match t {
            Ty::Unit => self.error(no_value_rule, pos, "this expression produces no value"),
            Ty::Res(..) => {
                self.reject_unhandled(t, pos);
            }
            _ => {}
        }
    }

    fn name(&mut self, n: &str, pos: Pos, expect: Option<&Ty>) -> Ty {
        if let Some(k) = self.pending_of(n) {
            let want = match expect {
                Some(Ty::Opt(t)) => Some(&**t),
                e => e,
            };
            return match want {
                Some(t @ Ty::List(_)) if !self.pending[k].brace => {
                    let t = t.clone();
                    self.settle(k, t.clone());
                    t
                }
                Some(t @ (Ty::Map(..) | Ty::Set(_))) if self.pending[k].brace => {
                    let t = t.clone();
                    self.settle(k, t.clone());
                    t
                }
                _ => {
                    self.unsettled(k, pos);
                    Ty::Err
                }
            };
        }
        if let Some(l) = self.local(n) {
            return l.ty.clone();
        }
        if let Some(&i) = self.consts.get(n) {
            self.resolved.symbols.insert(pos, format!("const:{n}"));
            return self.const_type(i);
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
            return self.enum_value(e, None, expect, pos, n);
        }
        if !self.variant_owners(n).is_empty() {
            return Ty::Err;
        }
        if self.module_as_value(n, pos) {
            return Ty::Err;
        }
        if self.fns.contains_key(n) || PRELUDE.contains(&n) {
            self.unsupported(pos, format!("using the function `{n}` as a value is not supported by the bootstrap yet"));
            return Ty::Err;
        }
        if let Some(def) = self.types.get(n).copied() {
            let help = match def {
                Def::Struct(_) => format!("build a value with `{n}(field: value, ...)`"),
                Def::Enum(e) => format!("use one of its variants, e.g. `{n}.{}`", self.prog.enums[e].variants[0].name),
            };
            self.error_help("expr.name.1", pos, format!("`{n}` is a type, not a value"), help);
            return Ty::Err;
        }
        if self.traits.contains_key(n) {
            self.error_help(
                "expr.name.1",
                pos,
                format!("`{n}` is a trait, not a value"),
                format!("call its methods on a value, `x.method()`, or as `{n}.method(x)`"),
            );
            return Ty::Err;
        }
        if BUILTIN_TYPES.contains(&n) || SPEC_TYPES_NOT_YET.contains(&n) {
            self.error("expr.name.1", pos, format!("`{n}` is a type, not a value"));
            return Ty::Err;
        }
        self.unknown_name(n, pos);
        Ty::Err
    }

    /// The type of a variant of enum `e` that carries no data. Its type
    /// arguments are `explicit`, or come from the expected type (gen.infer.1).
    fn enum_value(&mut self, e: usize, explicit: Option<Vec<Ty>>, expect: Option<&Ty>, pos: Pos, variant: &str) -> Ty {
        let n = self.enum_params[e].len();
        if n == 0 {
            return Ty::Enum(e, Vec::new());
        }
        if let Some(args) = explicit.or_else(|| self.expected_args(Def::Enum(e), expect)) {
            return Ty::Enum(e, args);
        }
        let name = &self.prog.enums[e].name;
        self.error_help(
            "gen.infer.1",
            pos,
            format!("the type arguments of `{name}` aren't known here"),
            format!("write them, e.g. `{name}[int].{variant}`"),
        );
        Ty::Enum(e, vec![Ty::Err; n])
    }

    /// The type arguments of `def` in the expected type, if that is (or
    /// wraps) an instance of it.
    fn expected_args(&self, def: Def, expect: Option<&Ty>) -> Option<Vec<Ty>> {
        match (def, expect?) {
            (Def::Struct(s), Ty::Struct(t, args)) | (Def::Enum(s), Ty::Enum(t, args)) if s == *t => Some(args.clone()),
            (_, Ty::Opt(inner)) => self.expected_args(def, Some(inner)),
            (_, Ty::Res(ok, errs)) => self
                .expected_args(def, Some(ok))
                .or_else(|| self.errs_of(errs).iter().find_map(|t| self.expected_args(def, Some(t)))),
            (_, Ty::Union(m)) => m.iter().find_map(|t| self.expected_args(def, Some(t))),
            _ => None,
        }
    }

    /// Bindings of `def`'s type parameters from the expected type, skipping
    /// unknown ones.
    fn expected_binds(&self, def: Def, expect: Option<&Ty>) -> Vec<(usize, Ty)> {
        let params = self.def_params(def);
        match self.expected_args(def, expect) {
            Some(args) => params.iter().zip(args).filter(|(_, a)| !has_err(a)).map(|(p, a)| (*p, a)).collect(),
            None => Vec::new(),
        }
    }

    /// Enums whose variants may be written bare here (type.enum.3): the
    /// expected enum, the enums of an expected `T?` or `T ! E`, and, for a
    /// result whose errors are inferred, every error enum.
    fn expected_enums(&self, expect: Option<&Ty>) -> Vec<usize> {
        let enum_index = |t: &Ty| if let Ty::Enum(e, _) = t { Some(*e) } else { None };
        match expect {
            Some(Ty::Enum(e, _)) => vec![*e],
            Some(Ty::Opt(t)) => self.expected_enums(Some(t)),
            Some(Ty::Union(m)) => m.iter().filter_map(enum_index).collect(),
            Some(Ty::Res(ok, errs)) => {
                let mut v = self.expected_enums(Some(ok));
                match errs {
                    Errs::Set(set) => v.extend(set.iter().filter_map(enum_index)),
                    Errs::Inferred(_) => v.extend((0..self.prog.enums.len()).filter(|e| self.prog.enums[*e].is_error)),
                }
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

    /// `todo`, a typed hole (expr.hole.1): it takes the expected type, or an
    /// unknown one that any use accepts, and warns with what the gap needs.
    fn hole(&mut self, pos: Pos, expect: Option<&Ty>) -> Ty {
        let mut scope: Vec<String> = (self.locals.iter())
            .map(|l| match l.ty {
                Ty::Err => format!("`{}` (type unknown)", l.name),
                _ => format!("`{} {}`", l.name, self.ty_name(&l.ty)),
            })
            .collect();
        scope.sort();
        let scope = if scope.is_empty() { "nothing".to_string() } else { scope.join(", ") };
        let (msg, help, ty) = match expect {
            Some(t) if !matches!(t, Ty::Err) => (
                format!("hole: expected {}; in scope: {scope}", self.show(t)),
                format!("replace `todo` with a value of type {}", self.show(t)),
                t.clone(),
            ),
            _ => (
                format!("hole: the expected type isn't known here; in scope: {scope}"),
                "replace `todo` with an expression".to_string(),
                Ty::Err,
            ),
        };
        self.diags.push(Diagnostic::warning("expr.hole.1", pos, msg).help(help));
        ty
    }

    /// `e?` and `e? "context"` (err.prop.1-3, err.ctx.2).
    fn try_expr(&mut self, inner: &'p Expr, context: Option<&'p Expr>, pos: Pos) -> Ty {
        let t = self.expr(inner, None, true);
        if let Some(c) = context {
            let ct = self.expr(c, Some(&Ty::Str), true);
            // err.prop.5: `opt? "context"` fails with `Absent` and the context.
            let absent = matches!(t, Ty::Opt(_)) && self.catching == 0 && matches!(self.ret, Ty::Res(_, Errs::Inferred(_)));
            if !matches!(t, Ty::Res(..) | Ty::Err) && !absent {
                self.error("err.ctx.2", c.pos(), "only an error can carry context; this value can't fail");
            } else if !self.accepts(&Ty::Str, &ct) {
                self.mismatch("err.ctx.2", c.pos(), &Ty::Str, &ct);
            }
        }
        // ct.const.3: `?` in a constant is reported by `constant_expr`.
        if self.const_busy.iter().any(|&b| b) {
            return match t {
                Ty::Res(ok, _) => *ok,
                Ty::Opt(v) => *v,
                t => t,
            };
        }
        let ret = self.ret.clone();
        match t {
            Ty::Opt(v) => {
                self.resolved.optional_try.insert(pos);
                if self.catching > 0 {
                    // err.prop.4: an enclosing `else` handles the absence.
                    self.resolved.else_try.insert(pos);
                    self.caught += 1;
                } else if !matches!(ret, Ty::Opt(_)) && !self.absent_try(&ret, pos) {
                    self.error_help(
                        "err.prop.2",
                        pos,
                        "`?` on an optional only works in a function that returns an optional or infers its errors, or before an `else`",
                        "add a fallback: `x?.y else d`, or leave: `x := value else return ...`",
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

    fn list(&mut self, elems: &'p [Expr], pos: Pos, expect: Option<&Ty>) -> Ty {
        let hint = match expect {
            Some(Ty::List(t)) => Some((**t).clone()),
            Some(Ty::Opt(inner)) => match &**inner {
                Ty::List(t) => Some((**t).clone()),
                _ => None,
            },
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
        // A known element type: each element must fit it, converting to `any` if needed.
        if let Some(h) = hint.clone().filter(|h| !has_err(h)) {
            for e in elems {
                let t = self.expr(e, Some(&h), true);
                if self.reject_unhandled(&t, e.pos()) {
                    continue;
                }
                if t == Ty::Unit {
                    self.error("type.coll.2", e.pos(), "a list element must be a value");
                } else if !self.accepts(&h, &t) {
                    let msg = format!("the elements of this list are {}, but this is {}", self.show(&h), self.show(&t));
                    self.error("type.coll.2", e.pos(), msg);
                }
            }
            return Ty::List(Box::new(h));
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
        let (n, npos, targs) = match callee {
            Expr::Name(n, p) => (n, *p, None),
            Expr::TypeArgs { name, args, pos } => (name, *pos, Some(args.as_slice())),
            _ => {
                self.expr(callee, None, true);
                self.args_only(args);
                self.unsupported(
                    callee.pos(),
                    "calling the result of an expression is not supported by the bootstrap yet",
                );
                return Ty::Err;
            }
        };
        if targs.is_none() && self.local(n).is_some() {
            self.args_only(args);
            self.error("expr.call.1", npos, format!("`{n}` is a value, not a function"));
            return Ty::Err;
        }
        if n == "print" {
            return self.print_call(args, npos);
        }
        // err.model.2: `unreachable(message)` traps; it never returns.
        if n == "unreachable" {
            let sig = Sig::plain(None, vec![(false, Ty::Str)], Ty::Unit);
            self.check_call(n, &sig, Vec::new(), args, None, npos, None);
            return Ty::Never;
        }
        if n == "assert" || n == "assert_eq" {
            return self.assert_call(n, args, npos);
        }
        if let Some(t) = self.conc_constructor(n, targs, args, npos, expect) {
            return t;
        }
        if let Some(t) = targs.and_then(|a| self.collection_constructor(n, a, npos)) {
            if !args.is_empty() {
                self.args_only(args);
                self.error_help(
                    "type.coll.8",
                    npos,
                    format!("`{n}[...]()` makes an empty collection and takes no arguments"),
                    "write the elements as a literal, e.g. `[1, 2]`",
                );
            }
            return t;
        }
        match self.types.get(n.as_str()).copied() {
            Some(Def::Struct(s)) => {
                self.resolved.symbols.insert(npos, format!("type:{n}"));
                let explicit = targs.map(|a| self.explicit_args(n, &self.struct_params[s].clone(), a, npos));
                return self.struct_literal(s, explicit, args, npos, expect);
            }
            Some(Def::Enum(e)) => {
                self.args_only(args);
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
            self.reject_test_call(i, npos);
            self.note_deprecated(i, npos);
            self.resolved.symbols.insert(npos, format!("fn:{n}"));
            let sig = self.sigs[i].clone();
            let binds = match targs {
                Some(a) => {
                    let declared = sig.generics[sig.owner..sig.owner + sig.declared].to_vec();
                    self.explicit_args(n, &declared, a, npos)
                }
                None => Vec::new(),
            };
            return self.check_call(n, &sig, binds, args, None, npos, expect);
        }
        if let Some((e, v)) = self.bare_variant(n, npos, expect) {
            return self.variant_literal(e, v, None, args, npos, expect);
        }
        self.args_only(args);
        if self.traits.contains_key(n.as_str()) {
            self.error_help(
                "expr.name.1",
                npos,
                format!("`{n}` is a trait, not a function"),
                format!("call a method of it, e.g. `{n}.method(x)`"),
            );
        } else if self.variant_owners(n).is_empty() {
            self.unknown_name(n, npos);
        }
        Ty::Err
    }

    /// `List[T]()`: an empty collection of a given type (type.coll.8).
    fn collection_constructor(&mut self, name: &str, targs: &[TypeRef], pos: Pos) -> Option<Ty> {
        let want = match name {
            "List" | "Set" => 1,
            "Map" => 2,
            _ => return None,
        };
        if targs.len() != want {
            self.error_help(
                "gram.generic.1",
                pos,
                format!("`{name}` takes {}, but {} given", count(want, "type argument"), was_were(targs.len())),
                format!("write e.g. `{name}[int]()`"),
            );
            return Some(Ty::Err);
        }
        let first = self.resolve_type(&targs[0]);
        Some(match name {
            "List" => {
                self.no_unique(&first, pos, "a list element");
                Ty::List(Box::new(first))
            }
            "Set" => {
                self.require_key(&first, pos);
                Ty::Set(Box::new(first))
            }
            _ => {
                self.require_key(&first, pos);
                let second = self.resolve_type(&targs[1]);
                self.no_unique(&second, pos, "a map value");
                Ty::Map(Box::new(first), Box::new(second))
            }
        })
    }

    /// Map keys and set elements must implement `Hash` (type.map.2).
    fn require_key(&mut self, t: &Ty, pos: Pos) {
        if self.is_unique(t) {
            self.no_unique(t, pos, "a map key or set element");
            return;
        }
        if self.implements(t, self.std.hash) {
            return;
        }
        let why = self.hash_cause(t);
        self.error_help(
            "type.map.2",
            pos,
            format!("{} can't be a map key or set element (it isn't `Hash`)", self.show(t)),
            why,
        );
    }

    /// `{k: v, ...}`, or `{}` where a map or set is expected (type.coll.2-3).
    fn map_literal(&mut self, pairs: &'p [(Expr, Expr)], pos: Pos, expect: Option<&Ty>) -> Ty {
        let expected = match expect {
            Some(Ty::Opt(t)) => Some(&**t),
            e => e,
        };
        if pairs.is_empty() {
            return match expected {
                Some(t @ Ty::Set(_)) => {
                    self.resolved.empty_sets.insert(pos);
                    t.clone()
                }
                Some(t @ Ty::Map(..)) => t.clone(),
                _ => {
                    self.error_help(
                        "type.coll.3",
                        pos,
                        "the type of this empty `{}` isn't known",
                        "use `{}` where a map or set is expected, or write `Map[K, V]()` or `Set[T]()`",
                    );
                    Ty::Err
                }
            };
        }
        let (kh, vh) = match expected {
            Some(Ty::Map(k, v)) => (Some((**k).clone()), Some((**v).clone())),
            _ => (None, None),
        };
        let keys: Vec<&'p Expr> = pairs.iter().map(|(k, _)| k).collect();
        let vals: Vec<&'p Expr> = pairs.iter().map(|(_, v)| v).collect();
        let k = self.elements(&keys, kh, "type.coll.2", "map keys");
        let v = self.elements(&vals, vh, "type.coll.2", "map values");
        self.require_key(&k, pos);
        self.duplicate_literals(&keys, "key");
        Ty::Map(Box::new(k), Box::new(v))
    }

    /// `{a, b, ...}` (type.coll.2).
    fn set_literal(&mut self, items: &'p [Expr], pos: Pos, expect: Option<&Ty>) -> Ty {
        let hint = match expect {
            Some(Ty::Set(t)) => Some((**t).clone()),
            Some(Ty::Opt(o)) => match &**o {
                Ty::Set(t) => Some((**t).clone()),
                _ => None,
            },
            _ => None,
        };
        let items: Vec<&'p Expr> = items.iter().collect();
        let t = self.elements(&items, hint, "type.coll.2", "set elements");
        self.require_key(&t, pos);
        self.duplicate_literals(&items, "element");
        Ty::Set(Box::new(t))
    }

    /// Report a key or element written twice as the same literal (type.map.1).
    fn duplicate_literals(&mut self, items: &[&'p Expr], what: &str) {
        let mut seen: Vec<(Literal, Pos)> = Vec::new();
        for e in items {
            let lit = match e {
                Expr::Int(n, _) => Literal::Int(*n),
                Expr::Unary { op: UnOp::Neg, operand, .. } => match &**operand {
                    Expr::Int(n, _) => Literal::Int(n.wrapping_neg()),
                    _ => continue,
                },
                Expr::Str(s, _) => Literal::Str(s.clone()),
                Expr::Char(c, _) => Literal::Char(*c),
                Expr::Bool(b, _) => Literal::Bool(*b),
                _ => continue,
            };
            match seen.iter().find(|(l, _)| *l == lit) {
                Some((_, first)) => {
                    let (line, col) = (first.line, first.col);
                    self.error_help(
                        "type.map.1",
                        e.pos(),
                        format!("this {what} is written twice in the literal"),
                        format!("the first is at {line}:{col}; remove one"),
                    );
                }
                None => seen.push((lit, e.pos())),
            }
        }
    }

    /// The common type of a literal's elements: the expected one, which each
    /// must fit, or else the first element's.
    fn elements(&mut self, items: &[&'p Expr], hint: Option<Ty>, rule: &'static str, what: &str) -> Ty {
        let known = hint.filter(|h| !has_err(h));
        let mut first: Option<Ty> = None;
        for e in items {
            let want = known.clone().or_else(|| first.clone());
            let t = self.expr(e, want.as_ref(), true);
            if self.reject_unhandled(&t, e.pos()) {
                continue;
            }
            if t == Ty::Unit {
                self.error(rule, e.pos(), format!("{what} must be values"));
                continue;
            }
            match &want {
                Some(w) if !self.accepts(w, &t) => {
                    let msg = format!(
                        "{what} must all have one type: this is {}, the others are {}",
                        self.show(&t),
                        self.show(w)
                    );
                    self.error(rule, e.pos(), msg);
                }
                Some(_) => {}
                None if t != Ty::Never => first = Some(t),
                None => {}
            }
        }
        known.or(first).unwrap_or(Ty::Err)
    }

    /// Explicit type arguments for `params` (gen.infer.2).
    fn explicit_args(&mut self, what: &str, params: &[usize], refs: &[TypeRef], pos: Pos) -> Vec<(usize, Ty)> {
        if refs.len() != params.len() {
            let msg = if params.is_empty() {
                format!("`{what}` takes no type arguments")
            } else {
                format!("`{what}` takes {}, but {} given", count(params.len(), "type argument"), was_were(refs.len()))
            };
            self.error("gen.infer.2", pos, msg);
            return params.iter().map(|p| (*p, Ty::Err)).collect();
        }
        params.iter().zip(refs).map(|(p, r)| (*p, self.resolve_type(r))).collect()
    }

    /// `Variant(field: value, ...)` (type.enum.6), inferring the enum's type
    /// arguments (gen.infer.1).
    fn variant_literal(
        &mut self,
        e: usize,
        v: usize,
        explicit: Option<Vec<(usize, Ty)>>,
        args: &'p [Arg],
        pos: Pos,
        expect: Option<&Ty>,
    ) -> Ty {
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
            let explicit = explicit.map(|b| b.into_iter().map(|(_, t)| t).collect());
            return self.enum_value(e, explicit, expect, pos, &variant.name);
        }
        let params = self.enum_params[e].clone();
        let mut binds = explicit.unwrap_or_else(|| self.expected_binds(Def::Enum(e), expect));
        let types = self.variant_types[e][v].clone();
        let label = format!("{}.{}", prog.enums[e].name, variant.name);
        let before = self.diags.len();
        self.fields_literal(&label, &variant.fields, &types, &params, &mut binds, args, pos);
        let quiet = self.diags.len() > before;
        self.finish(&label, &params, &mut binds, pos, quiet);
        Ty::Enum(e, params.iter().map(|p| bound(&binds, *p)).collect())
    }

    /// Check a call's arguments against a signature (expr.call.1-2,
    /// own.param.2, own.excl.1-2), inferring its type parameters
    /// (gen.infer.1) and checking their bounds (gen.bound.3). `binds` holds
    /// the type parameters already known, and `recv` is a method's receiver.
    /// Returns the type of the call.
    #[allow(clippy::too_many_arguments)]
    fn check_call(
        &mut self,
        what: &str,
        sig: &Sig,
        mut binds: Vec<(usize, Ty)>,
        args: &'p [Arg],
        recv: Option<&'p Expr>,
        pos: Pos,
        expect: Option<&Ty>,
    ) -> Ty {
        for a in args {
            if let Some((_, ppos)) = &a.param {
                self.key_misplaced(*ppos);
            } else if let Some((_, lpos)) = &a.label {
                self.unsupported(*lpos, "named arguments are not supported by the bootstrap yet (ADR 0058)");
            }
        }
        let vars = &sig.generics;
        if args.len() != sig.params.len() {
            self.args_only(args);
            self.error(
                "expr.call.1",
                pos,
                format!("`{what}` takes {}, but {} given", count(sig.params.len(), "argument"), was_were(args.len())),
            );
            return subst_unknown(&sig.ret, &binds, vars);
        }
        let before = self.diags.len();
        for (k, (a, (pmut, pty))) in args.iter().zip(&sig.params).enumerate() {
            let hint = subst_unknown(pty, &binds, vars);
            let t = self.expr(&a.value, (hint != Ty::Err).then_some(&hint), true);
            self.move_arg(a, sig.moves.get(k).copied().unwrap_or(false));
            bind(pty, &t, vars, &mut binds);
            let want = subst_unknown(pty, &binds, vars);
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
            if !self.accepts(&want, &t) {
                self.mismatch("expr.call.2", a.value.pos(), &want, &t);
            }
        }
        // Type parameters that only the result mentions come from the expected type.
        if let Some(e) = expect
            && vars.iter().any(|v| !binds.iter().any(|(p, _)| p == v))
        {
            bind(&sig.ret, e, vars, &mut binds);
        }
        let quiet = self.diags.len() > before;
        self.finish(what, vars, &mut binds, pos, quiet);
        // own.uniq.5: generic code may copy its values, so a unique type
        // can't be a function's type argument.
        for (v, t) in &binds {
            if sig.generics[sig.owner.min(sig.generics.len())..].contains(v) {
                self.no_unique(t, pos, "a type argument");
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
        subst(&sig.ret, &binds)
    }

    /// Report the type parameters among `vars` that inference left unknown
    /// (gen.infer.1), unless an error was already reported (`quiet`), and
    /// check every bound (gen.bound.3). A trait's `Self` is already known to
    /// implement it.
    fn finish(&mut self, what: &str, vars: &[usize], binds: &mut Vec<(usize, Ty)>, pos: Pos, quiet: bool) {
        let missing: Vec<usize> = vars.iter().copied().filter(|v| !binds.iter().any(|(p, _)| p == v)).collect();
        if !missing.is_empty() && !quiet {
            let names: Vec<&str> = missing.iter().map(|&m| self.tparams[m].name.as_str()).collect();
            let example = match what.split_once('.') {
                Some((t, m)) => format!("{t}[...].{m}(...)"),
                None => format!("{what}[...](...)"),
            };
            self.error_help(
                "gen.infer.1",
                pos,
                format!(
                    "the type of {} for `{what}` can't be inferred from the arguments or the expected type",
                    list_names(&names)
                ),
                format!("write the type arguments, e.g. `{example}`"),
            );
        }
        binds.extend(missing.into_iter().map(|m| (m, Ty::Err)));
        for &v in vars {
            if self.trait_self.contains(&v) {
                continue;
            }
            let t = bound(binds, v);
            for b in self.tparams[v].bounds.clone() {
                let what = format!("the type parameter `{}` of `{what}`", self.tparams[v].name);
                self.require(&t, b, pos, what);
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

    /// `Type(field: value, ...)` (type.struct.3-5), inferring the struct's
    /// type arguments (gen.infer.1).
    fn struct_literal(
        &mut self,
        s: usize,
        explicit: Option<Vec<(usize, Ty)>>,
        args: &'p [Arg],
        pos: Pos,
        expect: Option<&Ty>,
    ) -> Ty {
        let prog = self.prog;
        let decl = &prog.structs[s];
        let params = self.struct_params[s].clone();
        let mut binds = explicit.unwrap_or_else(|| self.expected_binds(Def::Struct(s), expect));
        let types = self.field_types[s].clone();
        let before = self.diags.len();
        self.private_literal(s, pos);
        self.fields_literal(&decl.name, &decl.fields, &types, &params, &mut binds, args, pos);
        let quiet = self.diags.len() > before;
        self.finish(&decl.name, &params, &mut binds, pos, quiet);
        Ty::Struct(s, params.iter().map(|p| bound(&binds, *p)).collect())
    }

    /// The `(field: value, ...)` of a struct literal or a variant with data
    /// (type.struct.3-5, type.enum.6). `types` may mention the type
    /// parameters `vars`, which the values bind.
    #[allow(clippy::too_many_arguments)]
    fn fields_literal(
        &mut self,
        type_label: &str,
        fields: &'p [Field],
        types: &[Ty],
        vars: &[usize],
        binds: &mut Vec<(usize, Ty)>,
        args: &'p [Arg],
        pos: Pos,
    ) {
        let mut given = vec![false; fields.len()];
        let slots = field_slots(fields, args);
        self.positional_types(type_label, fields, types, args, &slots);
        let mut lost = false;
        for (a, (slot, kind)) in args.iter().zip(&slots) {
            if a.mutable {
                self.error_help(
                    "own.param.2",
                    a.value.pos(),
                    "`mut` can't mark a struct field's value",
                    "remove the `mut`",
                );
            }
            let Some(i) = *slot else {
                self.expr(&a.value, None, true);
                lost = true;
                match &a.label {
                    Some((label, lpos)) => {
                        let names: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();
                        let mut d =
                            Diagnostic::new("type.struct.4", *lpos, format!("`{type_label}` has no field `{label}`"));
                        if let Some(s) = suggest(label, &names) {
                            d = d.did_you_mean(label, s);
                        }
                        self.diags.push(d);
                    }
                    None => self.error(
                        "type.struct.4",
                        a.value.pos(),
                        format!("too many arguments: `{type_label}` has {}", count(fields.len(), "field")),
                    ),
                }
                continue;
            };
            let label = &fields[i].name;
            if given[i] {
                let at = a.label.as_ref().map_or(a.value.pos(), |(_, p)| *p);
                let how = if *kind == ArgKind::Shorthand { " (a local named like it sets it too)" } else { "" };
                self.error("type.struct.4", at, format!("field `{label}` is given more than once{how}"));
            }
            given[i] = true;
            let hint = subst_unknown(&types[i], binds, vars);
            let t = self.expr(&a.value, (hint != Ty::Err).then_some(&hint), true);
            if self.reject_unhandled(&t, a.value.pos()) {
                continue;
            }
            self.consume(&a.value, &t);
            bind(&types[i], &t, vars, binds);
            let ft = subst_unknown(&types[i], binds, vars);
            if !self.accepts(&ft, &t) {
                let msg = format!("field `{label}` is {}, but this is {}", self.show(&ft), self.show(&t));
                self.error("type.struct.5", a.value.pos(), msg);
            }
        }
        let missing: Vec<&str> =
            fields.iter().zip(&given).filter(|(_, g)| !**g).map(|(f, _)| f.name.as_str()).collect();
        if !missing.is_empty() && !lost {
            let msg = format!(
                "`{type_label}(...)` is missing {} {}",
                if missing.len() == 1 { "field" } else { "fields" },
                list_names(&missing)
            );
            self.error_help("type.struct.4", pos, msg, "every field must be given; there are no defaults yet");
        }
    }

    /// type.struct.7: positional arguments may only fill fields whose types
    /// all differ, so that swapping two of them is a type error.
    fn positional_types(
        &mut self,
        type_label: &str,
        fields: &[Field],
        types: &[Ty],
        args: &[Arg],
        slots: &[(Option<usize>, ArgKind)],
    ) {
        let filled: Vec<usize> =
            slots.iter().filter(|(_, k)| *k == ArgKind::Positional).filter_map(|(i, _)| *i).collect();
        let clash = filled
            .iter()
            .enumerate()
            .find_map(|(n, &a)| filled[n + 1..].iter().find(|&&b| may_be_same(&types[a], &types[b])).map(|&b| (a, b)));
        let Some((a, b)) = clash else { return };
        let at = args
            .iter()
            .zip(slots)
            .find(|(_, (i, k))| *k == ArgKind::Positional && *i == Some(b))
            .map_or(Pos { line: 0, col: 0 }, |(arg, _)| arg.value.pos());
        let (fa, fb) = (&fields[a].name, &fields[b].name);
        let msg = format!(
            "fields `{fa}` and `{fb}` of `{type_label}` are both {}, so they can't be given by position",
            self.show(&types[a])
        );
        self.error_help("type.struct.7", at, msg, format!("name them: `{type_label}({fa}: ..., {fb}: ...)`"));
    }

    fn method(&mut self, recv: &'p Expr, name: &'p str, pos: Pos, args: &'p [Arg], expect: Option<&Ty>) -> Ty {
        // type.coll.13: `xs.push(x)` or `s.add(x)` can fix a pending local's type.
        if let Expr::Name(n, _) = recv
            && let Some(k) = self.pending_of(n)
            && let [arg] = args
        {
            match (name, self.pending[k].brace) {
                ("push", false) => {
                    let t = self.quiet_type(&arg.value);
                    self.settle(k, Ty::List(Box::new(t)));
                }
                ("add", true) => {
                    let t = self.quiet_type(&arg.value);
                    self.settle(k, Ty::Set(Box::new(t)));
                }
                _ => {}
            }
        }
        // `Type.name(...)` and `Type[int].name(...)`: an associated function
        // (trait.method.4) or a variant; `Trait.name(x, ...)`: a qualified
        // trait method call (trait.res.1).
        let head = match recv {
            Expr::Name(n, _) if self.local(n).is_none() => Some((n.as_str(), None)),
            Expr::TypeArgs { name, args, .. } => Some((name.as_str(), Some(args.as_slice()))),
            _ => None,
        };
        if let Some((tn, targs)) = head {
            if targs.is_none()
                && let Some(t) = self.std_call(tn, name, pos, args, expect)
            {
                return t;
            }
            if let Some(def) = self.types.get(tn).copied() {
                return self.associated(def, targs, name, pos, args, expect);
            }
            // type.char.4: `char.from_code(n)`.
            if tn == "char" && targs.is_none() {
                if name != "from_code" {
                    self.args_only(args);
                    self.error("expr.name.1", pos, format!("`char` has no associated function `{name}`"));
                    return Ty::Err;
                }
                let sig = Sig::plain(None, vec![(false, Ty::Int)], Ty::Opt(Box::new(Ty::Char)));
                return self.check_call("char.from_code", &sig, Vec::new(), args, None, pos, expect);
            }
            if targs.is_none()
                && let Some(&tr) = self.traits.get(tn)
            {
                return self.qualified(tr, name, pos, args);
            }
        }
        let rt = self.expr(recv, None, true);
        if self.reject_unhandled(&rt, recv.pos()) || rt == Ty::Err {
            self.args_only(args);
            return Ty::Err;
        }
        // own.ref.3: a method of a `ref T` is called on the `T` in the cell.
        let rt = match rt {
            Ty::Ref(t) => *t,
            t => t,
        };
        if self.reject_drop_call(&rt, name, pos) {
            self.args_only(args);
            return Ty::Err;
        }
        if let Ty::Conc(k, inner) = &rt {
            self.resolved.handle_calls.insert(pos);
            return self.conc_method(*k, inner, name, pos, args);
        }
        if let Some(t) = self.ui_method(recv, &rt, name, pos, args, expect) {
            return t;
        }
        let cands = self.candidates(&rt, name);
        match cands.as_slice() {
            [] => {
                self.args_only(args);
                if !self.hidden_std_module(recv, name, pos) {
                    self.no_method(&rt, name, pos);
                }
                Ty::Err
            }
            [Cand::Builtin] => self.builtin_call(recv, &rt, name, pos, args),
            [Cand::Inherent(i)] => {
                self.private_fn(*i, pos);
                let sig = self.sigs[*i].clone();
                let tn = self.type_name(&rt).unwrap_or_default();
                self.resolved.symbols.insert(pos, format!("method:{tn}.{name}"));
                match sig.receiver {
                    None => {
                        self.args_only(args);
                        self.error_help(
                            "trait.method.4",
                            pos,
                            format!("`{name}` is an associated function, not a method"),
                            format!("call it as `{tn}.{name}(...)`"),
                        );
                        Ty::Err
                    }
                    Some(mutable) => {
                        if mutable {
                            self.require_mutable(recv, Some("trait.method.3"), "the receiver of a `mut self` method");
                        }
                        self.resolved.targets.insert(pos, *i);
                        self.note_deprecated(*i, pos);
                        let binds = match &rt {
                            Ty::Struct(_, a) | Ty::Enum(_, a) => {
                                sig.generics[..sig.owner].iter().copied().zip(a.iter().cloned()).collect()
                            }
                            _ => Vec::new(),
                        };
                        self.check_call(name, &sig, binds, args, Some(recv), pos, expect)
                    }
                }
            }
            [Cand::Trait(tr)] => self.trait_call(*tr, &rt, recv, name, pos, args, expect),
            _ => {
                self.args_only(args);
                let forms: Vec<String> = cands
                    .iter()
                    .map(|c| match c {
                        Cand::Trait(t) => format!("{}.{name}(x)", self.prog.traits[*t].name),
                        _ => format!("{}.{name}", self.ty_name(&rt)),
                    })
                    .collect();
                let forms: Vec<&str> = forms.iter().map(String::as_str).collect();
                self.error_help(
                    "trait.res.1",
                    pos,
                    format!("`{name}` could mean any of {} here", list_names(&forms)),
                    "call a trait's method in the qualified form, e.g. `Trait.name(x, ...)`",
                );
                Ty::Err
            }
        }
    }

    /// `x.name(...)` where `name` comes from trait `tr`, which the type of `x` implements.
    #[allow(clippy::too_many_arguments)]
    fn trait_call(
        &mut self,
        tr: usize,
        rt: &Ty,
        recv: &'p Expr,
        name: &str,
        pos: Pos,
        args: &'p [Arg],
        expect: Option<&Ty>,
    ) -> Ty {
        let f = self.trait_method(tr, name).expect("a candidate's method");
        let sig = self.sigs[f].clone();
        if matches!(rt, Ty::Any(_)) && !self.callable_through_any(tr, f) {
            self.args_only(args);
            self.error_help(
                "trait.any.4",
                pos,
                format!(
                    "`{name}` can't be called on {}: its signature uses `Self` other than as the receiver",
                    self.show(rt)
                ),
                format!(
                    "call it on a value of a concrete type, or use a type parameter `[T {}]` instead of `any`",
                    self.prog.traits[tr].name
                ),
            );
            return Ty::Err;
        }
        if sig.receiver == Some(true) {
            self.require_mutable(recv, Some("trait.method.3"), "the receiver of a `mut self` method");
        }
        self.resolved.calls.insert(pos, Call::Trait(tr));
        if let Some(&k) = traits::con_of(rt).and_then(|c| self.impl_of.get(&(tr, c))) {
            effects::record_impl_target(self.prog, &mut self.resolved, pos, k, tr, name);
        }
        let binds = vec![(self.trait_self[tr], rt.clone())];
        self.check_call(name, &sig, binds, args, Some(recv), pos, expect)
    }

    /// `Trait.name(x, ...)`: a trait's method called on `x` (trait.res.1).
    fn qualified(&mut self, tr: usize, name: &str, pos: Pos, args: &'p [Arg]) -> Ty {
        let trait_name = &self.prog.traits[tr].name;
        if self.trait_method(tr, name).is_none() {
            self.args_only(args);
            let names: Vec<&str> =
                self.prog.traits[tr].methods.iter().map(|&m| self.prog.fns[m].name.as_str()).collect();
            let mut d = Diagnostic::new("expr.name.1", pos, format!("`{trait_name}` has no method `{name}`"));
            if let Some(s) = suggest(name, &names) {
                d = d.did_you_mean(name, s);
            }
            self.diags.push(d);
            return Ty::Err;
        }
        let Some(first) = args.first() else {
            self.error_help(
                "expr.call.1",
                pos,
                format!("`{trait_name}.{name}` needs the value to call it on as its first argument"),
                format!("write `{trait_name}.{name}(x, ...)`"),
            );
            return Ty::Err;
        };
        let recv = &first.value;
        let rt = self.expr(recv, None, true);
        if self.reject_unhandled(&rt, recv.pos()) || rt == Ty::Err {
            self.args_only(&args[1..]);
            return Ty::Err;
        }
        if !self.implements(&rt, tr) && !matches!(rt, Ty::Any(a) if a == tr) {
            self.args_only(&args[1..]);
            let msg = format!("{} doesn't implement `{trait_name}`", self.show(&rt));
            match self.how_to_implement(&rt, tr) {
                Some(h) => self.error_help("gen.bound.3", recv.pos(), msg, h),
                None => self.error("gen.bound.3", recv.pos(), msg),
            }
            return Ty::Err;
        }
        let t = self.trait_call(tr, &rt, recv, name, pos, &args[1..], None);
        self.resolved.calls.insert(pos, Call::Qualified(tr));
        t
    }

    /// `Type.name(...)` or `Type[int].name(...)`: an associated function
    /// (trait.method.4), or a variant with data (type.enum.6).
    #[allow(clippy::too_many_arguments)]
    fn associated(
        &mut self,
        def: Def,
        targs: Option<&[TypeRef]>,
        name: &'p str,
        pos: Pos,
        args: &'p [Arg],
        expect: Option<&Ty>,
    ) -> Ty {
        let tn = self.def_name(def);
        let owner = self.def_params(def).to_vec();
        if let Def::Enum(e) = def
            && let Some(v) = self.variant_index(e, name)
        {
            self.resolved.symbols.insert(pos, format!("variant:{tn}.{name}"));
            let explicit = targs.map(|a| self.explicit_args(tn, &owner, a, pos));
            return self.variant_literal(e, v, explicit, args, pos, expect);
        }
        let Some(i) = self.method_index(tn, name) else {
            self.args_only(args);
            self.error("expr.name.1", pos, format!("`{tn}` has no associated function `{name}`"));
            return Ty::Err;
        };
        self.private_fn(i, pos);
        self.resolved.symbols.insert(pos, format!("method:{tn}.{name}"));
        let sig = self.sigs[i].clone();
        if sig.receiver.is_some() {
            self.args_only(args);
            self.error_help(
                "trait.method.4",
                pos,
                format!("`{name}` is a method, so it's called on a value"),
                format!("call it as `x.{name}(...)` where `x` is a `{tn}`"),
            );
            return Ty::Err;
        }
        let binds = match targs {
            Some(a) => {
                let params = sig.generics[..sig.owner].to_vec();
                self.explicit_args(tn, &params, a, pos)
            }
            None => Vec::new(),
        };
        self.resolved.targets.insert(pos, i);
        self.note_deprecated(i, pos);
        self.check_call(&format!("{tn}.{name}"), &sig, binds, args, None, pos, expect)
    }

    /// Report a method call on a type without that method.
    fn no_method(&mut self, rt: &Ty, name: &str, pos: Pos) {
        let mut names: Vec<&str> = match self.type_name(rt).and_then(|tn| self.methods.get(tn)) {
            Some(m) => m.keys().copied().collect(),
            None => builtins::builtin_names(rt).to_vec(),
        };
        for (t, decl) in self.prog.traits.iter().enumerate() {
            if self.implements(rt, t) || matches!(rt, Ty::Any(a) if *a == t) {
                names.extend(decl.methods.iter().map(|&m| self.prog.fns[m].name.as_str()));
            }
        }
        let is_field = matches!(rt, Ty::Struct(s, _) if self.prog.structs[*s].fields.iter().any(|f| f.name == name));
        let shown = self.show(rt);
        if is_field {
            self.error_help(
                "expr.name.1",
                pos,
                format!("`{name}` is a field of {shown}, not a method"),
                "remove the `()`",
            );
            return;
        }
        let mut d = Diagnostic::new("expr.name.1", pos, format!("{shown} has no method `{name}`"));
        if let Some(h) = builtins::method_help(rt, name) {
            d = d.help(h);
        } else if let Some(s) = suggest(name, &names) {
            d = d.did_you_mean(name, s);
        } else if let Ty::Param(p) = rt {
            d = d.help(format!(
                "a type parameter has only the methods of its constraints; add one, e.g. `[{} Trait]`",
                self.tparams[*p].name
            ));
        }
        self.diags.push(d);
    }

    fn field(&mut self, recv: &'p Expr, name: &str, pos: Pos, expect: Option<&Ty>) -> Ty {
        // `Enum.Variant` and `Enum[int].Variant` (type.enum.3).
        let head = match recv {
            Expr::Name(n, _) if self.local(n).is_none() => Some((n.as_str(), None)),
            Expr::TypeArgs { name, args, .. } => Some((name.as_str(), Some(args.as_slice()))),
            _ => None,
        };
        if let Some((tn, targs)) = head
            && let Some(def) = self.types.get(tn).copied()
        {
            match def {
                Def::Enum(e) => {
                    if let Some(v) = self.variant_index(e, name) {
                        self.resolved.symbols.insert(pos, format!("variant:{tn}.{name}"));
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
                        let params = self.enum_params[e].clone();
                        let explicit = targs
                            .map(|a| self.explicit_args(tn, &params, a, pos).into_iter().map(|(_, t)| t).collect());
                        return self.enum_value(e, explicit, expect, pos, name);
                    }
                    let names: Vec<&str> = self.prog.enums[e].variants.iter().map(|v| v.name.as_str()).collect();
                    let mut d = Diagnostic::new("expr.name.1", pos, format!("`{tn}` has no variant `{name}`"));
                    if let Some(s) = suggest(name, &names) {
                        d = d.did_you_mean(name, s);
                    }
                    self.diags.push(d);
                }
                Def::Struct(_) => {
                    self.error("type.struct.6", pos, format!("`{tn}` is a type; fields are read from values"))
                }
            }
            return Ty::Err;
        }
        let rt = self.expr(recv, None, true);
        if self.reject_unhandled(&rt, recv.pos()) || rt == Ty::Err {
            return Ty::Err;
        }
        // own.ref.3: a field of a `ref T` is the field of the `T` in the cell.
        let rt = match rt {
            Ty::Ref(t) => *t,
            t => t,
        };
        if !self.candidates(&rt, name).is_empty() {
            self.error_help(
                "expr.op.3",
                pos,
                format!("`{name}` is a method, so calling it needs `()`"),
                format!("write `.{name}()`"),
            );
            return Ty::Err;
        }
        match &rt {
            Ty::Struct(s, args) => {
                let decl = &self.prog.structs[*s];
                if let Some(i) = decl.fields.iter().position(|f| f.name == name) {
                    self.private_field(*s, i, pos);
                    self.resolved.symbols.insert(pos, format!("field:{}.{name}", decl.name));
                    let binds: Vec<(usize, Ty)> =
                        self.struct_params[*s].iter().copied().zip(args.iter().cloned()).collect();
                    return subst(&self.field_types[*s][i], &binds);
                }
                let names: Vec<&str> = decl.fields.iter().map(|f| f.name.as_str()).collect();
                let mut d = Diagnostic::new("type.struct.6", pos, format!("`{}` has no field `{name}`", decl.name));
                if let Some(s) = suggest(name, &names) {
                    d = d.did_you_mean(name, s);
                }
                self.diags.push(d);
                Ty::Err
            }
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
        // `m[k]` (type.map.3).
        if let Ty::Map(k, v) = &rt {
            if matches!(index, Expr::Range { .. }) {
                self.error("type.map.3", index.pos(), "a map can't be sliced");
                return Ty::Err;
            }
            let kt = self.expr(index, Some(k), true);
            if !self.accepts(k, &kt) {
                self.mismatch("type.map.3", index.pos(), k, &kt);
            }
            return (**v).clone();
        }
        // type.coll.4: indexes start at 0; there are no negative indexes.
        let negative =
            |e: &Expr| matches!(e, Expr::Unary { op: UnOp::Neg, operand, .. } if matches!(**operand, Expr::Int(..)));
        let bad = match index {
            Expr::Range { lo, hi, .. } => [lo, hi].into_iter().flatten().find(|b| negative(b)).map(|b| b.pos()),
            e if negative(e) => Some(e.pos()),
            _ => None,
        };
        if let Some(at) = bad {
            self.error_help(
                "type.coll.4",
                at,
                "a list has no negative indexes",
                "count from the end with `xs.len()`: `xs[xs.len() - 1]`, or `xs.last()`",
            );
        }
        let slice = matches!(index, Expr::Range { .. });
        if let Expr::Range { lo, hi, .. } = index {
            self.range_bounds(lo.as_deref(), hi.as_deref());
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
                self.error_help(
                    "type.str.2",
                    recv.pos(),
                    "a `str` can't be indexed",
                    "index its characters: `s.chars()[i]`, or `s.chars()[a..b].join(\"\")` for a part",
                );
                Ty::Err
            }
            t => {
                let msg = format!("only lists and maps can be indexed, not {}", self.show(&t));
                self.error("type.coll.4", recv.pos(), msg);
                Ty::Err
            }
        }
    }

    /// Whether `e` is an integer literal (or its negation), which can be a
    /// float where one is expected; if so, it is recorded as one (type.num.10).
    fn float_literal(&mut self, e: &Expr) -> bool {
        match e {
            Expr::Int(_, pos) => {
                self.resolved.float_literals.insert(*pos);
                true
            }
            Expr::Unary { op: UnOp::Neg, operand, .. } => self.float_literal(operand),
            _ => false,
        }
    }

    /// An integer literal `v` (negated when `neg`) typed as `k` by context
    /// (type.num.13): it must fit, and an `f32` takes it as a float.
    fn sized_literal(&mut self, v: i64, neg: bool, pos: Pos, k: Num) {
        if !k.is_float() {
            let value = if neg { -i128::from(v) } else { i128::from(v) };
            if !k.fits(value) {
                let (lo, hi) = k.range();
                self.error_help(
                    "type.num.13",
                    pos,
                    format!("`{value}` doesn't fit in `{}`", k.name()),
                    format!("`{}` holds {lo} to {hi}", k.name()),
                );
            }
        }
        self.resolved.sized_literals.insert(pos, k);
        // The interpreter's fast path treats every literal in this set as special.
        self.resolved.float_literals.insert(pos);
    }

    /// Whether `e` is an integer literal (or its negation) that takes the
    /// sized type `k` from its other operand (type.num.13); if so, record it.
    fn sized_operand(&mut self, e: &Expr, k: Num, neg: bool) -> bool {
        match e {
            Expr::Int(v, pos) => {
                self.sized_literal(*v, neg, *pos, k);
                true
            }
            Expr::Unary { op: UnOp::Neg, operand, .. } => self.sized_operand(operand, k, true),
            _ => false,
        }
    }

    fn unary(&mut self, op: UnOp, operand: &'p Expr, pos: Pos) -> Ty {
        let (want, rule, what) = match op {
            UnOp::Neg => (Ty::Int, "trait.op.1", "unary `-`"),
            UnOp::Not => (Ty::Bool, "type.bool.1", "`not`"),
            UnOp::Spawn => return self.spawn_expr(operand, pos),
        };
        let t = self.expr(operand, Some(&want), true);
        if self.reject_unhandled(&t, operand.pos()) {
            return Ty::Err;
        }
        if op == UnOp::Neg && t == Ty::Float {
            return Ty::Float;
        }
        if let (UnOp::Neg, Ty::Num(k)) = (op, &t) {
            if !k.is_signed() && !k.is_float() {
                let msg = format!("unary `-` is not defined for `{}`, which is unsigned", k.name());
                self.error_help("type.num.14", pos, msg, "negate a signed type, or subtract from zero with `wrapping_neg`");
                return Ty::Err;
            }
            return t;
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
        // expr.cmp.1: the right operand's expected type is the left's.
        let mut lt = self.expr(lhs, None, true);
        let rt = self.expr(rhs, Some(&lt), true);
        // type.num.10: `2 * x` with a float `x`.
        if lt == Ty::Int && rt == Ty::Float && self.float_literal(lhs) {
            lt = Ty::Float;
        }
        // type.num.13: `1 + x` with a sized `x`.
        if let (Ty::Int, Ty::Num(k)) = (&lt, &rt)
            && self.sized_operand(lhs, *k, false)
        {
            lt = rt.clone();
        }
        // expr.cmp.3: `xs.index_of(x) == -1` would always be false.
        if matches!(op, BinOp::Eq | BinOp::Ne)
            && lt == Ty::Opt(Box::new(Ty::Int))
            && matches!(rhs, Expr::Unary { op: UnOp::Neg, operand, .. } if matches!(**operand, Expr::Int(..)))
        {
            self.error_help(
                "expr.cmp.3",
                rhs.pos(),
                "this compares an optional with a negative number, so it's never equal: absence is `none`",
                "write `== none`, or bind it: `if i := ...`",
            );
        }
        self.operator(op, &lt, &rt, pos)
    }

    /// Which types support which operators (type.num.5, type.str.3,
    /// trait.op.1, trait.op.3, expr.cmp.1, trait.std.1-2).
    fn operator(&mut self, op: BinOp, lt: &Ty, rt: &Ty, pos: Pos) -> Ty {
        let sym = op.symbol();
        let fail = if op.is_arithmetic() { Ty::Err } else { Ty::Bool };
        if matches!(lt, Ty::Err | Ty::Never) || matches!(rt, Ty::Err | Ty::Never) {
            return fail;
        }
        if self.reject_unhandled(lt, pos) || self.reject_unhandled(rt, pos) {
            return Ty::Err;
        }
        if *lt == Ty::Unit || *rt == Ty::Unit {
            self.error("trait.op.1", pos, format!("an operand of `{sym}` produces no value"));
            return Ty::Err;
        }
        // expr.cmp.1: the right operand converts to the left's type, as an
        // argument would (`opt == 1`, type.opt.5).
        if !same(lt, rt) && (op.is_arithmetic() || !self.accepts(lt, rt)) {
            let msg = format!("`{sym}` needs operands of one type, found {} and {}", self.show(lt), self.show(rt));
            if op == BinOp::Add && (*lt == Ty::Str || *rt == Ty::Str) {
                self.error_help("trait.op.3", pos, msg, "build text with interpolation: \"{a}{b}\"");
            } else {
                self.error("trait.op.3", pos, msg);
            }
            return fail;
        }
        if op.is_arithmetic() {
            match lt {
                Ty::Int => return Ty::Int,
                // type.num.14: sized arithmetic; `f32` has no `%`.
                Ty::Num(k) if !(k.is_float() && op == BinOp::Rem) => return lt.clone(),
                // type.num.6: float arithmetic, without `%`.
                Ty::Float if op != BinOp::Rem => return Ty::Float,
                Ty::Str if op == BinOp::Add => self.diags.push(
                    Diagnostic::new(UNSUPPORTED, pos, "joining strings with `+` is not decided yet (spec trait.op.1)")
                        .help("build text with interpolation: \"{a}{b}\""),
                ),
                t => {
                    let msg = format!("`{sym}` is not defined for {}", self.show(t));
                    self.error("trait.op.1", pos, msg);
                }
            }
            return Ty::Err;
        }
        let ordering = matches!(op, BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge);
        let tr = if ordering { self.std.ord } else { self.std.eq };
        if !self.implements(lt, tr) {
            let trait_name = if ordering { "Ord" } else { "Eq" };
            let msg = format!("`{sym}` needs a type that implements `{trait_name}`, and {} doesn't", self.show(lt));
            match self.how_to_implement(lt, tr) {
                Some(h) => self.error_help("expr.cmp.1", pos, msg, h),
                None => self.error("expr.cmp.1", pos, msg),
            }
        }
        Ty::Bool
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
                        self.locals.push(Local { name: name.clone(), ty, mutable: false, kind: Kind::Binding, decl: None });
                    }
                }
            }
            agree.block(self, body, pos);
            self.end_scope(mark);
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
                            self.locals.push(Local { name: name.clone(), ty: t, mutable: false, kind: Kind::Binding, decl: None });
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
            self.end_scope(mark);
        }
        if need && els.is_none() { Ty::Err } else { agree.result() }
    }

    /// Help for `else` or `if name :=` on a value that is never absent, when
    /// it looks like an optional in other languages (expr.name.2).
    fn never_absent_help(value: &Expr) -> Option<&'static str> {
        match value {
            Expr::Index { .. } => {
                Some("indexing traps when the index or key is missing; for a map, `m.get(k)` gives an optional")
            }
            Expr::Method { name, .. } if name == "to_int" => {
                Some("`x.to_int()` on a float traps on NaN or out of range; it never returns `none`")
            }
            _ => None,
        }
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
                let help = Self::never_absent_help(value).unwrap_or("compare it instead, e.g. `if x == ...`");
                self.error_help("expr.if.5", value.pos(), msg, help);
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
        // An enum, or a type with literal patterns (expr.match.7).
        let (e, literal) = match st {
            Ty::Enum(e, ref args) => (Some((e, args.clone())), None),
            Ty::Int | Ty::Char | Ty::Str | Ty::Bool => (None, Some(st.clone())),
            Ty::Num(k) if !k.is_float() => (None, Some(st.clone())),
            Ty::Err => (None, None),
            ref t if self.reject_unhandled(t, scrutinee.pos()) => (None, None),
            Ty::Union(_) => {
                self.unsupported(
                    scrutinee.pos(),
                    "matching on a set of error types is not designed yet (OQ-029); match on one error type",
                );
                (None, None)
            }
            ref t => {
                let msg = format!("a `match` works on an enum, `int`, `char`, `str` or `bool`, not {}", self.show(t));
                self.error_help("expr.match.4", scrutinee.pos(), msg, "use `if` and `==` instead");
                (None, None)
            }
        };
        let mut covered = vec![false; e.as_ref().map_or(0, |(e, _)| self.prog.enums[*e].variants.len())];
        let mut seen: Vec<crate::ast::Literal> = Vec::new();
        let mut wildcard = false;
        let mut agree = Agree::new(expect, need, "arm", "`match`");
        for arm in arms {
            let bindings = match (&e, &literal, &arm.pat) {
                (Some((e, args)), _, _) => self.pattern(*e, args, &arm.pat, &mut covered, &mut wildcard),
                (_, Some(t), _) => {
                    self.literal_pattern(t, &arm.pat, &mut seen, &mut wildcard);
                    Vec::new()
                }
                // The scrutinee was already reported: bind names quietly to
                // avoid follow-on errors.
                (None, None, Pattern::Variant { bindings: Some(names), .. }) => {
                    names.iter().map(|(n, p)| (n.clone(), *p, Ty::Err)).collect()
                }
                _ => Vec::new(),
            };
            let mark = self.locals.len();
            for (name, npos, ty) in bindings {
                if name != "_" && self.declare(&name, npos) {
                    self.locals.push(Local { name, ty, mutable: false, kind: Kind::Binding, decl: None });
                }
            }
            agree.block(self, &arm.body, pos);
            self.end_scope(mark);
        }
        if let Some((e, _)) = e {
            let missing_variants: Vec<&crate::ast::Variant> =
                (self.prog.enums[e].variants.iter()).zip(&covered).filter(|(_, c)| !**c).map(|(v, _)| v).collect();
            let missing: Vec<&str> = missing_variants.iter().map(|v| v.name.as_str()).collect();
            if !wildcard && !missing.is_empty() {
                let mut d = Diagnostic::new(
                    "expr.match.2",
                    pos,
                    format!("this `match` doesn't cover {}", list_names(&missing)),
                )
                .help("add an arm for each, or a `_` arm");
                // fix.edit.4: one arm per missing variant, each a typed hole.
                if let Some(last) = arms.last() {
                    let indent = " ".repeat(last.pat.pos().col - 1);
                    let text: String = missing_variants
                        .iter()
                        .map(|v| {
                            let binds = if v.fields.is_empty() {
                                String::new()
                            } else {
                                format!("({})", vec!["_"; v.fields.len()].join(", "))
                            };
                            format!("{indent}{}{binds} => todo\n", v.name)
                        })
                        .collect();
                    d = d.fix("add the missing arms, each a `todo`", vec![Edit::after_block(last.pat.pos(), text)]);
                }
                self.diags.push(d);
            }
        }
        if let Some(t) = &literal {
            let both_bools = *t == Ty::Bool
                && seen.contains(&crate::ast::Literal::Bool(true))
                && seen.contains(&crate::ast::Literal::Bool(false));
            if !wildcard && !both_bools {
                let help = if *t == Ty::Bool {
                    "add the missing `true` or `false` arm"
                } else {
                    "add a `_` arm for the other values"
                };
                let mut d = Diagnostic::new(
                    "expr.match.2",
                    pos,
                    format!("this `match` on {} doesn't cover every value", self.show(t)),
                )
                .help(help);
                if let Some(last) = arms.last() {
                    let indent = " ".repeat(last.pat.pos().col - 1);
                    let text: String = if *t == Ty::Bool {
                        [true, false]
                            .iter()
                            .filter(|b| !seen.contains(&crate::ast::Literal::Bool(**b)))
                            .map(|b| format!("{indent}{b} => todo\n"))
                            .collect()
                    } else {
                        format!("{indent}_ => todo\n")
                    };
                    d = d.fix("add the missing arms, each a `todo`", vec![Edit::after_block(last.pat.pos(), text)]);
                }
                self.diags.push(d);
            }
        }
        agree.result()
    }

    /// Check a pattern on a value of type `t` with literal patterns
    /// (expr.match.5, expr.match.7-8).
    fn literal_pattern(&mut self, t: &Ty, pat: &Pattern, seen: &mut Vec<crate::ast::Literal>, wildcard: &mut bool) {
        use crate::ast::Literal;
        match pat {
            Pattern::Wildcard(pos) => {
                // `true` and `false` cover a `bool` (expr.match.7).
                let bools =
                    *t == Ty::Bool && seen.contains(&Literal::Bool(true)) && seen.contains(&Literal::Bool(false));
                if *wildcard || bools {
                    self.error("expr.match.5", *pos, "this arm can never match; earlier arms cover every case");
                }
                *wildcard = true;
            }
            Pattern::Or(alts, _) => {
                for a in alts {
                    if matches!(a, Pattern::Wildcard(_)) {
                        self.error(
                            "expr.match.8",
                            a.pos(),
                            "`_` can't be one of several alternatives; it already matches everything",
                        );
                    } else {
                        self.literal_pattern(t, a, seen, wildcard);
                    }
                }
            }
            Pattern::Literal(lit, pos) => {
                let lt = match lit {
                    Literal::Int(_) if matches!(t, Ty::Num(k) if !k.is_float()) => t.clone(),
                    Literal::Int(_) => Ty::Int,
                    Literal::Char(_) => Ty::Char,
                    Literal::Str(_) => Ty::Str,
                    Literal::Bool(_) => Ty::Bool,
                };
                if lt != *t {
                    let msg = format!("this pattern is {}, but the value matched is {}", self.show(&lt), self.show(t));
                    let help = match (&lt, t) {
                        (Ty::Str, Ty::Char) => Some("a character pattern is written in single quotes: `'a'`"),
                        (Ty::Char, Ty::Str) => Some("a string pattern is written in double quotes: `\"a\"`"),
                        _ => None,
                    };
                    match help {
                        Some(h) => self.error_help("expr.match.7", *pos, msg, h),
                        None => self.error("expr.match.7", *pos, msg),
                    }
                } else if *wildcard || seen.contains(lit) {
                    self.error("expr.match.5", *pos, "this arm can never match; an earlier arm covers this value");
                } else {
                    seen.push(lit.clone());
                }
            }
            Pattern::Variant { pos, name, .. } => {
                let msg = format!("`{name}` is a variant pattern, but the value matched is {}", self.show(t));
                self.error("expr.match.4", *pos, msg);
            }
        }
    }

    /// Check one pattern against the scrutinee's enum and type arguments
    /// (expr.match.3-6, expr.match.8), and return the names it binds with
    /// their types.
    fn pattern(
        &mut self,
        e: usize,
        args: &[Ty],
        pat: &Pattern,
        covered: &mut [bool],
        wildcard: &mut bool,
    ) -> Vec<(String, Pos, Ty)> {
        let prog = self.prog;
        let decl = &prog.enums[e];
        match pat {
            // expr.match.8: alternatives bind no names.
            Pattern::Or(alts, _) => {
                for a in alts {
                    match a {
                        Pattern::Variant { bindings: Some(names), pos, .. } if names.iter().any(|(n, _)| n != "_") => {
                            self.error_help(
                                "expr.match.8",
                                *pos,
                                "a pattern with alternatives can't bind names",
                                "give this variant an arm of its own, or use `_` for its fields",
                            );
                        }
                        Pattern::Wildcard(p) => {
                            self.error(
                                "expr.match.8",
                                *p,
                                "`_` can't be one of several alternatives; it already matches everything",
                            );
                        }
                        _ => {
                            self.pattern(e, args, a, covered, wildcard);
                        }
                    }
                }
                Vec::new()
            }
            Pattern::Literal(_, pos) => {
                let msg = format!("`{}` is an enum; match it with its variants", decl.name);
                self.error("expr.match.4", *pos, msg);
                Vec::new()
            }
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
                        d = d.did_you_mean(name, s);
                    }
                    self.diags.push(d);
                    return Vec::new();
                };
                self.resolved.symbols.insert(*pos, format!("variant:{}.{name}", decl.name));
                if *wildcard || covered[v] {
                    self.error("expr.match.5", *pos, format!("this arm can never match; `{name}` is already covered"));
                } else {
                    covered[v] = true;
                }
                // expr.match.6: one binding (or `_`) per field, in order;
                // expr.match.10: a bare name matches any fields and binds none.
                let fields = &decl.variants[v].fields;
                let Some(given) = bindings.as_deref() else {
                    return Vec::new();
                };
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
                let binds: Vec<(usize, Ty)> = self.enum_params[e].iter().copied().zip(args.iter().cloned()).collect();
                let types = &self.variant_types[e][v];
                given.iter().zip(types).map(|((n, p), t)| (n.clone(), *p, subst(t, &binds))).collect()
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
        if let Some(h) = builtins::function_help(name) {
            d = d.help(h);
        } else if let Some(s) = suggest(name, &candidates) {
            d = d.did_you_mean(name, s);
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

/// Whether a loop body has a `break` that ends this loop (not one of a
/// nested loop's).
fn breaks(body: &[Stmt]) -> bool {
    // A `break` for this loop can sit in any block of an `if` or `match`
    // expression, wherever that expression appears (but not in a nested loop).
    fn in_expr(e: &Expr) -> bool {
        match e {
            Expr::If { branches, els, .. } => {
                branches.iter().any(|(_, b)| breaks(b)) || els.as_ref().is_some_and(|b| breaks(b))
            }
            Expr::Match { scrutinee, arms, .. } => in_expr(scrutinee) || arms.iter().any(|a| breaks(&a.body)),
            Expr::Unary { operand, .. } => in_expr(operand),
            Expr::Binary { lhs, rhs, .. } => in_expr(lhs) || in_expr(rhs),
            Expr::Call { callee, args, .. } => in_expr(callee) || args.iter().any(|a| in_expr(&a.value)),
            Expr::Method { recv, args, .. } => in_expr(recv) || args.iter().any(|a| in_expr(&a.value)),
            Expr::Field { recv, .. } => in_expr(recv),
            Expr::Index { recv, index, .. } => in_expr(recv) || in_expr(index),
            Expr::List(items, _) | Expr::Set(items, _) => items.iter().any(in_expr),
            Expr::Map(pairs, _) => pairs.iter().any(|(k, v)| in_expr(k) || in_expr(v)),
            Expr::Try { expr, .. } => in_expr(expr),
            Expr::Else { value, otherwise, .. } => in_expr(value) || in_else(otherwise),
            _ => false,
        }
    }
    fn in_else(o: &Else) -> bool {
        match o {
            Else::Leave(st) => matches!(**st, Stmt::Break(_)),
            Else::Block { body, .. } => breaks(body),
            Else::Value(v) => in_expr(v),
        }
    }
    body.iter().any(|s| match s {
        Stmt::Break(_) => true,
        Stmt::Expr(e) | Stmt::Return(Some(e), _) => in_expr(e),
        Stmt::Let { value, otherwise, .. } => in_expr(value) || otherwise.as_ref().is_some_and(in_else),
        Stmt::Handle { value, otherwise, .. } => in_expr(value) || in_else(otherwise),
        Stmt::Assign { target, value, .. } => in_expr(target) || in_expr(value),
        Stmt::While { cond, .. } => in_expr(cond),
        Stmt::For { iter, .. } => in_expr(iter),
        _ => false,
    })
}

/// Whether two declared field types could be the same type: equal, or
/// both mentioning type parameters (type.struct.7).
fn may_be_same(a: &Ty, b: &Ty) -> bool {
    fn has_param(t: &Ty) -> bool {
        match t {
            Ty::Param(_) => true,
            Ty::List(e) | Ty::Opt(e) | Ty::Set(e) | Ty::Conc(_, e) => has_param(e),
            Ty::Map(k, v) => has_param(k) || has_param(v),
            Ty::Struct(_, args) | Ty::Enum(_, args) => args.iter().any(has_param),
            _ => false,
        }
    }
    a == b || (has_param(a) && has_param(b))
}

fn count(n: usize, noun: &str) -> String {
    if n == 1 { format!("1 {noun}") } else { format!("{n} {noun}s") }
}

fn was_were(n: usize) -> String {
    if n == 1 { "1 was".into() } else { format!("{n} were") }
}

/// Closest candidate by edit distance, if it's close enough to be a likely typo.
pub(crate) fn suggest<'a>(name: &str, candidates: &[&'a str]) -> Option<&'a str> {
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
    use crate::{lexer::lex, parser::parse_with_prelude};

    fn diags(src: &str) -> Vec<Diagnostic> {
        let (tokens, ld) = lex(src);
        assert!(ld.is_empty(), "{ld:?}");
        let (prog, pd) = parse_with_prelude(tokens);
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
    fn generic_calls_infer_and_check_bounds() {
        let lib = "fn largest[T Ord](xs List[T]) T\n    xs[0]\nfn empty[T]() List[T]\n    []\nstruct P\n    x int\n";
        let ok = format!("{lib}fn main()\n    a := largest([1, 2])\n    b := largest(empty[str]())\n");
        assert!(rules(&ok).is_empty(), "{:?}", rules(&ok));
        let bad = format!("{lib}fn main()\n    a := largest([P(x: 1)])\n    b := empty()\n");
        assert_eq!(rules(&bad), vec!["gen.bound.3", "gen.infer.1"]);
    }

    #[test]
    fn equality_is_automatic_and_ordering_opt_in() {
        let src = "struct P\n    x int\nenum E\n    A\n    B\nfn main()\n    a := P(x: 1) == P(x: 2)\n    b := E.A != B\n    c := P(x: 1) < P(x: 2)\n";
        assert_eq!(rules(src), vec!["expr.cmp.1"]);
        let derived = format!("{src}impl Ord for P\n");
        assert!(rules(&derived).is_empty(), "{:?}", rules(&derived));
    }

    #[test]
    fn edit_distance_basics() {
        assert_eq!(edit_distance("prnt", "print"), 1);
        assert_eq!(edit_distance("", "abc"), 3);
    }
}
