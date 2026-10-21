# compiler: codegen

## codegen
```
pub codegen.FLOAT_BASE := 32  ## Where a value lives: a register (0-63, targets number vector registers
pub codegen.SLOT_BASE := 1000
pub codegen.REMAT := -2
pub codegen.NOWHERE := -1
pub struct codegen.Target  ## The registers a target lets the allocator use, best first.
    pub int_caller List[int]
    pub int_callee List[int]
    pub float_caller List[int]
    pub float_callee List[int]
pub struct codegen.Alloc
    pub loc List[int]  ## The location of each value.
    pub order List[int]  ## The blocks in layout order.
    pub slots int
    pub saved List[int]  ## The callee-saved registers used, which the prologue saves.
    pub wrap int  ## The block where the callee-saved registers are saved, or -1 for the
    pub region List[bool]
pub fn codegen.cells(f mir.Func) int  ## Whether an instruction's value is rebuilt at each use instead of held.
pub fn codegen.is_remat(i mir.Inst) bool
pub fn codegen.is_call(i mir.Inst) bool  ## Whether an instruction clobbers the caller-saved registers. A `push`
pub fn codegen.is_push(i mir.Inst) bool
pub fn codegen.caller_used(al codegen.Alloc, t codegen.Target) List[int]  ## The caller-saved registers that hold some value of the function.
pub fn codegen.allocate(f mir.Func, t codegen.Target) codegen.Alloc
fn codegen.use_weights(f mir.Func) List[int]  ## The cost of keeping each value in memory: one per definition or use,
struct codegen.Wrap
    wrap int
    region List[bool]
fn codegen.shrink_wrap(f mir.Func, order List[int], s codegen.Scan, bstart List[int], bend List[int], t codegen.Target) codegen.Wrap  ## Where the callee-saved registers can be saved instead of at the entry:
fn codegen.add_range(mut r List[int], from int, to int) [mutates]  ## Append the range `from..=to`, joining it to the last one when they meet
fn codegen.overlaps(a List[int], b List[int]) bool  ## Whether two values' ranges overlap.
fn codegen.liveness(f mir.Func, order List[int]) List[List[bool]]  ## Which values are live on entry to each block (the block's parameters
struct codegen.Scan
    loc List[int]
    active List[int]  ## Values holding a register now.
    slots int
    saved List[int]
    start List[int]
    end List[int]
    ranges List[List[int]]
    calls List[int]
    hints List[List[int]]
    weight List[int]  ## How much a value costs to keep in memory: its definitions and uses,
    weighed bool  ## Whether `weight` has been computed: only a function that runs out of
    func mir.Func
    fn codegen.Scan.crosses(self, v int) bool  ## Whether `v` is live across a call, so it needs a callee-saved register.
    fn codegen.Scan.free(self, r int, v int) bool  ## Whether register `r` is free for all of `v`'s ranges.
    fn codegen.Scan.assign(mut self, v int, float bool, t codegen.Target) [mutates]
    fn codegen.Scan.sole(self, a int, v int) bool  ## Whether `a` is the only active value whose use of its register meets `v`.
    fn codegen.Scan.new_slot(mut self) int [mutates]
pub struct codegen.Reloc  ## A field of the code to patch once addresses are known: `kind` is "rel32"
    pub at int
    pub kind str
    pub sym str
    pub tail int
pub struct codegen.Image
    pub code List[int]
    pub funcs Map[str, int]
    pub relocs List[codegen.Reloc]
    pub data List[int]
    pub data_syms Map[str, int]
    pub globals int  ## The size of the runtime's globals, in 8-byte slots.
    pub entry int  ## The entry point's offset in `code`.
pub fn codegen.new_image() codegen.Image
pub fn codegen.add_str(mut img codegen.Image, sym str, s str) [mutates]  ## A string object (native-abi.md §2) in the read-only data, once per symbol.
pub fn codegen.add_f64(mut img codegen.Image, x float) str [mutates]  ## A float constant in the read-only data; gives its symbol.
pub fn codegen.align(mut bytes List[int], n int) [mutates]
pub fn codegen.push64(mut bytes List[int], v int) [mutates]
pub fn codegen.utf8(s str) List[int]  ## The UTF-8 bytes of a string.
pub fn codegen.add_trap_texts(mut img codegen.Image, m mir.Module, k int, op str) [mutates]  ## The constant texts a trap stub passes to `rt.trap` (native-abi.md §7):
pub fn codegen.split_edges(f mir.Func) mir.Func  ## Split each branch edge that passes arguments: the new block jumps to the
fn codegen.thread_jumps(f mir.Func) mir.Func  ## Retarget each edge into an empty block that jumps on without arguments.
fn codegen.is_forwarder(blk mir.Block) bool
pub fn codegen.block_order(f mir.Func) List[int]  ## The blocks in reverse postorder from the entry.
pub fn codegen.f64_bits(x float) int  ## Float constants in a 64-bit pattern, computed with exact float scaling
pub fn codegen.split_for_calls(f mir.Func) mir.Func  ## Values that cross calls are copied where the calls begin (ADR 0625). The
fn codegen.renamed(m Map[int, int], v int) int
pub fn codegen.wrap_block(f mir.Func, order List[int], dom List[List[bool]], blocks List[int]) int  ## The deepest block, other than the entry, that dominates every block in
pub fn codegen.div_magic(d int) List[int]  ## Division by the constant `d` (3 <= d < 2^31) with a multiply, after
```
