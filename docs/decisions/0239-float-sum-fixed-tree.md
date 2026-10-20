# 0239. Float `sum()` adds in a fixed four-lane tree

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 4 — Type system; 21 — Standard library; 25 — Optimization strategy
- **Spec rules:** [type.coll.12](../spec/04-types.md) (amended)
- **Amends:** [ADR 0136](0136-list-methods-and-slices.md) (float `sum` was a left fold)

## Context

ADR 0136 defined float `xs.sum()` as a left-to-right fold. That order can't
be vectorised or parallelised without changing bits, and conc.par.1 (ADR
0236) requires parallel results to equal sequential ones. A summation order
fixed by the *length* alone can be computed the same way by scalar, SIMD and
multi-core code.

## Options considered

### Option A — Fixed tree: leaves of up to 32 with four running sums (chosen)
- **Pros:** One AVX2 register or two NEON registers compute a leaf exactly; leaves combine pairwise, so cores split at subtree boundaries. Pairwise summation's error grows with log n instead of n.
- **Cons:** Results differ from ADR 0136's left fold for lists of four or more elements.

### Option B — Keep the left fold
- **Pros:** Matches a Python loop.
- **Cons:** `sum()` stays single-core and scalar forever.

### Option C — Compensated (Kahan/Neumaier) summation, like Python 3.12
- **Pros:** Most accurate.
- **Cons:** About 4× the work per element, and still order-dependent unless the order is also fixed.

### Option D — Leaves summed left to right
- **Cons:** A vector unit can't compute a left-fold leaf without transposes.

## Decision

For `List[float]`, `S(xs)` is: if `len ≤ 32`, four partial sums `p0..p3`,
`pk` the left-to-right sum from `0.0` of the elements whose index is
`≡ k (mod 4)`, and `S = (p0 + p1) + (p2 + p3)`; otherwise
`S(xs[..m]) + S(xs[m..])` with `m = 32 × ⌈len / 64⌉`. Integer `sum()` is
unchanged (exact, traps at the first overflowing prefix).

## Rationale

Criterion 4 (speed): `sum()` becomes vector- and multi-core-ready with
identical bits everywhere. Tokens and AI understanding are unchanged (same
call). The cost is a change from "matches a Python loop"; lists of up to
three elements give the same result as before.

## Consequences

- The interpreter implements the tree (`interp/builtins.rs` `tree_sum`).
- Hand-written float `+=` loops stay sequential; the checker's help points to `sum()`.

## Rejected alternatives

- **Left fold:** blocks SIMD and parallel `sum()`.
- **Compensated:** too slow for the default, and still needs a fixed order.
- **Left-fold leaves:** not vector-friendly.
