# compiler
entry: main in main.ail

## (root) (10 items)
## (root)
```
fn main() ! [env fail fs io]
fn check_dump(str, bool) ! [env fail fs io]
fn runtime(str, str) !ast.Program [fail fs io]
fn runtime_with(str, str, str, str) !ast.Program [fail fs io]
fn source_text(str) !str [fail fs io]
fn executable_hex(mir.Module, str, str) str [io]
fn runtime_syntax(str, str) [io]
fn lower_file(str, ast.Program?, str) !mir.Module [fail fs io]
fn std_file(str, str) str
fn uses_regex(ast.Program) bool
fn parse_source(str, str) !mir.Source [fail fs io]
```

## a64 (58 items)

## ast (129 items)

## check (304 items)

## codegen (44 items)

## mir (290 items)
## codegen
```
pub codegen.FLOAT_BASE := 32
pub codegen.SLOT_BASE := 1000
pub codegen.REMAT := -2
pub codegen.NOWHERE := -1
pub struct codegen.Target
pub struct codegen.Alloc
pub fn codegen.is_remat(mir.Inst) bool
pub fn codegen.is_call(mir.Inst) bool
pub fn codegen.is_push(mir.Inst) bool
pub fn codegen.caller_used(codegen.Alloc, codegen.Target) List[int]
pub fn codegen.allocate(mir.Func, codegen.Target) codegen.Alloc
pub struct codegen.Reloc
pub struct codegen.Image
pub fn codegen.new_image() codegen.Image
pub fn codegen.add_str(mut codegen.Image, str, str) [mutates]
pub fn codegen.add_f64(mut codegen.Image, float) str [mutates]
pub fn codegen.import_slot(mut codegen.Image, str) int [mutates]
pub fn codegen.align(mut List[int], int) [mutates]
pub fn codegen.push64(mut List[int], int) [mutates]
pub fn codegen.utf8(str) List[int]
pub fn codegen.add_trap_texts(mut codegen.Image, mir.Module, int, str) [mutates]
pub fn codegen.split_edges(mir.Func) mir.Func
pub fn codegen.block_order(mir.Func) List[int]
pub fn codegen.f64_bits(float) int
pub fn codegen.split_for_calls(mir.Func) mir.Func
pub fn codegen.wrap_block(mir.Func, List[int], List[List[bool]], List[int]) int
pub fn codegen.div_magic(int) List[int]
```
## mir
```
pub fn mir.fold_addresses(mir.Func) mir.Func
pub fn mir.fold_prim_addresses(mir.Func) mir.Func
pub struct mir.Versioned
pub fn mir.bce(mir.Func, bool, bool) mir.Func
pub fn mir.lists_only_grow(mir.Module) bool
pub fn mir.fold(mir.Func) mir.Func
pub fn mir.rpo(mir.Func) List[int]
pub fn mir.forward_loads(mir.Func) mir.Func
pub fn mir.inline_module(mir.Module) mir.Module
pub fn mir.merge_blocks(mir.Func) mir.Func
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
pub fn mir.lower_module(str, str, ast.Program, ast.Program?) mir.Module ! mir.Problems [fail]
pub struct mir.Source
pub fn mir.lower_package(List[mir.Source], ast.Program?) mir.Module ! mir.Problems [fail]
pub fn mir.expr_pos(ast.Expr) ast.Pos
pub mir.RT_GLOBALS := 64
pub fn mir.aliases_of(ast.Program) Map[str, str]
pub fn mir.imports_of(ast.Program) List[str]
pub struct mir.TaskState
pub fn mir.new_task_state() mir.TaskState
pub fn mir.uses_tasks(str) bool
pub fn mir.is_wait_call(ast.Expr) bool
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
pub fn mir.keep_lengths(mir.Func) mir.Func
pub mir.LIMIT := 1073741824
pub struct mir.Proven
pub fn mir.prove(mir.Func, List[mir.Def], mir.LoopInfo, mir.Counted, List[int], List[bool]) mir.Proven
pub fn mir.reuse_lists(mir.Module) mir.Module
pub fn mir.simplify(mir.Func) mir.Func
pub fn mir.subset_help(str) str
pub fn mir.module_text(mir.Module) str
pub fn mir.func_text(mir.Func) str
pub fn mir.verify(mir.Func) List[str]
pub fn mir.dominators(mir.Func) List[List[bool]]
```

## obj (56 items)

## syntax (99 items)

## x64 (74 items)

trimmed to fit 4000 tokens (~97); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence), fields, variants and test names, signatures (names only)
trimmed to fit 4000 tokens (~3924); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence), fields, variants and test names
