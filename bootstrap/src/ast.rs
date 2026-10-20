//! Syntax tree for the subset the bootstrap supports.

use std::collections::HashSet;

use crate::diag::Pos;

#[derive(Debug, Default)]
pub struct Program {
    /// Every function: free functions, methods, and the functions of
    /// `trait` and `impl` blocks (see `FnDecl::within`).
    pub fns: Vec<FnDecl>,
    pub structs: Vec<StructDecl>,
    pub enums: Vec<EnumDecl>,
    pub traits: Vec<TraitDecl>,
    pub impls: Vec<ImplDecl>,
    pub consts: Vec<ConstDecl>,
    /// `use` declarations of every file (mod.use.1).
    pub uses: Vec<Use>,
    /// The lines that start with `pub`: a declaration or struct field on one
    /// of them is public (mod.vis.1). Every declaration starts its own line.
    pub public: HashSet<usize>,
    /// The source files of a package, in line order (`modules.rs`). Empty
    /// for a single-file program without imports.
    pub files: Vec<SourceFile>,
}

/// `use a.b.c`, or `use a.b.c as d` (mod.use.1, std.use.1).
#[derive(Debug, Clone)]
pub struct Use {
    pub path: Vec<(String, Pos)>,
    pub alias: Option<(String, Pos)>,
    pub pos: Pos,
}

impl Use {
    /// The name the module is available under: the alias, or the last segment.
    pub fn name(&self) -> &(String, Pos) {
        self.alias.as_ref().unwrap_or_else(|| self.path.last().expect("a use path has a segment"))
    }

    /// `a.b.c`
    pub fn dotted(&self) -> String {
        self.path.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(".")
    }
}

/// One source file of a package: its lines are numbered from `base + 1`.
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: String,
    pub src: String,
    /// The module's import path, `a.b` (mod.unit.2); empty for the root module.
    pub module: String,
    pub base: usize,
}

/// Each file of a package is numbered from a multiple of this, so a
/// position tells its file (the first file starts at 0).
pub const FILE_LINES: usize = 1 << 20;

impl Program {
    /// The file a position is in, if the program has more than one.
    pub fn file_at(&self, pos: Pos) -> Option<&SourceFile> {
        if is_prelude(pos) {
            return None;
        }
        self.files.iter().rev().find(|f| f.base < pos.line)
    }

    /// The import path of the module a position is in (mod.unit.2); empty
    /// for the root module and the prelude.
    pub fn module_at(&self, pos: Pos) -> &str {
        self.file_at(pos).map_or("", |f| f.module.as_str())
    }

    /// Whether the declaration at `pos` is marked `pub`, or is in the prelude.
    pub fn is_pub(&self, pos: Pos) -> bool {
        is_prelude(pos) || self.public.contains(&pos.line)
    }

    /// Whether the declaration at `decl` is hidden from code at `at`: it is
    /// private and in another module (mod.vis.1).
    pub fn hidden(&self, decl: Pos, at: Pos) -> bool {
        !self.is_pub(decl) && self.module_at(decl) != self.module_at(at)
    }
}

/// `NAME := expr` at the top level: a constant (mod.const.1).
#[derive(Debug)]
pub struct ConstDecl {
    pub name: String,
    pub pos: Pos,
    /// The `##` comment just before the declaration (lex.comment.2); filled
    /// by `docs::attach`.
    pub doc: Option<String>,
    pub value: Expr,
}

/// The prelude (the standard traits, `Ordering` and a few functions) is
/// read before the program, numbered from this line so its positions never
/// collide with the program's.
pub const PRELUDE_LINE: usize = 1 << 30;

pub fn is_prelude(pos: Pos) -> bool {
    pos.line >= PRELUDE_LINE
}

/// `T`, or `T Bound + Bound`, in a generic parameter list (gen.decl.1, gen.bound.1).
#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: String,
    pub pos: Pos,
    pub bounds: Vec<(String, Pos)>,
}

/// `trait Name` with an indented block of functions (trait.decl.1-2).
#[derive(Debug)]
pub struct TraitDecl {
    pub name: String,
    pub pos: Pos,
    /// The `##` comment just before the declaration (lex.comment.2); filled
    /// by `docs::attach`.
    pub doc: Option<String>,
    /// Indexes into `Program::fns`.
    pub methods: Vec<usize>,
}

