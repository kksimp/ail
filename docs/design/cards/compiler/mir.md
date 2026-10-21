# compiler: mir

## mir
```
pub fn mir.fold_addresses(f mir.Func) mir.Func
pub fn mir.fold_prim_addresses(f mir.Func) mir.Func
pub struct mir.Versioned
    pub f mir.Func
    pub header int
    pub floors Map[str, int]
pub fn mir.bce(f mir.Func, reserve bool, grow_only bool) mir.Func
pub fn mir.lists_only_grow(m mir.Module) bool
pub fn mir.fold(f mir.Func) mir.Func
pub fn mir.rpo(f mir.Func) List[int]
pub fn mir.forward_loads(f mir.Func) mir.Func
pub fn mir.inline_module(m mir.Module) mir.Module
pub fn mir.merge_blocks(f mir.Func) mir.Func
pub struct mir.Def
    pub block int
    pub at int
pub fn mir.defs_of(f mir.Func) List[mir.Def]
pub fn mir.inst_of(f mir.Func, defs List[mir.Def], v int) mir.Inst?
pub fn mir.const_of(f mir.Func, defs List[mir.Def], v int) int?
pub struct mir.LoopInfo
    pub header int
    pub member List[bool]
    pub blocks List[int]
    pub pre int
    pub latches List[int]
    pub exits int
pub fn mir.find_loops(f mir.Func) List[mir.LoopInfo]
pub fn mir.is_innermost(loops List[mir.LoopInfo], lp mir.LoopInfo) bool
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
pub fn mir.counted(f mir.Func, defs List[mir.Def], lp mir.LoopInfo) mir.Counted?
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
pub fn mir.lower_module(path str, text str, prog ast.Program, rt ast.Program?) mir.Module ! mir.Problems [fail]
pub struct mir.Source
    pub module str
    pub path str
    pub text str
    pub prog ast.Program
pub fn mir.lower_package(input List[mir.Source], rt ast.Program?) mir.Module ! mir.Problems [fail]
pub fn mir.lower_package_with(input List[mir.Source], rt ast.Program?, lazy int) mir.Module ! mir.Problems [fail]
pub fn mir.expr_pos(e ast.Expr) ast.Pos
pub mir.RT_GLOBALS := 64
pub fn mir.aliases_of(prog ast.Program) Map[str, str]
pub fn mir.imports_of(prog ast.Program) List[str]
pub fn mir.prelude_text(text str) str
pub struct mir.TaskState
    on bool
    sites List[mir.TaskSite]
    pars List[mir.ParSite]
    rets Map[str, mir.Ty]
    in_chunk bool
    owner str
    accs Set[str]
pub fn mir.new_task_state() mir.TaskState
pub fn mir.uses_tasks(text str) bool
pub fn mir.is_wait_call(e ast.Expr) bool
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
    Ptr(inner mir.Ty)
    Unknown
    Unit
    Atomic(elem mir.Ty)
    Fn(params List[mir.Ty], ret mir.Ty)
    Sized(kind str)
    pub fn mir.Ty.opt_inner(self) mir.Ty?
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
pub fn mir.inst(dst int, op str, args List[int]) mir.Inst
pub struct mir.Term
    pub kind str
    pub cond int
    pub targets List[int]
    pub args List[List[int]]
    pub value int
    pub fn mir.Term.uses(self) List[int]
pub fn mir.jump_to(target int, args List[int]) mir.Term
pub fn mir.branch_on(cond int, yes int, no int) mir.Term
pub fn mir.return_of(value int) mir.Term
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
pub fn mir.excerpt(line str, n int, col int) str
pub struct mir.Module
    pub path str
    pub funcs List[mir.Func]
    pub strs List[str]
    pub sites List[mir.Site]
    pub lines List[str]
    pub files List[str]
    pub file_lines List[List[str]]
    pub fn mir.Module.globals_used(self) int
pub fn mir.call_pos(file int, line int, col int) int
pub fn mir.call_file(pos int) int
pub fn mir.call_line(pos int) int
pub fn mir.call_col(pos int) int
pub fn mir.keep_lengths(f mir.Func) mir.Func
pub mir.LIMIT := 1073741824
pub struct mir.Proven
    pub sites Set[str]
    pub limited List[int]
    pub lows Map[int, int]
    pub highs Map[int, int]
pub fn mir.prove(f mir.Func, defs List[mir.Def], lp mir.LoopInfo, c mir.Counted, order List[int], held List[bool]) mir.Proven
pub fn mir.mark_recursion(m mir.Module) mir.Module
pub fn mir.reuse_lists(m mir.Module) mir.Module
pub fn mir.simplify(f mir.Func) mir.Func
pub fn mir.subset_help(message str) str
pub fn mir.module_text(m mir.Module) str
pub fn mir.func_text(f mir.Func) str
pub fn mir.verify(f mir.Func) List[str]
pub fn mir.dominators(f mir.Func) List[List[bool]]
```

trimmed to fit 2000 tokens (~1858); dropped: private items, doc comments
