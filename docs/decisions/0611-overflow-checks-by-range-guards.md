# 0611. Overflow checks removed by range guards, and smaller codegen wins

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.bce.4](../spec/23-native.md), native.build.1, expr.arith.1
- **Builds on:** [ADR 0577](0577-bounds-checks-in-counted-loops.md) (loop versioning), [ADR 0391](0391-native-backend-optimisations.md) (MIR passes)

## Context

After ADR 0577 the remaining gaps to `gcc -O2` on spectralnorm (1.45x),
mandelbrot (1.42x), collatz (1.45x) and matmul (2.5x) were instruction
overhead: checked `+`, `-` and `*` on loop indices, copies, and
comparisons with 0. Reading the code showed `(i + j) * (i + j + 1) / 2 + i + 1`
with eight `jo` checks, the same `i + j` checked twice, `3 * n` loaded into a
register before the `imul`, and `cmp r, 0`.

## Decision

1. **Range guards** (`compiler/mir/ranges.ail`, used by bce.ail). In a
   counted loop that is being versioned, a forward pass gives each value a
   span: the induction value is `start .. 2^30 - 1`, a constant its own
   value, a value fixed in the loop `-2^30 .. 2^30` (the guard tests this
   once, ahead of the loop), and `+`, `-`, `*`, `neg`, `/` and `%` by a
   positive constant follow by interval arithmetic. A checked operation
   whose result span fits in `int` loses its check in the loop's copy. If a
   guard fails, the original loop runs, so the trap is the interpreter's
   (native.build.1). It applies when two or more checks go, or one in a loop
   that is versioned anyway, and with at most 8 guarded values.
2. **Duplicate checked operations are removed** in CSE (fold.ail): the
   first one dominates and would have trapped.
3. **x86-64:** a constant on the left of `+` or `*` is encoded as the
   immediate (the trap still reports `x OP y` in source order), and a
   comparison with 0 is `test r, r`.
4. **`rt.list_reserve` is a runtime root** (lower.ail): with the runtime
   lowered on demand (ADR 0393) the optimiser's call to it had no function
   behind it, and fannkuch had lost ADR 0579's sizing (383M → 271M
   instructions).
5. **`copy_bytes` copies 32 bytes per iteration** while it can.
6. **Row-length floors.** In a versioned loop, `r[j]` where `r` is a row
   fetched from a list of lists `b` and `j` doesn't change loses its check
   if `0 <= j < floor`, where `floor = rt.list_rows_min(b)`, the length of
   the shortest row. The scan runs once, in the preheader of the outermost
   enclosing loop where `b` is defined outside it and nothing in it gets
   `b`, pushes to it or sets its elements; for as long as no operation
   shortens a list (the same condition as ADR 0578), the rows' lengths can
   only grow, so the floor holds for the whole nest. The inner guard tests
   `j` against it (two compares). A ragged matrix fails the guard and runs
   the checked loop, which traps as before. matmul at 400: 1.11G → 0.98G
   instructions, 0.128 s → 0.098 s.

Instructions at the `.args` sizes, before → after: spectralnorm 88M → 75M,
fannkuch 383M → 271M, collatz 188M → 184M, mandelbrot 44.0M → 43.0M, sieve
134M → 130M. Wall time against `gcc -O2` at the large sizes is in
[native-performance.md](../design/native-performance.md).

## Consequences

- A loop whose bound or fixed operands exceed 2^30 runs the original loop
  after a few compares per entry.
- On ARM64 a constant on the left of `+` uses the immediate form, and a
  comparison with 0 was already `cmp x, #0`. A checked `3 * n` keeps
  `smulh`/`mul` on purpose, not by oversight: A64 has no immediate multiply,
  and `add x, n, n, lsl #1` can't carry the overflow check, since the flags
  reflect only the final add (ADR 0008 asks for each ISA's best correct
  form, not identical instructions).

## Rejected alternatives

- **Scanning the rows in each inner guard.** The scan costs as much as the
  loop it guards (one load and compare per row), so the check saved is paid
  back; hoisting it to the outer nest makes it once per matrix.
- **A "rectangular" flag on the list.** Another header word to maintain on
  every `push` and `set`, for all programs, to help one access pattern.

- **A general value-range lattice over the whole function.** A far larger
  pass for a compiler that runs in the interpreter; loops are where the
  checks cost.
- **Guard on the product instead of the operands.** One compare for the
  result, but it needs the checked operation to compute first, which is the
  check being removed.
- **Four-fold list growth for binarytrees.** 14% fewer instructions and no
  change in wall time (the program is bound by memory), at more waste.
