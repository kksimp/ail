# compiler: mir

## mir
```
pub fn mir.fold_addresses(mut mir.Func) [mutates]
pub fn mir.fold_prim_addresses(mut mir.Func) [mutates]
pub struct mir.Versioned
pub fn mir.opt_off(str) bool [env]
pub fn mir.bce(mir.Func, bool, bool) mir.Func [env]
pub fn mir.bce_noting(mir.Func, bool, bool, bool, mut List[mir.RkNote]) mir.Func [env mutates]
pub fn mir.lists_only_grow(mir.Module) bool
pub fn mir.fill_loops(mir.Func, bool) mir.Func
pub fn mir.fold(mir.Func) mir.Func
pub fn mir.fold_in(mut mir.Func) [mutates]
pub fn mir.rpo(mir.Func) List[int]
pub fn mir.forward_loads(mir.Func) mir.Func
pub fn mir.hoist_headers(mir.Func) mir.Func [env]
pub fn mir.inline_module(mut mir.Module) [env mutates]
pub fn mir.inline_call(mut mir.Func, int, int, mir.Func, mut List[mir.Site]) [mutates]
pub fn mir.merge_blocks(mir.Func) mir.Func
pub fn mir.merge_blocks_in(mut mir.Func) [mutates]
pub fn mir.interchange(mir.Func) mir.Func [env]
pub fn mir.iv_sums(mir.Func) mir.Func [env]
pub struct mir.Def
pub fn mir.defs_of(mir.Func) List[mir.Def]
pub fn mir.inst_of(mir.Func, List[mir.Def], int) mir.Inst?
pub fn mir.const_of(mir.Func, List[mir.Def], int) int?
pub struct mir.LoopInfo
pub fn mir.find_loops(mir.Func) List[mir.LoopInfo]
pub fn mir.is_innermost(List[mir.LoopInfo], mir.LoopInfo) bool
pub struct mir.Counted
pub fn mir.counted(mir.Func, List[mir.Def], mir.LoopInfo) mir.Counted?
pub struct mir.Sig
pub struct mir.Problem
pub error struct mir.Problems
pub impl ToStr for mir.Problems
pub fn mir.lower_module(str, str, ast.Program, ast.Program?) mir.Module ! mir.Problems [env fail fs io]
pub struct mir.Source
pub fn mir.lower_package(List[mir.Source], ast.Program?) mir.Module ! mir.Problems [env fail fs io]
pub fn mir.lower_package_with(List[mir.Source], ast.Program?, int) mir.Module ! mir.Problems [env fail fs io]
pub fn mir.lower_package_par(List[mir.Source], ast.Program?, int, Set[str]) mir.Module ! mir.Problems [env fail fs io]
pub fn mir.expr_pos(ast.Expr) ast.Pos
pub mir.RT_GLOBALS := 72
pub struct mir.ValDesc
pub struct mir.Instr
pub mir.STD_AIL := ...
pub fn mir.aliases_of(ast.Program) Map[str, str]
pub fn mir.imports_of(ast.Program) List[str]
pub fn mir.prelude_text(str) str
pub struct mir.Probe
pub struct mir.TaskState
pub fn mir.new_task_state() mir.TaskState
pub fn mir.uses_tasks(str) bool
pub fn mir.is_wait_call(ast.Expr) bool
pub fn mir.plain_name(str) str
pub struct mir.Traits
pub fn mir.new_traits() mir.Traits
pub fn mir.memoize(mut mir.Module) [mutates]
pub enum mir.Ty
pub struct mir.Inst
pub fn mir.is_push_name(str) bool
pub fn mir.push_bytes(str) int
pub fn mir.inst(int, str, List[int]) mir.Inst
pub struct mir.Term
pub fn mir.jump_to(int, List[int]) mir.Term
pub fn mir.branch_on(int, int, int) mir.Term
pub fn mir.switch_on(int, List[int], int) mir.Term
pub fn mir.return_of(int) mir.Term
pub fn mir.unreachable_term() mir.Term
pub struct mir.Block
pub struct mir.Func
pub struct mir.Site
pub fn mir.excerpt(str, int, int) str
pub struct mir.Module
pub fn mir.call_pos(int, int, int) int
pub fn mir.call_file(int) int
pub fn mir.call_line(int) int
pub fn mir.call_col(int) int
pub fn mir.mark_nonneg(mir.Func) mir.Func
pub fn mir.rewrite_nonneg(mir.Func) mir.Func
pub fn mir.unroll_self(mut mir.Func, mut List[mir.Site]) bool [env mutates]
pub fn mir.unroll_self_upto(mut mir.Func, mut List[mir.Site], int) bool [env mutates]
pub mir.PRELUDE_EMBED := ...
pub fn mir.pure_calls(mut mir.Module) [mutates]
pub fn mir.keep_lengths(mir.Func) mir.Func
pub mir.LIMIT := 1073741824
pub mir.DATA := 1048576
pub struct mir.Proven
pub fn mir.prove(mir.Func, List[mir.Def], mir.LoopInfo, mir.Counted, List[int], List[bool], Map[int, int]) mir.Proven [env]
pub fn mir.mark_recursion(mut mir.Module) [mutates]
pub struct mir.RkBlocker
pub struct mir.RkNote
pub fn mir.rk_target() str [env]
pub fn mir.rk_matches(str, str) bool
pub fn mir.rk_key(mir.Func, mir.LoopInfo) int
pub fn mir.rk_uncounted(mir.LoopInfo) str
pub fn mir.rk_blockers(mir.Func, List[mir.Def], mir.LoopInfo) List[mir.RkBlocker]
pub fn mir.rk_explain(str, mir.Func, mir.Func, mir.Func, List[mir.RkNote], List[mir.Site], List[str], List[mir.Func]) List[str] [env]
pub fn mir.reuse_lists(mut mir.Module) [mutates]
pub fn mir.simplify(mir.Func) mir.Func
pub fn mir.simplify_in(mut mir.Func) [mutates]
pub fn mir.sret_convert(mut mir.Module) [mutates]
pub fn mir.sroa_in(mut mir.Func) bool [mutates]
pub mir.STACK_MAX_BYTES := 128
pub mir.STACK_MAX_WORDS := 24
pub fn mir.st_is_struct(mir.Ty) bool
pub fn mir.st_call_borrows(Map[str, List[bool]], str, int) bool
pub fn mir.borrow_summaries(mir.Module) Map[str, List[bool]]
pub fn mir.stackify(mut mir.Module) [mutates]
pub fn mir.subset_help(str) str
pub fn mir.module_text(mir.Module) str
pub fn mir.func_text(mir.Func) str
pub fn mir.vec_loops(mir.Func) mir.Func [env]
pub fn mir.verify(mir.Func) List[str]
pub fn mir.dominators(mir.Func) List[List[bool]]
```

trimmed to fit 2000 tokens (~1854); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence), fields, variants and test names
