# 1323. Small counted loops that carry an accumulator are unrolled

- **Status:** Accepted
- **Date:** 2026-11-28
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer made ail-llm speed top priority (kksimp/ail#1598, item 2)
- **Topic:** 25 — Optimization strategy
- **Spec rules:** [native.unroll.1](../spec/23-native.md)
- **Builds on:** [ADR 0577](0577-bounds-checks-in-counted-loops.md) (bounds checks in counted loops)

## Context

ail-llm's row kernel sums a handful of products per output in a short inner loop with constant bounds. Each pass
paid for the counter, the compare and the branch, and the counter-dependent shifts and addresses stayed in the loop.
A hand-unrolled version ran 103 ms against 140 ms.

## Decision

A counted loop from a constant to a constant by 1 that runs at most 16 passes, carries an accumulator, is
innermost, leaves only from its header, has one path back, does not `return`, and has `passes * instructions` at
most 400 becomes that many copies of its body with the counter a constant in each (`compiler/mir/unrollloop.ail`).
Checks and trap sites stay in order, so results and traps are the loop's; the later bounds and forwarding passes then
fold the constant indices. Loops that carry nothing are left to `native.vec.4`. `AIL_OPT_OFF=unrollloop` keeps the
loop. The row kernel runs 103 ms, as the hand-unrolled one. The `tests/native/bce.mir` golden changes because
`total([1, 2, 3], 3)` is now inlined and fully unrolled; its checks still go, by constant folding instead of
versioning.

## Rejected alternatives

- **Partial unrolling by 4 of any loop.** Adds a remainder loop and code size for loops whose count is unknown; the
  measured kernels have constant counts.
- **A larger limit than 16 passes or 400 instructions.** Grows code with no measured gain.
