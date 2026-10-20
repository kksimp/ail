# 0391. First optimisations in the native backend

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.build.1](../spec/23-native.md), native.build.2, expr.arith.1, expr.arith.3, type.coll.4
- **Builds on:** [ADR 0384](0384-mir-shape-and-native-subset.md) (MIR), [ADR 0386](0386-linear-scan-register-allocation.md) (allocation), [ADR 0390](0390-native-subset-for-the-drag-race.md) (the race)

## Context

The first drag race (ADR 0390, [native-performance.md](../design/native-performance.md))
had native AIL 1.65x to 7x behind `clang -O2`, with no optimisation passes.
Reading the generated code showed where the time went: values spilled
across loops because a live interval had no holes, constants rematerialised
per use, `n % 2 == 0` divided, every `push` was a call, a `List[bool]`
took 8 bytes per element, and small helpers paid a full call.

## Decision

Every pass keeps native.build.1: the executable prints the interpreter's
bytes, and a trap names the interpreter's rule, line and operands.

1. **Allocation with lifetime holes.** A value's interval is a list of
   ranges built per block from liveness, so a register is free in the gaps
   (Wimmer and Franz's linear scan). A call clobbers only the caller-saved
   registers live across it; `push` is not counted as a call.
2. **MIR folding** (`compiler/mir/fold.ail`), between two `simplify` runs:
   constant folding of integer, float and unary operations (never folding
   an operation that would trap, so the trap stays at run time); a division
   or remainder by a constant other than 0 (and -1 for division) loses its
   trap site; `x % 2^k == 0` becomes a mask test; an overflow check on
   `i + c` is dropped where a dominating `i < bound` proves it can't
   overflow; and common subexpressions are shared in dominator order (pure
   operations only, never `len`, which `push` changes).
3. **Inlining** (`compiler/mir/inline.ail`): a call to a function of at
   most 40 instructions that calls nothing is replaced by a copy of its
   blocks, up to 64 per caller, then folded again. A block with one
   predecessor that jumps to it is joined to it. Inlined instructions keep
   their trap sites, so a trap still names the callee's line.
4. **`push` inline.** The fast path stores and bumps the length when there
   is capacity; the slow path saves the live caller-saved registers and
   calls `rt.list_push`.
5. **In-place arithmetic.** `x = x + y` computes into `x`'s register; the
   overflow stub undoes the operation before reporting, so the trap shows
   the operands from before it (expr.arith.1).
6. **`List[bool]` takes one byte per element** (native-abi.md). `List[int]`
   and `List[float]` keep 8.
7. **ELF symbols.** Executables carry `.symtab` with each function's name,
   so `perf`, `gdb` and `objdump` name the code.

Instruction counts at the perfbench sizes fell 13–64% (collatz 326M → 188M,
sieve 367M → 144M, spectralnorm 151M → 95M); the race table is in
native-performance.md.

## Consequences

- At large sizes native AIL is 1.0–1.9x of `gcc -O2` on collatz, sieve,
  spectralnorm, mandelbrot and nbody, and 3–5.6x on fannkuch, matmul,
  binarytrees and fasta, where bounds checks, list allocation and
  callee-saved pushes remain.
- `.mir` dumps (`ail run compiler mir`) now show the optimised MIR.
- Inlining makes functions larger; the 40-instruction limit keeps the
  growth small for the programs measured.

## Rejected alternatives

- **Optimise in the code generators.** Each pass would be written twice
  (x86-64 and ARM64, ADR 0008); in MIR it is written once and both targets
  gain.
- **Graph-colouring allocation.** Better allocation at a compile-time cost
  that the interpreted backend can't yet afford; lifetime holes recovered
  most of the spills.
- **Fold away operations that trap.** Faster, but a program that overflows
  would stop at build time or not at all instead of trapping as the
  interpreter does.
- **Inline everything below a size limit, calls included.** Needs a
  call-graph order and recursion guard; leaf functions cover the hot
  helpers in the race.
- **`List[bool]` as bits.** Smaller still, but every access needs shifts
  and a read-modify-write; bytes keep loads and stores single
  instructions.