/// `impl Trait for Type`, optionally with an indented block of functions
/// (trait.impl.1). The target may list its parameters to add bounds
/// (gen.method.2).
#[derive(Debug)]
pub struct ImplDecl {
    pub trait_name: String,
    pub trait_pos: Pos,
    pub target: Owner,
    pub pos: Pos,
    /// Indexes into `Program::fns`.
    pub methods: Vec<usize>,
}

/// `struct Name` with one `name Type` field per line (type.struct.1-2), or
/// `error struct Name` (err.decl.1).
#[derive(Debug)]
pub struct StructDecl {
    pub name: String,
    pub pos: Pos,
    /// The `##` comment just before the declaration (lex.comment.2); filled
    /// by `docs::attach`.
    pub doc: Option<String>,
    pub is_error: bool,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<Field>,
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub pos: Pos,
    /// The `##` comment just before the declaration (lex.comment.2); filled
    /// by `docs::attach`.
    pub doc: Option<String>,
    pub ty: TypeRef,
}

/// `enum Name` with one variant per line (type.enum.1), or `error enum Name`
/// (err.decl.1).
#[derive(Debug)]
pub struct EnumDecl {
    pub name: String,
    pub pos: Pos,
    /// The `##` comment just before the declaration (lex.comment.2); filled
    /// by `docs::attach`.
    pub doc: Option<String>,
    pub is_error: bool,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<Variant>,
}

/// `Name` or `Name(field Type, ...)` (type.enum.2).
#[derive(Debug)]
pub struct Variant {
    pub name: String,
    pub pos: Pos,
    /// The `##` comment just before the declaration (lex.comment.2); filled
    /// by `docs::attach`.
    pub doc: Option<String>,
    pub fields: Vec<Field>,
}

/// `fn name(...)`, or `fn Type.name(...)` for a method or associated
/// function (trait.method.1, trait.method.4). Functions in `trait` and `impl`
/// blocks have the trait or the implementing type as their owner.
#[derive(Debug)]
pub struct FnDecl {
    /// Attributes written before `fn` (gram.attr.1-2).
    pub attrs: Vec<Attr>,
    pub owner: Option<Owner>,
    pub name: String,
    pub pos: Pos,
    /// The `##` comment just before the declaration (lex.comment.2); filled
    /// by `docs::attach`.
    pub doc: Option<String>,
    /// The function's own type parameters (gen.decl.1, gen.method.3).
    pub generics: Vec<GenericParam>,
    pub receiver: Option<Receiver>,
    pub params: Vec<Param>,
    pub ret: Option<TypeRef>,
    /// `None` only for a trait method without a default (trait.decl.2).
    pub body: Option<Block>,
    pub within: Within,
}

/// `@name` or `@name(arguments)` (gram.attr.1); the arguments aren't kept yet.
#[derive(Debug, Clone)]
pub struct Attr {
    pub name: String,
    pub pos: Pos,
    /// The string argument, as in `@deprecated("why")` (gram.attr.3).
    pub arg: Option<String>,
}

/// Where a function is declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Within {
    /// At the top level: a free function, method or associated function.
    TopLevel,
    /// In `trait` block number `n`.
    Trait(usize),
    /// In `impl` block number `n`.
    Impl(usize),
}

/// The type a method belongs to: `Stack`, or `Stack[T Ord]` to add bounds
/// (gen.method.1-2). For a trait's functions, the trait.
#[derive(Debug, Clone)]
pub struct Owner {
    pub name: String,
    pub pos: Pos,
    pub params: Option<Vec<GenericParam>>,
}

impl FnDecl {
    /// `name` or `Type.name`, as written in diagnostics.
    pub fn full_name(&self) -> String {
        match &self.owner {
            Some(owner) => format!("{}.{}", owner.name, self.name),
            None => self.name.clone(),
        }
    }

    /// A free function, method or associated function declared at the top level.
    pub fn top_level(&self) -> bool {
        self.within == Within::TopLevel
    }

