# compiler: mir

## mir
```
pub fn mir.fold_addresses(mut mir.Func) [mutates]
pub fn mir.fold_prim_addresses(mut mir.Func) [mutates]
pub struct mir.Versioned
pub fn mir.bce(mir.Func, bool, bool) mir.Func
pub fn mir.lists_only_grow(mir.Module) bool
pub fn mir.fill_loops(mir.Func, bool) mir.Func
pub fn mir.fold(mir.Func) mir.Func
pub fn mir.fold_in(mut mir.Func) [mutates]
pub fn mir.rpo(mir.Func) List[int]
pub fn mir.forward_loads(mir.Func) mir.Func
pub fn mir.inline_module(mut mir.Module) [mutates]
pub fn mir.merge_blocks(mir.Func) mir.Func
pub fn mir.merge_blocks_in(mut mir.Func) [mutates]
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
pub fn mir.expr_pos(ast.Expr) ast.Pos
pub mir.RT_GLOBALS := 64
pub struct mir.Instr
pub mir.STD_AIL := ...
pub fn mir.aliases_of(ast.Program) Map[str, str]
pub fn mir.imports_of(ast.Program) List[str]
pub fn mir.prelude_text(str) str
pub struct mir.TaskState
pub fn mir.new_task_state() mir.TaskState
pub fn mir.uses_tasks(str) bool
pub fn mir.is_wait_call(ast.Expr) bool
pub fn mir.plain_name(str) str
pub struct mir.Traits
pub fn mir.new_traits() mir.Traits
pub enum mir.Ty
pub struct mir.Inst
pub fn mir.inst(int, str, List[int]) mir.Inst
pub struct mir.Term
pub fn mir.jump_to(int, List[int]) mir.Term
pub fn mir.branch_on(int, int, int) mir.Term
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
pub mir.PRELUDE_EMBED := ...
pub fn mir.pure_calls(mut mir.Module) [mutates]
pub fn mir.keep_lengths(mir.Func) mir.Func
pub mir.LIMIT := 1073741824
pub struct mir.Proven
pub fn mir.prove(mir.Func, List[mir.Def], mir.LoopInfo, mir.Counted, List[int], List[bool]) mir.Proven
pub fn mir.mark_recursion(mut mir.Module) [mutates]
pub fn mir.reuse_lists(mut mir.Module) [mutates]
pub fn mir.simplify(mir.Func) mir.Func
pub fn mir.simplify_in(mut mir.Func) [mutates]
pub fn mir.subset_help(str) str
pub fn mir.module_text(mir.Module) str
pub fn mir.func_text(mir.Func) str
pub fn mir.verify(mir.Func) List[str]
pub fn mir.dominators(mir.Func) List[List[bool]]
```

trimmed to fit 2000 tokens (~1193); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence), fields, variants and test names
