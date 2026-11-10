# 1049. x86-64 allocates rdx and r11

- **Status:** Accepted
- **Date:** 2026-11-10
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Builds on:** [ADR 0386](0386-linear-scan-register-allocation.md), [ADR 0848](0848-idle-values-leave-registers-in-loops.md)
- **Amends:** [ADR 0385](0385-x86-64-code-generation.md) (the scratch registers)

## Context

The x86-64 code generator kept four integer scratch registers (rax, rdx, r10,
r11), leaving ten for the allocator against ARM64's 24. The slow-spots survey
of 2026-11-10 found lzr's chain loop and vm's dispatch loop spilling for want
of them: lzr's loop does six memory operations a register would remove.

## Decision

1. **rdx and r11 are allocatable**, caller-saved, after rsi, rdi, r8, r9 and
   rcx. The generator keeps rax and r10 (and xmm14, xmm15).
2. **Divisions overwrite rdx.** `Target.div_regs` names the registers a `div`
   or `rem` overwrites (`[rdx]` on x86-64, none on ARM64). The allocator keeps
   any value live across a division out of them; the division's operands and
   result may use them. The constant-divisor path copies a dividend held in rdx
   to r10 before its high multiply.
3. **Three operands, two scratch registers.** An instruction that needed three
   scratch registers now uses two:
   - a list element's address is the data pointer in r10 and the index in rax;
     a store whose value is also spilled first takes the address with `lea`
     into r10, then stores through rax (`store_at`);
   - `push` adds one to the length in memory before it stores, so r10 holds
     the data pointer while rax still holds the old length;
   - `atomic_cas` with a spilled new value borrows r11 (rdx when the cell's
     address is r11) on the stack around the `cmpxchg`;
   - the divisor of `idiv` sits in r10, and a remainder by `2^k` is computed in
     rax alone.
4. **Parallel-move cycles between registers use `xchg`**, which needs no
   scratch. A cycle through a stack slot (only block parameters make one, and
   their moves never write rax) still goes through rax.

ARM64 is unchanged apart from the empty `div_regs`: it already has the
registers (ADR 0008's lockstep holds because there is nothing to port). The
loop-depth spill weights the slow-spots plan asks for already exist
(`use_weights` in `compiler/codegen/alloc.ail`: each use counts 8 times per
enclosing loop), so no weight change is made here.

## Consequences

Twelve allocatable integer registers on x86-64. A spilled index with a
spilled stored value costs one `lea` more than before; everything else costs
the same or less. Before and after numbers come from `tools/bench` on
ail-linux and the Mac and are recorded in the pull request.

## Rejected alternatives

- **Save rdx around each division** (`mov r11, rdx` before and after) instead
  of telling the allocator. Two moves on every division in a function that
  uses rdx anywhere, and it needs a third scratch register, which is what
  this decision frees.
- **Also allocate r10, leaving only rax.** Every list access and store through
  a spilled address needs two scratch registers; with one, each would need a
  stack push. Not worth one more register.
- **Spill weights by distance to the conflict** (cost only inside the
  overlapping range). No measurement shows a wrong victim once loop-weighted
  uses and holes are in; left until one does.