    pub fn has_attr(&self, name: &str) -> bool {
        self.attrs.iter().any(|a| a.name == name)
    }
}

/// `self` or `mut self` (trait.method.1).
#[derive(Debug, Clone, Copy)]
pub struct Receiver {
    pub mutable: bool,
}

/// `name Type` or `mut name Type` (gram.fn.2, own.param.1-2).
#[derive(Debug)]
pub struct Param {
    pub name: String,
    pub pos: Pos,
    pub mutable: bool,
    pub ty: TypeRef,
}

/// A type as written (gram.generic.1, type.opt.2, err.type.1-2, err.type.5).
#[derive(Debug, Clone)]
pub struct TypeRef {
    pub kind: TypeKind,
    pub pos: Pos,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    /// `int`, `List[Student]`
    Named { name: String, args: Vec<TypeRef> },
    /// `T?`
    Optional(Box<TypeRef>),
    /// `T ! E`, `T ! A | B`, `!T`, or a bare `!`. `ok` is `None` when there is
    /// no success value; `errors` is `None` when the set is inferred.
    Fallible { ok: Option<Box<TypeRef>>, errors: Option<Vec<TypeRef>> },
    /// `any Trait` (trait.any.1)
    Any(String),
}

pub type Block = Vec<Stmt>;

#[derive(Debug)]
pub enum Stmt {
    /// `name := value` or `mut name := value` (expr.bind.1-2), optionally
    /// with an `else` part (err.else.1-3).
    Let {
        name: String,
        pos: Pos,
        mutable: bool,
        value: Expr,
        otherwise: Option<Else>,
    },
    /// `target = value` or `target op= value` (expr.bind.3, expr.op.2); `pos` is the operator's.
    Assign {
        target: Expr,
        op: Option<BinOp>,
        value: Expr,
        pos: Pos,
    },
    While {
        cond: Expr,
        body: Block,
        pos: Pos,
    },
    /// `for x in e`, or `for mut x in e` (expr.loop.6).
    For {
        var: String,
        var_pos: Pos,
        mutable: bool,
        iter: Expr,
        body: Block,
        pos: Pos,
    },
    Break(Pos),
    Continue(Pos),
    Return(Option<Expr>, Pos),
    Expr(Expr),
    /// `f(x) else ...`: a call whose result has no success value, with its
    /// failure handled (err.else.4).
    Handle {
        value: Expr,
        otherwise: Else,
    },
}

impl Stmt {
    pub fn pos(&self) -> Pos {
        match self {
            Stmt::Let { pos, .. } | Stmt::While { pos, .. } | Stmt::For { pos, .. } => *pos,
            Stmt::Assign { target, .. } => target.pos(),
            Stmt::Break(p) | Stmt::Continue(p) | Stmt::Return(_, p) => *p,
            Stmt::Expr(e) | Stmt::Handle { value: e, .. } => e.pos(),
        }
    }
}

/// The `else` part of `x := e else ...` (err.else.1-3).
#[derive(Debug)]
pub enum Else {
    /// `else d`: a fallback value.
    Value(Expr),
    /// `else return x`, `else break`, `else continue`.
    Leave(Box<Stmt>),
    /// `else` or `else err`, followed by an indented block.
    Block { err: Option<(String, Pos)>, body: Block, pos: Pos },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::And => "and",
            BinOp::Or => "or",
        }
    }

    pub fn is_arithmetic(self) -> bool {
        matches!(self, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug)]
