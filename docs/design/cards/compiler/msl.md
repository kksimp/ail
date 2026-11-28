# compiler: msl

## msl
```
pub struct msl.KParam  ## One kernel argument, in buffer order: `kind` is `in` or `out` (a list; `ty` its element type),
    pub name str
    pub kind str
    pub ty str
pub struct msl.Kernel  ## A kernel, or why there is none.
    pub ok bool
    pub why str
    pub source str
    pub params List[msl.KParam]
struct msl.Ex
    text str
    ty str
struct msl.Em
    var str
    env Map[str, str]
    locals Set[str]
    ins Set[str]
    outs Set[str]
    scalars Set[str]
    lens Set[str]
    bytes_in Set[str]
    structs Map[str, Map[str, str]]
    why str
    depth int
    fn msl.Em.reject(mut self, why str) msl.Ex [mutates]
    fn msl.Em.expr(mut self, e ast.Expr) msl.Ex [mutates]
    fn msl.Em.name_expr(mut self, n str) msl.Ex [mutates]
    fn msl.Em.member_expr(mut self, recv ast.Expr, field str) msl.Ex [mutates]  ## `p.field` of a struct argument: the argument's field, passed as a constant `p_field`.
    fn msl.Em.bytes_of(mut self, recv ast.Expr) str [mutates]  ## The byte list a `bytes` argument name, or "".
    fn msl.Em.list_of(mut self, recv ast.Expr) str [mutates]
    fn msl.Em.index_expr(mut self, recv ast.Expr, idx ast.Expr) msl.Ex [mutates]
    fn msl.Em.unary_expr(mut self, op ast.UnOp, x ast.Expr) msl.Ex [mutates]
    fn msl.Em.binary_expr(mut self, op ast.BinOp, l ast.Expr, r ast.Expr) msl.Ex [mutates]
    fn msl.Em.method_expr(mut self, recv ast.Expr, name str, args List[ast.Arg]) msl.Ex [mutates]
    fn msl.Em.block(mut self, body List[ast.Stmt], depth int) List[str] [mutates]  ## The lines of `body`, indented one level deeper than `depth`.
    fn msl.Em.stmt(mut self, s ast.Stmt, depth int) List[str] [mutates]
    fn msl.Em.assign(mut self, target ast.Expr, op ast.BinOp?, value ast.Expr, p str) List[str] [mutates]
    fn msl.Em.for_stmt(mut self, is_par bool, var str, iter ast.Expr, body List[ast.Stmt], depth int) List[str] [mutates]
    fn msl.Em.if_stmt(mut self, branches List[ast.Branch], els List[ast.Stmt]?, depth int) List[str] [mutates]
    fn msl.Em.finish(mut self, name str, var str, lines List[str]) msl.Kernel [mutates]
fn msl.type_text(t ast.TypeRef) str  ## The AIL type as written, `List[float]`.
pub fn msl.type_name(t ast.TypeRef) str  ## The AIL type as written, `float`, `List[float]`.
fn msl.elem_of(ty str) str  ## The element type of a list type, or "".
fn msl.msl_type(ty str) str  ## The MSL type of an AIL scalar type, or "".
fn msl.is_int(ty str) bool
fn msl.is_num(ty str) bool
fn msl.op_text(op ast.BinOp) str
fn msl.is_cmp(op ast.BinOp) bool
fn msl.names(s Set[str]) List[str]  ## The members of `s`, in order.
fn msl.cat3(a List[str], b List[str], c List[str]) List[str]  ## `a` then `b`, `c` as one list.
fn msl.pad(depth int) str
fn msl.bytes_helpers() str
fn msl.new_em(var str, env Map[str, str], structs Map[str, Map[str, str]]) msl.Em
pub fn msl.kernel(name str, var str, body List[ast.Stmt], env Map[str, str]) msl.Kernel  ## Builds kernel `name` for `par for var in 0..n` with `body`. `env` maps each name the body may use
pub fn msl.kernel_fn(name str, f ast.FnDecl, var str, structs List[ast.StructDecl]) msl.Kernel  ## Kernel `name` of function `f` run for each value of its parameter `var` (the `r` of
pub fn msl.kernel_decls(name str, f ast.FnDecl, var str, decls Map[str, Map[str, str]]) msl.Kernel  ## `kernel_fn` with the structs given as struct name to field name to AIL type.
fn msl.names_of(m Map[str, str]) List[str]
fn msl.fn_kernels(f ast.FnDecl) List[str]  ## The `par for v in 0..n` loops of `fn`'s body at its top level, each as a kernel named
fn msl.comps_of(e ast.Expr) List[str]  ## The parallel comprehensions `[f(.., r, ..) par for r in 0..n]` in `e`, as (callee, loop variable).
fn msl.stmts_comps(body List[ast.Stmt]) List[str]
fn msl.comp_kernels(f ast.FnDecl, prog ast.Program) List[str]
pub fn msl.command(args List[str]) int [fs io]  ## `ail gpu-msl FILE`: the kernel of each top-level `par for` in FILE, or why not (a development
```
