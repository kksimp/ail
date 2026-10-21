# 0622. Versioned loops with computed starts and steps, and index arithmetic bounded by a constant

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.bce.7, native.bce.8, native.bce.9](../spec/23-native.md), native.build.1, type.coll.4, expr.arith.1
- **Builds on:** [ADR 0577](0577-bounds-checks-in-counted-loops.md) (loop versioning), [ADR 0611](0611-overflow-checks-by-range-guards.md) (range guards)

## Context

Counted-loop versioning (ADR 0577) needed a loop `while p < hi` that starts
at a constant and steps by exactly 1. Three common shapes fell outside it,
and kept every bounds and overflow check:

- `for j in i + 1..5`: the start is computed before the loop (nbody's pair
  loop, triangular loops everywhere);
- `while j < n` with `j += s` for an unchanging `s` (the sieve's inner
  loop);
- `xs[b * 7 + k]` with `b < 5`: an index that is not `p + constant`
  (nbody's flat state list, any fixed-width record array).

## Decision

1. **Computed start** (`counted`, loops.ail; bce.ail). The start may be any
   `int` defined outside the loop. The guard tests `0 <= start <= 2^30`
   once; the induction value's span starts at 0. With a negative start the
   original loop runs and traps as before.
2. **Unchanging step** (`Counted.step`). Every path back may add a value `s`
   defined outside the loop instead of the constant 1 (not for the
   two-pointer form). The guard tests `1 <= s <= 2^30`; `p < hi <= len`
   still bounds `xs[p]`, and the `p + s` overflow check goes by the
   range-guard proof (the step is a held `int`). Loops that pre-size lists
   or reserve room (ADRs 0578, 0579) still need step 1 and skip these loops.
3. **Constant bound** (ranges.ail). When the loop's bound is a constant `K`,
   the induction value's span is `start .. K - 1` instead of `start ..
   2^30 - 1`, so `b * 7 + 6` has the span 6..34 and not a guard-sized
   one. Ranges now also return each value's span. In bce.ail, an access
   `xs[e]` (`xs` unchanged by the loop) whose index span is known, starts at
   0 or more and ends at `hi <= 2^24` needs only that `len(xs) > hi`, tested
   once per list with the largest `hi`. The range guards are then required
   (the span used held operands within ±2^30), so a list shorter than
   `hi + 1` or a held operand out of range runs the checked loop.

All three leave the original loop in place for a failed guard, so traps
name the same line and message (native.build.1).

## Measurements (Linux x86-64, `valgrind --tool=lackey` instructions at reduced sizes)

| program | before | after |
|---|---:|---:|
| nbody 100000 | 163.0M | 120.2M |
| sieve 3000000 | 117.8M | 91.3M |

nbody wall (best of 7, 1,000,000 steps): 0.176 s to 0.117 s; gcc -O2 0.085 s
(2.08x to 1.37x).

## Rejected alternatives

- **A general affine range analysis with symbolic bounds** (`len`-relative):
  a larger pass; the constant-bound case covers the fixed-width records that
  show up in the programs measured so far. Left for later.
- **Versioning only constant starts, hoisting `i + 1` into a constant by
  peeling:** loop peeling grows code and doesn't help `while j < n` loops.
- **Allowing negative steps:** a descending loop needs the opposite bound
  test; none of the measured programs has one.
