# compiler: mir

## mir
```
pub fn mir.fold_addresses(mir.Func) mir.Func
pub fn mir.fold_prim_addresses(mir.Func) mir.Func
pub struct mir.Versioned
    pub f mir.Func
    pub header int
    pub floors Map[str, int]
pub fn mir.bce(mir.Func, bool, bool) mir.Func
pub fn mir.lists_only_grow(mir.Module) bool
pub fn mir.fill_loops(mir.Func, bool) mir.Func
pub fn mir.fold(mir.Func) mir.Func
pub fn mir.rpo(mir.Func) List[int]
pub fn mir.forward_loads(mir.Func) mir.Func
pub fn mir.inline_module(mir.Module) mir.Module
pub fn mir.merge_blocks(mir.Func) mir.Func
pub struct mir.Def
    pub block int
    pub at int
pub fn mir.defs_of(mir.Func) List[mir.Def]
pub fn mir.inst_of(mir.Func, List[mir.Def], int) mir.Inst?
pub fn mir.const_of(mir.Func, List[mir.Def], int) int?
pub struct mir.LoopInfo
    pub header int
    pub member List[bool]
    pub blocks List[int]
    pub pre int
    pub latches List[int]
    pub exits int
pub fn mir.find_loops(mir.Func) List[mir.LoopInfo]
pub fn mir.is_innermost(List[mir.LoopInfo], mir.LoopInfo) bool
pub struct mir.Counted
    pub p int
    pub k int
    pub lo int
    pub hi int
    pub q int
    pub bound int
    pub body int
    pub exit int
    pub exit_edge int
    pub step int
pub fn mir.counted(mir.Func, List[mir.Def], mir.LoopInfo) mir.Counted?
pub struct mir.Sig
    pub params List[mir.Ty]
    pub ret mir.Ty
    pub fails bool
pub struct mir.Problem
    pub rule str
    pub pos ast.Pos
    pub message str
pub error struct mir.Problems
    pub items List[mir.Problem]
    pub files List[str]
pub impl ToStr for mir.Problems
    pub fn to_str(self) str
pub fn mir.lower_module(str, str, ast.Program, ast.Program?) mir.Module ! mir.Problems [fail]
pub struct mir.Source
    pub module str
    pub path str
    pub text str
    pub prog ast.Program
pub fn mir.lower_package(List[mir.Source], ast.Program?) mir.Module ! mir.Problems [fail]
pub fn mir.lower_package_with(List[mir.Source], ast.Program?, int) mir.Module ! mir.Problems [fail]
pub fn mir.expr_pos(ast.Expr) ast.Pos
pub mir.RT_GLOBALS := 64
pub fn mir.aliases_of(ast.Program) Map[str, str]
pub fn mir.imports_of(ast.Program) List[str]
pub fn mir.prelude_text(str) str
pub struct mir.TaskState
    on bool
    sites List[mir.TaskSite]
    pars List[mir.ParSite]
    rets Map[str, mir.Ty]
    in_chunk bool
    owner str
    accs Set[str]
pub fn mir.new_task_state() mir.TaskState
pub fn mir.uses_tasks(str) bool
pub fn mir.is_wait_call(ast.Expr) bool
pub struct mir.Traits
    impls Map[str, List[str]]
    dispatch Map[str, List[str]]
    mut_self Set[str]
    recv_path List[str]
    recv_mut bool
    stmt_pos ast.Pos?
pub fn mir.new_traits() mir.Traits
pub enum mir.Ty
    Int
    Float
    Bool
    Str
    Bytes
    Byte
    Char
    List(elem mir.Ty)
    Map(key mir.Ty, val mir.Ty)
    Set(elem mir.Ty)
    Opt(inner mir.Ty)
    Struct(name str)
    Enum(name str)
    Tensor(elem mir.Ty)
    F32
    Tuple(elems List[mir.Ty])
    Any(trait_name str)
    Ref(inner mir.Ty)
    Unknown
    Unit
    Atomic(elem mir.Ty)
    Chan(elem mir.Ty)
    Fn(params List[mir.Ty], ret mir.Ty)
    Sized(kind str)
    pub fn mir.Ty.opt_inner(self) mir.Ty?
    pub fn mir.Ty.is_ref(self) bool
    pub fn mir.Ty.is_float(self) bool
    pub fn mir.Ty.is_unit(self) bool
    pub fn mir.Ty.is_unknown(self) bool
    pub fn mir.Ty.elem(self) mir.Ty?
    pub fn mir.Ty.is_sized(self) bool
    pub fn mir.Ty.is_tensor(self) bool
    pub fn mir.Ty.is_coll(self) bool
    pub fn mir.Ty.name(self) str
pub struct mir.Inst
    pub dst int
    pub op str
    pub args List[int]
    pub imm int
    pub fimm float
    pub name str
    pub site int
    pub fn mir.Inst.elem_off(self) int
    pub fn mir.Inst.is_pure(self) bool
pub fn mir.inst(int, str, List[int]) mir.Inst
pub struct mir.Term
    pub kind str
    pub cond int
    pub targets List[int]
    pub args List[List[int]]
    pub value int
    pub fn mir.Term.uses(self) List[int]
pub fn mir.jump_to(int, List[int]) mir.Term
pub fn mir.branch_on(int, int, int) mir.Term
pub fn mir.return_of(int) mir.Term
pub fn mir.unreachable_term() mir.Term
pub struct mir.Block
    pub params List[int]
    pub insts List[mir.Inst]
    pub term mir.Term
pub struct mir.Func
    pub name str
    pub params List[int]
    pub ret mir.Ty
    pub blocks List[mir.Block]
    pub vals List[mir.Ty]
    pub names List[str]
pub struct mir.Site
    pub rule str
    pub kind str
    pub op str
    pub line int
    pub col int
    pub func str
    pub file int
    pub from int
pub fn mir.excerpt(str, int, int) str
pub struct mir.Module
    pub path str
    pub funcs List[mir.Func]
    pub strs List[str]
    pub sites List[mir.Site]
    pub lines List[str]
    pub files List[str]
    pub file_lines List[List[str]]
    pub fn mir.Module.globals_used(self) int
pub fn mir.call_pos(int, int, int) int
pub fn mir.call_file(int) int
pub fn mir.call_line(int) int
pub fn mir.call_col(int) int
pub fn mir.mark_nonneg(mir.Func) mir.Func
pub fn mir.rewrite_nonneg(mir.Func) mir.Func
pub fn mir.keep_lengths(mir.Func) mir.Func
pub mir.LIMIT := 1073741824
pub struct mir.Proven
    pub sites Set[str]
    pub limited List[int]
    pub lows Map[int, int]
    pub highs Map[int, int]
pub fn mir.prove(mir.Func, List[mir.Def], mir.LoopInfo, mir.Counted, List[int], List[bool]) mir.Proven
pub fn mir.mark_recursion(mir.Module) mir.Module
pub fn mir.reuse_lists(mir.Module) mir.Module
pub fn mir.simplify(mir.Func) mir.Func
pub fn mir.subset_help(str) str
pub fn mir.module_text(mir.Module) str
pub fn mir.func_text(mir.Func) str
pub fn mir.verify(mir.Func) List[str]
pub fn mir.dominators(mir.Func) List[List[bool]]
```

trimmed to fit 2000 tokens (~1928); dropped: private items, doc comments, parameter names
