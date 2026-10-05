//! Syntax tree for the subset the bootstrap supports.

use crate::diag::Pos;

#[derive(Debug, Default)]
pub struct Program {
    pub fns: Vec<FnDecl>,
    pub structs: Vec<StructDecl>,
    pub enums: Vec<EnumDecl>,
}

/// `struct Name` with one `name Type` field per line (type.struct.1-2).
#[derive(Debug)]
pub struct StructDecl {
    pub name: String,
    pub pos: Pos,
    pub fields: Vec<Field>,
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub pos: Pos,
    pub ty: TypeRef,
}

/// `enum Name` with one variant per line (type.enum.1). Variants carrying
/// data (type.enum.2) aren't supported yet.
#[derive(Debug)]
pub struct EnumDecl {
    pub name: String,
    pub pos: Pos,
    pub variants: Vec<(String, Pos)>,
}

/// `fn name(...)`, or `fn Type.name(...)` for a method or associated
/// function (trait.method.1, trait.method.4).
#[derive(Debug)]
pub struct FnDecl {
    pub owner: Option<(String, Pos)>,
    pub name: String,
    pub pos: Pos,
    pub receiver: Option<Receiver>,
    pub params: Vec<Param>,
    pub ret: Option<TypeRef>,
    pub body: Block,
}

impl FnDecl {
    /// `name` or `Type.name`, as written in diagnostics.
    pub fn full_name(&self) -> String {
        match &self.owner {
            Some((owner, _)) => format!("{owner}.{}", self.name),
            None => self.name.clone(),
        }
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

/// A type as written, e.g. `int` or `List[Student]` (gram.generic.1).
#[derive(Debug, Clone)]
pub struct TypeRef {
    pub name: String,
    pub args: Vec<TypeRef>,
    pub pos: Pos,
}

pub type Block = Vec<Stmt>;

#[derive(Debug)]
pub enum Stmt {
    /// `name := value` or `mut name := value` (expr.bind.1-2)
    Let {
        name: String,
        pos: Pos,
        mutable: bool,
        value: Expr,
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
    For {
        var: String,
        var_pos: Pos,
        iter: Expr,
        body: Block,
        pos: Pos,
    },
    Break(Pos),
    Continue(Pos),
    Return(Option<Expr>, Pos),
    Expr(Expr),
}

impl Stmt {
    pub fn pos(&self) -> Pos {
        match self {
            Stmt::Let { pos, .. } | Stmt::While { pos, .. } | Stmt::For { pos, .. } => *pos,
            Stmt::Assign { target, .. } => target.pos(),
            Stmt::Break(p) | Stmt::Continue(p) | Stmt::Return(_, p) => *p,
            Stmt::Expr(e) => e.pos(),
        }
    }
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
    Bool(bool, Pos),
    /// A name, including `self` and bare enum variants (type.enum.3).
    Name(String, Pos),
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
    /// `a..b` or `a..=b` (expr.loop.3)
    Range {
        lo: Box<Expr>,
        hi: Box<Expr>,
        inclusive: bool,
        pos: Pos,
    },
    /// `if c` / `elif c` / `else` (expr.if.1)
    If {
        branches: Vec<(Expr, Block)>,
        els: Option<Block>,
        pos: Pos,
    },
    /// `match e` with `pattern => result` arms (expr.match.1)
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<Arm>,
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
            | Expr::Bool(_, p)
            | Expr::Name(_, p)
            | Expr::List(_, p)
            | Expr::Unary { pos: p, .. }
            | Expr::If { pos: p, .. }
            | Expr::Match { pos: p, .. } => *p,
            Expr::Call { callee: e, .. }
            | Expr::Method { recv: e, .. }
            | Expr::Field { recv: e, .. }
            | Expr::Index { recv: e, .. }
            | Expr::Binary { lhs: e, .. }
            | Expr::Range { lo: e, .. } => e.pos(),
        }
    }
}

#[derive(Debug)]
pub enum InterpPart {
    Lit(String),
    Expr(Expr),
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
    /// `Variant` or `Enum.Variant`
    Variant {
        qualifier: Option<String>,
        name: String,
        pos: Pos,
    },
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
