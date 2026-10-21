# 0623. Branch and codegen peepholes: flipped `not` branches, bit operations into the result register, direct spill moves

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** native.build.1 (output unchanged)
- **Builds on:** [ADR 0611](0611-overflow-checks-by-range-guards.md)

## Decision

1. **`branch (not c)` branches on `c` with the edges swapped** (simplify.ail),
   for both targets; `while not done` was a flip, a `test` and a jump.
2. **x86-64 `and`/`or`/`xor`** compute into the result's register (swapping
   the operands when it is the second one's) instead of `S1` followed by a
   move; constant shifts do too.
3. **x86-64 division by 2** biases with `shr 63` alone (the `sar 63` only
   matters for larger powers).
4. **A move from a register to a spill slot** is one store on both targets,
   not a copy through the scratch register.

## Measurements

collatz 100000: 173.2M to 155.3M instructions (2 and 3). fannkuch 8:
22.4M to 21.7M (1 and 4).

## Rejected alternatives

- **`test r, mask` fused with a preceding `and`:** it needs an AArch64 `tst`
  with a bitmask immediate (a new encoder path) for the same saving; left
  until both targets can do it together (ADR 0008).
