# 1079. Keep elements a loop assigns at an unchanging index in registers

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 25 — Optimization strategy
- **Spec rules:** native.promote.1
- **Builds on:** [ADR 0577](0577-bounds-checks-in-counted-loops.md) (counted loops), [ADR 0625](0625-address-folding.md) (address folding)

## Context

N-body's `advance` updates body `i`'s velocity `s[bi + 3..5]` once per body `j`
in the inner loop. After bounds-check elimination the loop is check-free, but
each of those updates is a load, a subtract and a store per iteration, and the
next iteration reloads the stored value. C compilers keep the three values in
registers. A hand edit that does the same in AIL source took nbody on the Mac
from 948 ms to 865 ms (C: 771 ms), about 45% of the gap to C (f13 lane, 2026-11-12).

## Options considered

### Option A — promote the elements in the MIR loop (chosen)

A pass after address folding finds, in a counted innermost loop of a header
and one body, the elements stored at an index the loop doesn't change, and
carries them in header parameters, loading them before the loop and storing
them on the exit edge. It runs after pairing (native.vec.3, ADR 1066) and
leaves paired loops alone.

- **Pros:** one small file (`compiler/mir/promote.ail`) and one dispatch line;
  works on both targets; no runtime check.
- **Cons:** only the shape it proves: one base, one header and one body,
  everything else unchecked and call-free, other accesses at the counter times
  the same stride.

### Option B — version the loop on a run-time overlap test

- **Pros:** covers indexes the static proof can't relate.
- **Cons:** a second copy of the loop and a guard for a case nbody doesn't
  need; more code before 0.1.0.

### Option C — general memory SSA with store sinking

- **Pros:** the general answer.
- **Cons:** a new IR layer before 0.1.0.

## Decision

1. In a counted loop (native.bce.1) with no loop inside, made of a header and
   one body that jumps back, whose instructions are all pure and without a trap
   site except unchecked 8-byte `ldx`/`stx` of one base defined outside it,
   the offsets stored at the first index `x` defined outside the loop are
   promoted, at most 8.
2. Every other access of the base must be at `p * C` (or `p`) with an offset
   `k'`, where `p` is the counter, stepping by the constant 1 from `x0 + d`
   (`d >= 0`), and `x = x0 * C` (or `x0`, `C = 1`), with `C * d` greater than
   the largest promoted offset minus `k'`. Then on every iteration
   `p * C + k' - (x + k) >= C * d + k' - k > 0`.
3. The rewrite: the block before the loop jumps to a copy of the header's
   instructions on the entry values, which goes straight to the exit when the
   loop runs no iteration, else to a block that loads the elements and enters
   the header with them as new parameters. In the body a load of an element is
   its current value and a store sets it; the back edge passes the values; the
   exit edge goes through a block that stores them back.
4. `AIL_OPT_OFF=promote` turns it off.

## Soundness

- No instruction in the loop can trap, call, or touch memory other than the
  base's elements, and every element access the loop keeps is provably at a
  different address on every iteration (2), so no one can read a promoted
  element while it lives in a register; on exit memory holds each element's
  last stored value, as it does without the pass.
- A loop that runs no iteration neither loads nor stores, so an index that is
  only valid when the loop runs is never touched.
- The counter's step and the index products have no overflow check in the loop
  (no trap site), so they don't wrap; `x0 * C` outside either traps before the
  loop or is exact.

## Rejected alternatives

Option B and C, for the reasons above. Promoting elements of loads only (no
store) was left out: it saves a load, not a store and a reload, and nbody's
loads of `s[bi]` stay cheap next to the division.