pub enum Expr {
    Str(String, Pos),
    /// `"text {expr} text"` (lex.str.2)
    Interp(Vec<InterpPart>, Pos),
    Int(i64, Pos),
    /// `1.5` (lex.num.2)
    Float(f64, Pos),
    /// `'a'` (lex.str.4)
    Char(char, Pos),
    Bool(bool, Pos),
    /// `none` (type.opt.4)
    None(Pos),
    /// `todo`, a typed hole (expr.hole.1-2).
    Hole(Pos),
    /// A name, including `self` and bare enum variants (type.enum.3).
    Name(String, Pos),
    /// A generic function or type with explicit type arguments, `f[int]` or
    /// `Stack[int]` (gen.infer.2). Only a callee or a method's receiver.
    TypeArgs {
        name: String,
        args: Vec<TypeRef>,
        pos: Pos,
    },
    /// `[a, b, c]` (type.coll.2)
    List(Vec<Expr>, Pos),
    /// `f(args)`, including struct literals `Type(field: value)` (type.struct.3).
    Call {
        callee: Box<Expr>,
        args: Vec<Arg>,
        pos: Pos,
    },
    /// `value.name(args)` (trait.method.2), or `Type.name(args)` (trait.method.4).
    Method {
        recv: Box<Expr>,
        name: String,
        name_pos: Pos,
        args: Vec<Arg>,
    },
    /// `value.name` (type.struct.6), or `Enum.Variant` (type.enum.3).
    Field {
        recv: Box<Expr>,
        name: String,
        name_pos: Pos,
    },
    /// `xs[i]` or `xs[a..b]` (type.coll.4); `pos` is the `[`.
    Index {
        recv: Box<Expr>,
        index: Box<Expr>,
        pos: Pos,
    },
    Unary {
        op: UnOp,
        operand: Box<Expr>,
        pos: Pos,
    },
    /// `pos` is the operator's.
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        pos: Pos,
    },
    /// `a..b` or `a..=b` (expr.loop.3); in a slice either bound may be
    /// left out, `xs[a..]`, `xs[..b]` (type.coll.10). `pos` is the `..`.
    Range {
        lo: Option<Box<Expr>>,
        hi: Option<Box<Expr>>,
        inclusive: bool,
        pos: Pos,
    },
    /// `{k: v, ...}`, or `{}` for an empty map or set (type.coll.2).
    Map(Vec<(Expr, Expr)>, Pos),
    /// `{a, b, ...}` (type.coll.2)
    Set(Vec<Expr>, Pos),
    /// `if c` / `elif c` / `else` (expr.if.1); `else_err` is the name in
    /// `else err` (expr.if.6).
    If {
        branches: Vec<(Cond, Block)>,
        els: Option<Block>,
        else_err: Option<(String, Pos)>,
        pos: Pos,
    },
    /// `match e` with `pattern => result` arms (expr.match.1)
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<Arm>,
        pos: Pos,
    },
    /// `e?` or `e? "context"` (err.prop.1, err.ctx.2); `pos` is the `?`.
    Try {
        expr: Box<Expr>,
        context: Option<Box<Expr>>,
        pos: Pos,
    },
    /// `e else d` or `e else return ...` as an expression (err.else.6): the
    /// value of `e`, or the fallback when it is absent or failed. `pos` is
    /// the `else`'s.
    Else {
        value: Box<Expr>,
        otherwise: Box<Else>,
        pos: Pos,
    },
}

impl Expr {
    /// Where the expression starts.
    pub fn pos(&self) -> Pos {
        match self {
            Expr::Str(_, p)
            | Expr::Interp(_, p)
            | Expr::Int(_, p)
            | Expr::Float(_, p)
            | Expr::Char(_, p)
            | Expr::Bool(_, p)
            | Expr::None(p)
            | Expr::Hole(p)
            | Expr::Name(_, p)
            | Expr::TypeArgs { pos: p, .. }
            | Expr::List(_, p)
            | Expr::Unary { pos: p, .. }
            | Expr::If { pos: p, .. }
            | Expr::Match { pos: p, .. }
            | Expr::Map(_, p)
            | Expr::Set(_, p)
            | Expr::Range { lo: None, pos: p, .. } => *p,
            Expr::Range { lo: Some(e), .. } => e.pos(),
            Expr::Call { callee: e, .. }
            | Expr::Method { recv: e, .. }
            | Expr::Field { recv: e, .. }
            | Expr::Index { recv: e, .. }
            | Expr::Binary { lhs: e, .. }
            | Expr::Try { expr: e, .. }
            | Expr::Else { value: e, .. } => e.pos(),
        }
    }
}

/// A condition of `if` or `elif`: a `bool` expression, or a binding
/// `name := e` of a value that can be absent or fail (expr.if.5).
#[derive(Debug)]
pub enum Cond {
    Expr(Expr),
    Bind { name: String, pos: Pos, value: Expr },
}

