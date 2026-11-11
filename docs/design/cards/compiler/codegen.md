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
    pub int_args List[int]  ## The registers the first integer and float arguments arrive in.
    pub float_args List[int]
    pub div_regs List[int]  ## The registers a `div` or `rem` overwrites (x86-64 `idiv` and `mul`
pub struct codegen.Alloc
    pub loc List[int]  ## The location of each value.
    pub order List[int]  ## The blocks in layout order.
    pub slots int
    pub saved List[int]  ## The callee-saved registers used, which the prologue saves.
    pub wrap int  ## The block where the callee-saved registers are saved, or -1 for the
    pub region List[bool]
    pub holes List[codegen.Hole]  ## Moves around loops a value is idle in (ADR 0848).
    pub sallocs Map[int, int]  ## For each `salloc`, the slot number (`SLOT_BASE + k`) of the first of
pub struct codegen.Hole  ## A value kept in a stack slot across a loop that never touches it, so the
    pub block int
    pub store bool
    pub reg int
    pub slot int
    pub float bool
pub fn codegen.is_remat(i mir.Inst) bool  ## Whether an instruction's value is rebuilt at each use instead of held.
pub fn codegen.is_call(i mir.Inst) bool  ## Whether an instruction clobbers the caller-saved registers. A `push`
pub fn codegen.is_push(i mir.Inst) bool
pub fn codegen.caller_used(al codegen.Alloc, t codegen.Target) List[int]  ## The caller-saved registers that hold some value of the function.
pub fn codegen.allocate(f mir.Func, t codegen.Target) codegen.Alloc
pub struct codegen.Reloc  ## A field of the code to patch once addresses are known: `kind` is "rel32"
    pub at int
    pub kind str
    pub sym str
    pub tail int
pub struct codegen.CallRow  ## A user call: its return address (an offset in `code`), the position of
    pub off int
    pub pos int
    pub func str
pub struct codegen.Image
    pub code List[u8]
    pub funcs Map[str, int]
    pub relocs List[codegen.Reloc]
    pub data List[u8]
    pub data_syms Map[str, int]
    pub globals int  ## The size of the runtime's globals, in 8-byte slots.
    pub entry int  ## The entry point's offset in `code`.
    pub calls List[codegen.CallRow]  ## Each user call, in address order (trace.frame.1).
    pub imports List[str]  ## The libSystem symbols the code calls (`import_call`, native-abi.md
pub fn codegen.new_image() codegen.Image
pub fn codegen.add_str(mut img codegen.Image, sym str, s str) [mutates]  ## A string object (native-abi.md §2) in the read-only data, once per symbol.
pub fn codegen.f64_sym(x float) str  ## The symbol of a float constant in the read-only data (`add_f64`).
pub struct codegen.Sites  ## The module's sites, shared by reference: a code generator keeps one per
    pub list List[mir.Site]
pub fn codegen.add_f64(mut img codegen.Image, x float) str [mutates]  ## A float constant in the read-only data; gives its symbol.
pub fn codegen.linux_import(sym str) bool  ## Whether `sym` (a `libsys.` name without its prefix) is a libc function a
pub fn codegen.import_slot(mut img codegen.Image, sym str) int [mutates]  ## The slot of import `sym`, added on first use.
pub fn codegen.align(mut bytes List[u8], n int) [mutates]
pub fn codegen.push64(mut bytes List[u8], v int) [mutates]
pub fn codegen.utf8(s str) List[u8]  ## The UTF-8 bytes of a string.
pub fn codegen.add_trap_texts(mut img codegen.Image, m mir.Module, k int, op str) [mutates]  ## The constant texts a trap stub passes to `rt.trap` (native-abi.md §7):
pub fn codegen.add_call_table(mut img codegen.Image, m mir.Module) [mutates]  ## Append the call table the runtime reads to print a trap's stack
pub fn codegen.split_edges(f mir.Func) mir.Func  ## Split each branch or switch edge that passes arguments: the new block
pub fn codegen.block_order(f mir.Func) List[int]  ## The blocks in reverse postorder from the entry.
pub fn codegen.f64_bits(x float) int  ## Float constants in a 64-bit pattern, computed with exact float scaling
pub fn codegen.split_for_calls(f0 mir.Func) mir.Func  ## Values that cross calls are copied where the calls begin (ADR 0625). The
pub fn codegen.wrap_block(f mir.Func, order List[int], dom List[List[bool]], blocks List[int]) int  ## The deepest block, other than the entry, that dominates every block in
pub fn codegen.div_magic(d int) List[int]  ## Division by the constant `d` (3 <= d < 2^31) with a multiply, after
```

trimmed to fit 2000 tokens (~1513); dropped: private items
