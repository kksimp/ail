# compiler: check

## check
```
pub fn check.expr_pos(ast.Expr) ast.Pos
pub fn check.stmt_pos(ast.Stmt) ast.Pos
pub enum check.Step
pub struct check.Place
pub enum check.ArgKind
pub struct check.Slot
pub fn check.field_slots(List[ast.Field], List[ast.Arg]) List[check.Slot]
pub fn check.op_symbol(ast.BinOp) str
pub fn check.full_name(ast.FnDecl) str
pub fn check.has_attr(ast.FnDecl, str) bool
pub fn check.module_at(ast.Program, ast.Pos) str
pub fn check.hidden(ast.Program, ast.Pos, ast.Pos) bool
pub struct check.Report
pub struct check.Analysis
pub fn check.check_program(ast.Program) check.Report [env io]
pub fn check.analyze(ast.Program, bool) check.Analysis [env io]
pub fn check.check_declarations(ast.Program) List[check.Diag] [env io]
pub enum check.Need
pub struct check.Builtin
pub check.STR_METHODS := ...
pub check.BYTES_METHODS := ...
pub check.CHAR_METHODS := ...
pub check.INT_METHODS := ...
pub check.FLOAT_METHODS := ...
pub check.SIZED_INT_METHODS := ...
pub check.F32_METHODS := ...
pub check.LIST_METHODS := ...
pub check.MAP_METHODS := ...
pub check.SET_METHODS := ...
pub enum check.Cand
pub struct check.TParam
pub struct check.Scoped
pub struct check.ImplInfo
pub struct check.SigParam
pub struct check.Sig
pub struct check.Deferred
pub struct check.Checker
pub enum check.LocalKind
pub struct check.Local
pub struct check.Pending
pub struct check.Moved
pub fn check.new_checker(ast.Program) check.Checker
pub check.COVERED_RULES := ...
pub struct check.Skip
pub fn check.skipped(ast.Program, check.Report) List[check.Skip]
pub struct check.Note
pub struct check.Diag
pub fn check.new_diag(str, ast.Pos, str) check.Diag
pub fn check.shifted(fix.Edit, int) fix.Edit
pub fn check.fix_json(fix.Fix) str
pub struct check.VariantRef
pub struct check.Agree
pub struct check.Binding
pub struct check.Coverage
pub struct check.LocalFact
pub fn check.method_rename(check.Ty, str) str?
pub fn check.method_help(check.Ty, str) str?
pub struct check.NameWord
pub fn check.name_word(str) check.NameWord?
pub struct check.CallHint
pub fn check.call_hint(str, int) check.CallHint?
pub fn check.call_help(str) str?
pub fn check.function_help(str) str?
pub fn check.check_handles(ast.Program) List[check.Diag]
pub fn check.check_moves(ast.Program, List[check.Moved]) List[check.Diag]
pub check.PRELUDE_FNS := ...
pub check.BUILTIN_TYPES := ...
pub check.SPEC_TYPES_NOT_YET := ...
pub check.GENERATABLE := ...
pub check.SIZED := ...
pub check.CONC_TYPES := ...
pub check.KNOWN_ATTRS := ...
pub check.STD_MODULES := ...
pub check.AIL_STD_MODULES := ...
pub check.PRELUDE_LINE := 1073741824
pub fn check.is_prelude_line(int) bool
pub fn check.list_names(List[str]) str
pub fn check.count(int, str) str
pub fn check.was_were(int) str
pub fn check.suggest(str, List[str]) str?
pub fn check.edit_distance(str, str) int
pub fn check.foreign_type_word(str) str?
pub struct check.SigFix
pub fn check.std_module_names() List[str]
pub enum check.Ty
pub enum check.Errs
pub enum check.Def
pub struct check.Bind
pub fn check.sort_errors(List[check.Ty]) List[check.Ty]
pub fn check.subst_all(List[check.Ty], List[check.Bind]) List[check.Ty]
```

trimmed to fit 2000 tokens (~1125); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence), fields, variants and test names
