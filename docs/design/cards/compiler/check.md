# compiler: check

## check
```
pub fn check.expr_pos(ast.Expr) ast.Pos  ## Where an expression is reported: its first token, as `Expr::pos`.
pub fn check.stmt_pos(ast.Stmt) ast.Pos  ## Where a statement is reported, as `Stmt::pos`.
pub enum check.Step  ## A step of a place: a field, or an index.
pub struct check.Place  ## A place expression: a local and the steps into it.
pub enum check.ArgKind  ## How an argument of a struct literal or variant sets a field.
pub struct check.Slot  ## The field an argument sets (`-1` for none) and how (type.struct.3-4, 7).
pub fn check.field_slots(List[ast.Field], List[ast.Arg]) List[check.Slot]  ## The field each argument sets, as `field_slots`.
pub fn check.op_symbol(ast.BinOp) str  ## The symbol of a binary operator, as `BinOp::symbol`.
pub fn check.full_name(ast.FnDecl) str  ## `fn Type.name` or `name`, as `FnDecl::full_name`.
pub fn check.has_attr(ast.FnDecl, str) bool
pub fn check.module_at(ast.Program, ast.Pos) str  ## The module a position is in (mod.unit.2): empty for the root and the prelude.
pub fn check.hidden(ast.Program, ast.Pos, ast.Pos) bool  ## Whether the declaration at `decl` is hidden from code at `at` (mod.vis.1).
pub struct check.Report  ## What the checker found, and the functions it couldn't type yet.
pub fn check.check_program(ast.Program) check.Report [env io]  ## Every pass of `check_with` this checker has, in its order.
pub fn check.check_declarations(ast.Program) List[check.Diag] [env io]  ## The diagnostics of the declaration passes only (slice 1).
pub enum check.Need  ## What a built-in method's element type must support.
pub struct check.Builtin  ## A built-in method's signature.
pub check.STR_METHODS := ...
pub check.BYTES_METHODS := ...  ## type.bytes.7-12
pub check.CHAR_METHODS := ...
pub check.INT_METHODS := ...
pub check.FLOAT_METHODS := ...
pub check.SIZED_INT_METHODS := ...
pub check.F32_METHODS := ...
pub check.LIST_METHODS := ...
pub check.MAP_METHODS := ...
pub check.SET_METHODS := ...
pub enum check.Cand  ## What `x.name(...)` can refer to (trait.res.1).
pub struct check.TParam  ## A type parameter: its name and the traits it must implement (gen.bound.1).
pub struct check.Scoped  ## A type parameter name in scope.
pub struct check.ImplInfo  ## An `impl` block's trait and target, with the block's type parameters
pub struct check.SigParam  ## A parameter of a signature: `mut`, and its type.
pub struct check.Sig  ## A function's signature (`Sig` in check.rs): `receiver` is `true` for
pub struct check.Deferred  ## A bound check waiting until every declaration is known (gen.bound.3).
pub struct check.Checker
pub enum check.LocalKind  ## How a local was declared.
pub struct check.Local
pub struct check.Pending  ## A local declared from `[]` or `{}`, typed by its first use (type.coll.13).
pub struct check.Moved
pub fn check.new_checker(ast.Program) check.Checker
pub check.COVERED_RULES := ...  ## The rules the declaration passes (slice 1), the bodies (slice 2) and the
pub struct check.Skip  ## A range of lines of one file not checked yet: `first` to `last`, inclusive.
pub fn check.skipped(ast.Program, check.Report) List[check.Skip]  ## The lines not checked yet: functions with a construct this checker
pub struct check.Note
pub struct check.Diag
pub fn check.new_diag(str, ast.Pos, str) check.Diag
pub struct check.VariantRef  ## A variant of an enum.
pub struct check.Agree  ## Whether the branches of an `if` or the arms of a `match` agree on a type
pub struct check.Binding  ## The names a pattern binds, with their types.
pub struct check.Coverage  ## What a `match` has covered so far.
pub fn check.check_handles(ast.Program) List[check.Diag]
pub fn check.check_moves(ast.Program, List[check.Moved]) List[check.Diag]
pub check.PRELUDE_FNS := ...  ## Names available without import (mod.main.3).
pub check.BUILTIN_TYPES := ...
pub check.SPEC_TYPES_NOT_YET := ...  ## Type names the spec defines that the bootstrap doesn't support yet.
pub check.GENERATABLE := ...  ## The types a property test can generate without type arguments (test.prop.2).
pub check.SIZED := ...
pub check.CONC_TYPES := ...
pub check.KNOWN_ATTRS := ...  ## The attributes the bootstrap knows (test.attr.1).
pub check.STD_MODULES := ...  ## The standard modules (std.use.1), sorted.
pub check.AIL_STD_MODULES := ...  ## The standard modules written in AIL (std.ui.19).
pub check.PRELUDE_LINE := 1073741824  ## Positions from here on are the prelude's.
pub fn check.is_prelude_line(int) bool
pub fn check.list_names(List[str]) str  ## "`a`, `b` and `c`"
pub fn check.count(int, str) str
pub fn check.was_were(int) str
pub fn check.suggest(str, List[str]) str?  ## The closest of `candidates` to `name`, within a third of its length
pub fn check.edit_distance(str, str) int
pub fn check.foreign_type_word(str) str?  ## A type name from another language and the AIL type for it.
pub fn check.std_module_names() List[str]  ## The names of the standard modules (std.use.1), for the loader's messages.
pub enum check.Ty
pub enum check.Errs  ## The error part of `T ! E`: a written set, or a function's inferred set.
pub enum check.Def  ## A declared struct or enum.
pub struct check.Bind  ## A binding of a type parameter.
pub fn check.sort_errors(List[check.Ty]) List[check.Ty]  ## `ts` sorted by `ty_key`, stable.
pub fn check.subst_all(List[check.Ty], List[check.Bind]) List[check.Ty]
```

trimmed to fit 2000 tokens (~1862); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence)