#[derive(Debug)]
pub enum InterpPart {
    Lit(String),
    Expr(Expr),
    /// `{expression:spec}` (lex.str.7).
    Fmt(Expr, crate::lexer::FormatSpec),
}

/// A call argument: `value`, `mut place` (own.param.2) or `label: value` (type.struct.3).
#[derive(Debug)]
pub struct Arg {
    pub label: Option<(String, Pos)>,
    pub mutable: bool,
    pub value: Expr,
}

#[derive(Debug)]
pub struct Arm {
    pub pat: Pattern,
    pub body: Block,
}

/// The patterns supported so far (expr.match.4).
#[derive(Debug)]
pub enum Pattern {
    Wildcard(Pos),
    /// An integer, character, string or `bool` literal (expr.match.7).
    Literal(Literal, Pos),
    /// `A | B`: any of the alternatives, none of which binds names (expr.match.8).
    Or(Vec<Pattern>, Pos),
    /// `Variant`, `Enum.Variant`, or either with one binding (a name or `_`)
    /// per field: `Rect(w, _)` (expr.match.6).
    Variant {
        qualifier: Option<String>,
        name: String,
        bindings: Option<Vec<(String, Pos)>>,
        pos: Pos,
    },
}

/// The value of a literal pattern.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Char(char),
    Str(String),
    Bool(bool),
}

impl Pattern {
    pub fn pos(&self) -> Pos {
        match self {
            Pattern::Wildcard(p) | Pattern::Literal(_, p) | Pattern::Or(_, p) | Pattern::Variant { pos: p, .. } => *p,
        }
    }
}

/// A place expression's root name and path (own.place.1), or `None` if the
/// expression isn't a place. Slices (`xs[a..b]`) are values, not places.
pub fn place_path(e: &Expr) -> Option<(&str, Vec<PathStep<'_>>)> {
    match e {
        Expr::Name(n, _) => Some((n.as_str(), Vec::new())),
        Expr::Field { recv, name, .. } => {
            let (root, mut path) = place_path(recv)?;
            path.push(PathStep::Field(name));
            Some((root, path))
        }
        Expr::Index { recv, index, .. } if !matches!(**index, Expr::Range { .. }) => {
            let (root, mut path) = place_path(recv)?;
            path.push(PathStep::Index(index));
            Some((root, path))
        }
        _ => None,
    }
}

#[derive(Debug)]
pub enum PathStep<'a> {
    Field(&'a str),
    Index(&'a Expr),
}

/// How an argument of a struct literal or variant sets a field
/// (type.struct.3, type.struct.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgKind {
    /// `field: value`
    Named,
    /// A bare name equal to a field's name.
    Shorthand,
    /// Any other expression: it fills the next field not set by name.
    Positional,
}

/// The field each argument sets, in declaration order of `fields`, and how
/// (type.struct.3-4, type.struct.7). A field index is `None` for an unknown
/// label or a positional argument with no field left to fill.
pub fn field_slots(fields: &[Field], args: &[Arg]) -> Vec<(Option<usize>, ArgKind)> {
    let by_name = |n: &str| fields.iter().position(|f| f.name == n);
    let mut slots: Vec<(Option<usize>, ArgKind)> = args
        .iter()
        .map(|a| match (&a.label, &a.value) {
            (Some((label, _)), _) => (by_name(label), ArgKind::Named),
            (None, Expr::Name(n, _)) if by_name(n).is_some() => (by_name(n), ArgKind::Shorthand),
            // A bare member name, which implicit `self` made `self.n` (trait.method.6).
            (None, Expr::Field { recv, name, name_pos })
                if matches!(&**recv, Expr::Name(s, p) if s == "self" && p == name_pos) && by_name(name).is_some() =>
            {
                (by_name(name), ArgKind::Shorthand)
            }
            _ => (None, ArgKind::Positional),
        })
        .collect();
    let taken: Vec<usize> = slots.iter().filter_map(|(i, _)| *i).collect();
    let mut free = (0..fields.len()).filter(|i| !taken.contains(i));
    for slot in slots.iter_mut().filter(|(_, k)| *k == ArgKind::Positional) {
        slot.0 = free.next();
    }
    slots
}
