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
pub fn check.proven_loops(ast.Program) List[ast.Pos] [env io]
pub fn check.parallel_rows(ast.Program) List[check.ParRow] [env io]
pub struct check.ParSites
pub fn check.par_sites(ast.Program) check.ParSites [env io]
pub fn check.fork_rows(ast.Program) List[check.ForkRow] [env io]
pub fn check.analyze_denying(ast.Program, bool, List[str]) check.Analysis [env io]
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
pub struct check.FailPath
pub struct check.Checker
pub enum check.LocalKind
pub struct check.Local
pub struct check.Pending
pub struct check.Moved
pub fn check.new_checker(ast.Program) check.Checker
pub struct check.LitState
pub fn check.new_lit_state() check.LitState
pub check.COVERED_RULES := ...
pub struct check.Skip
pub fn check.skipped(ast.Program, check.Report) List[check.Skip]
pub fn check.generator_effects(ast.Program, List[str]) List[str] [env io]
pub struct check.Note
pub struct check.Diag
pub fn check.new_diag(str, ast.Pos, str) check.Diag
pub fn check.shifted(fix.Edit, int) fix.Edit
pub fn check.fix_json(fix.Fix, List[str]) str
pub struct check.Discard
pub fn check.deterministic_deny() List[str]
pub fn check.parse_deny(str) (List[str], str)
pub fn check.parse_allow(str) (List[str], str)
pub fn check.effects_report(ast.Program, bool) str [env io]
pub struct check.VariantRef
pub struct check.Agree
pub struct check.Binding
pub struct check.Coverage
pub struct check.LocalFact
pub fn check.method_rename(check.Ty, str) str?
pub fn check.foreign_kinds() List[str]
pub fn check.rename_of_kind(str, str) str?
pub fn check.method_help(check.Ty, str) str?
pub fn check.help_of_kind(str, str) str?
pub struct check.NameWord
pub fn check.name_word(str) check.NameWord?
pub struct check.CallHint
pub fn check.call_hint(str, int) check.CallHint?
pub fn check.call_help(str) str?
pub fn check.function_help(str) str?
pub struct check.GenCheck
pub fn check.new_gen_check() check.GenCheck
pub fn check.grants_policy(ast.Program, str) str
pub fn check.check_handles(ast.Program) List[check.Diag]
pub struct check.OptFix
pub struct check.WordHelp
pub check.LIST_FN_METHODS := ...
pub fn check.check_moves(ast.Program, List[check.Moved]) List[check.Diag]
pub check.PRELUDE_FNS := ...
pub check.ASSERT_FNS := ...
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
pub fn check.prelude_alt(str) str
pub fn check.list_names(List[str]) str
pub fn check.count(int, str) str
pub fn check.was_were(int) str
pub fn check.suggest(str, List[str]) str?
pub fn check.edit_distance(str, str) int
pub fn check.foreign_type_word(str) str?
pub fn check.check_observe(ast.Program) List[check.Diag]
pub fn check.each_block(ast.Stmt) List[List[ast.Stmt]]
pub struct check.ParRow
pub struct check.ForkRow
pub struct check.ForkSite
pub fn check.stored_never_read_after(List[ast.Stmt], str) bool
pub struct check.SplitSite
pub struct check.ProofRow
pub struct check.Proofs
pub fn check.site_key(str, ast.Pos, ast.Pos) str
pub fn check.ens_key(str, ast.Pos) str
pub fn check.prove_fns(List[ast.FnDecl]) check.Proofs
pub struct check.SigFix
pub struct check.OpenBrace
pub fn check.std_module_names() List[str]
pub struct check.StdGuess
pub fn check.std_guess(str) check.StdGuess?
pub struct check.StdTypeHome
pub fn check.std_type_home(str) check.StdTypeHome?
pub struct check.TextFail
pub fn check.new_text_fail() check.TextFail
pub struct check.TryUse
pub struct check.TextHandler
pub struct check.TextReturn
pub struct check.TextFn
pub fn check.check_tools(ast.Program) List[check.Diag]
pub enum check.Ty
pub enum check.Errs
pub enum check.Def
pub struct check.Bind
pub fn check.sort_errors(List[check.Ty]) List[check.Ty]
pub fn check.subst_all(List[check.Ty], List[check.Bind]) List[check.Ty]
pub struct check.BindDecl
pub check.LIST_VOCAB := ...
pub check.SET_VOCAB := ...
pub check.MAP_VOCAB := ...
```

trimmed to fit 2000 tokens (~1871); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence), fields, variants and test names
