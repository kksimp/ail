# 0848. Values idle in a loop leave their registers, and checked products get their own

- **Status:** Accepted
- **Date:** 2026-10-22
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Builds on:** [ADR 0386](0386-linear-scan-register-allocation.md), [ADR 0816](0816-native-codegen-entry-addressing-constants.md)

## Context

Matmul's inner loop (`s += a[i][k] * b[k][j]`) had 14 instructions against 7.2
for C. About eleven values were live in it and ten registers are allocatable.
Each value kept one location for its whole life, so the allocator spilled a
hoisted row pointer that the loop reads every iteration while `a`, `b` and `i`,
which the loop never touches, held registers.

## Decision

1. **Holes for idle values.** When a loop uses a value held in a stack slot, the
   allocator is run again with holes for that loop: each value that is live
   through the loop without a definition or use in it (and, counting the
   values live at the header plus three temporaries, more than the target has
   registers) loses its live range inside the loop. The register is stored to
   a slot at the end of the block before the loop and loaded back at the start
   of each block after it. Both back ends emit the stores and loads through
   `emit_move` (`Alloc.holes`). The loop needs one block before it that only
   jumps in, blocks that follow each other in layout, and exits whose every
   predecessor is in the loop; any other loop is left alone.
2. **A checked `mul`'s result avoids its operands' registers** when another
   register is free. The overflow stub reports both operands, so a product in
   an operand's register went through scratch and cost two moves.

Matmul 115.99M to 100.19M instructions (-13.6%) at size 200. No other program of
the ten changes by more than 0.4% (mandelbrot -0.3%). Without the "loop already
spilled" condition, fannkuch's short loops paid 2.9% for stores and loads.

## Rejected alternatives

- **Holes in every loop above the register limit.** Fannkuch +2.9%: the stores
  and loads outweigh the registers when a loop runs a few iterations.
- **Splitting at MIR level** with copies before and after the loop. Uses after
  the loop inside an enclosing loop would need block parameters threaded
  round it.
- **Inline list data to remove the second dependent load of `b[k][j]`.** A list
  that grows by `push` has its data elsewhere, so generic code still has to
  load the data pointer. It also changes the list ABI (GC kinds, growth).
  Not done; Kaleb chose the allocator change only.
