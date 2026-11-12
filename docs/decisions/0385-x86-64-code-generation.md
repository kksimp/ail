# 0385. x86-64 code generation: the C convention, scratch registers, checks that branch to trap stubs

- **Status:** Accepted; the scratch registers amended by [ADR 1049](1049-x86-64-allocates-rdx-and-r11.md)
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.build.1](../spec/23-native.md), expr.arith.1, expr.arith.3, expr.arith.5, expr.bit.2, type.coll.4, type.num.9
- **Builds on:** [ADR 0384](0384-mir-shape-and-native-subset.md) (MIR),
  [ADR 0404](0404-native-calling-convention-and-value-layouts.md) (C convention, layouts),
  [ADR 0405](0405-native-traps.md) (trap text), [ADR 0005](0005-x86-64-baseline-v3.md) (x86-64-v3)

## Context

Lane 120's second slice turns MIR into x86-64 machine code with an encoder
written in AIL (`compiler/x64/asm.ail`, ADR 0010). Three choices shape every
function the backend emits: which registers are free for values, how a
check that fails reaches the trap text, and how much the instruction
selector does beyond one MIR instruction at a time.

## Decision

1. **Convention.** System V AMD64 for every AIL function (ADR 0404), with one
   addition: **xmm8–xmm13 are callee-saved** inside AIL code. System V has no
   callee-saved vector register, so without it every float live across a call
   (nbody's and spectral norm's inner loops) is spilled. Runtime and program
   functions are compiled by the same backend, so the extra contract holds
   wherever AIL calls AIL; an `extern` call (lane 126) will treat them as
   clobbered.
2. **Registers.** rax, rdx, r10, r11, xmm14 and xmm15 are never allocated:
   they are the code generator's scratch, so no instruction needs the
   allocator's help to find a temporary (`idiv` wants rax and rdx; `syscall`
   clobbers r11). Allocatable: rsi, rdi, r8, r9, rcx (caller-saved), rbx,
   r12–r15 (callee-saved), xmm0–7 and xmm8–13.
3. **Checks.** An overflow check is `jo`, a bounds check one unsigned `cmp`
   plus `jae` against the list's length, a division checks its divisor for 0
   and for -1 (so `MIN / -1` traps and `MIN % -1` is 0, expr.arith.5, instead
   of faulting in `idiv`). Each failing branch goes to a **stub** at the end
   of the function that calls `rt.trap(kind, a, b, head, op, tail)`: the
   kind picks the message, `a` and `b` are the operands, and `head` (path,
   line, column, rule) and `tail` (the source excerpt) are string constants
   built at compile time. The hot path pays one predicted-not-taken branch.
4. **Selection.** One MIR instruction at a time, plus: a comparison whose only
   use is the branch after it is fused into `cmp`/`jcc`; constants are
   rematerialised as immediates where the instruction takes one; division and
   remainder by a power of two are shifts; floats use VEX (AVX) three-operand
   forms; shifts by a register use BMI2 `shlx`/`sarx`, which need no `cl`.
   All of these are in x86-64-v3 (ADR 0005).
5. **Edges.** Branch edges that pass block arguments are split (a block of
   their own), and all moves at an edge or call are resolved as a parallel
   move, breaking cycles through a scratch register.

## Options considered

### Trap reporting
- **Trap record per site (native-abi.md §7 as first drafted).** One pointer
  argument, but the message must be formatted at compile time, so it can't
  contain the operands (`` `9 * 3` overflows `int` `` needs run-time values).
- **Stub calling `rt.trap` with the operands (chosen).** The message is the
  interpreter's byte for byte, with the operands; the excerpt is a constant.
- **`ud2` and a signal handler.** Smallest code, but a handler needs a table
  from addresses to sites and signal setup per OS; no gain for the drag race.

### Scratch registers
- **Allocate everything, reserve per instruction.** More registers for values,
  but `idiv`, `syscall` and shifts need fixed registers, so the allocator
  would need constraints and the code generator fix-up moves. Rejected for
  now: 9 integer and 14 vector registers already hold every inner loop of the
  drag-race programs.

### Callee-saved vector registers
- **Pure System V (no callee-saved xmm).** Rejected: floats live across a call
  would always be spilled, a cost the user can't opt out of (ADR 0076 §3 is
  about cost, so this is the cheaper side).

## Consequences

- Native traps print the interpreter's first line and source excerpt; the
  interpreter's snapshot lines (`at L:C in f(...)`) are not printed
  (native.build.1 allows that).
- `compiler/x64/gen.ail` is the only x86-64-specific file besides the
  encoder; the allocator (ADR 0386) and the runtime are shared with ARM64.

## Rejected alternatives

- Trap record per site (the first draft of native-abi.md section 7): rejected because the message must be formatted at compile time, so it cannot contain the operands in `` `9 * 3` overflows `int` ``, which need run-time values.
- `ud2` with a signal handler: rejected because the handler needs a table from addresses to sites and per-OS signal setup, and gains nothing for the drag race.
- Allocating every register and reserving per instruction: rejected for now, because `idiv`, `syscall` and shifts need fixed registers, so the allocator would need constraints and extra fix-up moves. Nine integer and 14 vector registers already hold every inner loop of the drag-race programs.
- Pure System V with no callee-saved vector registers: rejected, because floats live across a call would always be spilled, a cost the user cannot opt out of (ADR 0076 section 3).
