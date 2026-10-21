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

5. **Range analysis limit 2^62 - 1** (ranges.ail `BIG`, was 2^61): a product
   of two sums of guard-limited values (spectralnorm's `(i + j) * (i + j + 1)`,
   up to 2^62 - 2^31) is now proven not to overflow. Sums of two spans still
   fit the compiler's own `int`.
6. **x86-64 division by a power of two** computes into the result's register.
7. **`x & m == 0` as a branch condition** is one `test` (x86-64, immediate or
   register mask) or `tst` (AArch64, register mask); new encoders `test_ri`
   and `tst_rr` with encoder tests against objdump and llvm-mc.
8. **Loop rotation looks through rebuilt constants**: a loop header that is
   constants and one comparison repeats the test at the jump back.
9. **Branches on `bool` parameters are threaded** (simplify.ail): a block that
   only branches on its one `bool` parameter (what `a or b` lowers to) is
   bypassed: constant edges go straight to the target, a jump passing any other
   value branches on it. Runtime functions such as `rt.alloc` lost their
   `sete`/`movzx`/`test` chains.
10. **x86-64 globals touched only by loads and stores** are addressed
    `[rip + globals + 8k]`; the `lea` is skipped.

## Measurements

collatz 100000: 173.2M to 119.4M instructions (2, 3, 6 and 7); spectralnorm 200: 46.7M to 37.1M (5, 6). fannkuch 8:
22.4M to 21.7M (1 and 4).

## Rejected alternatives

- **Saving caller-saved registers around calls** so that values live across
  a call stay in registers (tried, ~150 lines in the allocator and both
  generators): no perfbench program got faster. fannkuch's outer-loop scalars
  spill because the inner loops use every allocatable register, not because
  of calls; the code was dropped.
