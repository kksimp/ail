//! Syntax tree for the subset the bootstrap supports.

use crate::diag::Pos;

#[derive(Debug)]
pub struct Program {
    pub fns: Vec<FnDecl>,
}

#[derive(Debug)]
pub struct FnDecl {
    pub name: String,
    pub pos: Pos,
    pub params: Vec<Param>,
    pub ret: Option<TypeRef>,
    pub body: Vec<Stmt>,
}

#[derive(Debug)]
pub struct Param {
    pub name: String,
    pub pos: Pos,
    pub ty: TypeRef,
}

#[derive(Debug, Clone)]
pub struct TypeRef {
    pub name: String,
    pub pos: Pos,
}

#[derive(Debug)]
pub enum Stmt {
    /// `name := value` (expr.bind.1)
    Let { name: String, pos: Pos, value: Expr },
    Expr(Expr),
}

#[derive(Debug)]
pub enum Expr {
    Str(String, Pos),
    Int(i64, Pos),
    Name(String, Pos),
    Call { callee: Box<Expr>, args: Vec<Expr>, pos: Pos },
}

impl Expr {
    pub fn pos(&self) -> Pos {
        match self {
            Expr::Str(_, p) | Expr::Int(_, p) | Expr::Name(_, p) => *p,
            Expr::Call { callee, .. } => callee.pos(),
        }
    }
}
