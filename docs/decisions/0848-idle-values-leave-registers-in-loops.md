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

## Amendment 2026-11-21: one store, loads only where read

Decided by: AI (delegated, ADR 0076), lane B3 of the post-0.2 plan.

A value is never redefined and its hole slot is its own, so the slot stays
valid from the first store on. The stores and loads of decision 1 change
(`place_holes` in `compiler/codegen/alloc.ail`; both back ends unchanged):

- **One store after the definition.** When the block that defines the value
  sits in no loop that the blocks before its holed loops are not in, the value
  is stored once at the end of that block, not at the end of each block before
  a loop. Otherwise the blocks before the loops store it, as before.
- **Loads only where read.** A load at the start of a block after the loop is
  kept only when some path from there reads the value before the next load of
  it or a new definition. A value that waits across the next loop too is not
  loaded between the two loops, and a value with no read after any of its
  loops gets no slot. This also drops a load of a value that is dead at an
  exit, whose register another value may hold there.

fannkuch on ail-linux before the change: 7 stores and 7 loads around each of
the copy's vector loops and 6 of each around the flips, about 40 memory
operations a permutation (`/mnt/project-files/b1-gaps/fannkuch.md`). Test
`hole_moves`.

Rejected: an availability analysis to drop repeated stores at the blocks
before loops; the store after the definition removes them in every case
measured, and the rest keeps the old stores.
