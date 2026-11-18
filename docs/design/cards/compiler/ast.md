# compiler: ast

## ast
```
pub struct ast.Pos  ## A source position: line and column, from 1.
pub struct ast.Ident  ## A name and where it is written.
pub struct ast.Program
pub struct ast.SourceFile  ## One source file of a package: its lines are numbered from `base + 1`.
pub struct ast.Grant  ## `caps LIST` on a manifest line (eff.cap.4-5).
pub struct ast.Generated  ## The methods a derive generator wrote for one `impl` (ct.gen.1-4).
pub struct ast.Use  ## `use a.b.c`, or `use a.b.c as d` (mod.use.1).
pub struct ast.ConstDecl  ## `NAME := expr` at the top level (mod.const.1).
pub struct ast.GenericParam  ## `T`, or `T Bound + Bound` (gen.decl.1, gen.bound.1).
pub struct ast.TraitDecl
pub struct ast.ImplDecl
pub struct ast.StructDecl
pub struct ast.Field
pub struct ast.EnumDecl
pub struct ast.Variant
pub struct ast.FnDecl
pub struct ast.Extern  ## Where an `extern c fn` lives; `none` is the process itself.
pub struct ast.Contract  ## `requires COND`, `ensures COND` or `invariant COND` (contract.decl.1-2).
pub struct ast.Old  ## An `old(e)` of an `ensures`: the hidden name it reads and `e`.
pub struct ast.Attr  ## `@name` or `@name("arg")` (gram.attr.1-3).
pub enum ast.Within  ## Where a function is declared.
pub struct ast.Owner  ## The type a method belongs to, or a trait's functions' trait.
pub struct ast.Receiver  ## `self`, `mut self` or `move self`.
pub struct ast.Param
pub struct ast.TypeRef  ## A type as written.
pub enum ast.TypeShape
pub enum ast.Stmt
pub struct ast.SelectArm  ## One arm of a `select` (conc.select.2): what it waits for and its block.
pub enum ast.SelectOp  ## What a `select` arm waits for (conc.select.2).
pub struct ast.Binder  ## One name of a destructuring binding (type.tuple.4).
pub enum ast.Else  ## The `else` part of `x := e else ...` (err.else.1-3).
pub enum ast.BinOp
pub enum ast.UnOp
pub enum ast.Expr
pub struct ast.ClosureParam  ## A parameter of a function literal: `x`, or `x int` with its type (type.fn.4).
pub struct ast.MapEntry
pub struct ast.Branch  ## One `if`/`elif` condition and its block.
pub enum ast.CompKind
pub struct ast.CompFor
pub enum ast.Cond  ## A `bool` condition, or a binding `name := e` (expr.if.5).
pub enum ast.InterpPart
pub struct ast.FormatSpec  ## `{expr:spec}` (lex.str.7).
pub struct ast.Arg  ## A call argument (own.param.2, own.move.2, type.struct.3, expr.key.1).
pub struct ast.Arm
pub enum ast.Pattern
pub enum ast.Literal
pub fn ast.tail_else(ast.Expr, ast.Else) ast.Expr?  ## err.else.7 (ADR 1095): `e else return ...`, `else break` or `else continue`
pub fn ast.expr_start(ast.Expr) ast.Pos  ## Where an expression starts, for the position of a tail `else` (err.else.7).
pub fn ast.key_body(ast.Arg) ast.Expr  ## The body `e` of a key function `|x| e` written as an argument (expr.key.1):
pub fn ast.expr_names(ast.Expr, mut Set[str]) [mutates]  ## Add each name `e` reads to `out`.
pub fn ast.stmt_names(ast.Stmt, mut Set[str]) [mutates]
pub struct ast.ParScan  ## What a loop body does, for deciding whether its chunks can run apart.
pub fn ast.new_scan() ast.ParScan
pub fn ast.lanes_target(List[ast.Stmt], str) str  ## A loop body over `var` that is one element-wise store, `c[var] = e` with
pub fn ast.scan_block(mut ast.ParScan, List[ast.Stmt], int) [mutates]
pub fn ast.select_heads(List[ast.SelectArm]) List[ast.Expr]  ## The expressions a `select` evaluates before it waits, in source order.
pub fn ast.select_bodies(List[ast.SelectArm]) List[List[ast.Stmt]]  ## The blocks of a `select`'s arms, in source order; at most one runs.
pub fn ast.select_var(ast.SelectOp) ast.Binder?  ## The name a receive arm binds, or none.
pub fn ast.select_always_runs(List[ast.SelectArm]) bool  ## Whether a `select` always runs an arm: it has a `closed` or an `after`
pub ast.YIELD := "yield"  ## The reserved callee of a `yield` statement.
pub fn ast.yield_stmt(ast.Expr, ast.Pos) ast.Stmt  ## The statement `yield value` at `pos`.
pub fn ast.is_yield_callee(ast.Expr) bool  ## Whether `callee` is the reserved name of a `yield`.
pub fn ast.yield_value(ast.Stmt) ast.Expr?  ## The operand of `s` when it is `yield e`.
pub fn ast.has_yield(List[ast.Stmt]) bool  ## Whether a function body contains a `yield`, at any depth of its blocks:
```

trimmed to fit 2000 tokens (~1563); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence)
