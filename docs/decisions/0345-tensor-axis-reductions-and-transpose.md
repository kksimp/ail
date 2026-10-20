# 0345. Tensor axis reductions and `transpose`, in the one summation order

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 25 — Optimization strategy; 2 — Targets
- **Spec rules:** [std.tensor.3](../spec/17-std.md), [std.tensor.11–12](../spec/17-std.md)
- **Extends:** [ADR 0328](0328-std-tensor-cpu-api.md), [ADR 0329](0329-std-tensor-kernels-and-dispatch.md), [ADR 0008](0008-isa-priority-policy.md)

## Context

Lane 75's `std.tensor` sums a whole tensor only. Column sums, row maxima,
means and transposes are the next things numeric programs need, and they
must keep ADR 0329's promise: one result on every target and vector unit.

## Options considered

### Option A — `sum(axis)`, `max(axis)`, `mean(axis)`, `transpose()` (chosen)
The axis form drops the axis from the shape; the no-argument form gives a
`T`. Every reduced sequence (the elements along the axis, in index order)
is summed in std.tensor.5's order: sixteen partial sums, halved, then the
tail. When the axis is the last one the sequence is contiguous and the
existing `sum` kernel runs; otherwise the sixteen partial sums are whole
rows of results, added a row at a time with the vector `c += b` kernel,
which is the same order per element, so no gather is needed and the result
is bit-identical (a unit test compares both with the scalar reference).
`max` uses IEEE `>` (AVX2 compare plus blend, NEON `vcgt` plus `vbsl`, not
the `max` instructions, whose NaN rules differ between the two). `mean` is
`float` only. `transpose` is 2-D and copies in 32 x 32 tiles.
- **Pros:** Bit-identical everywhere; vectorised along the contiguous
  dimension for any axis; the axis sum of a row equals `sum()` of it.
- **Cons:** `max` with NaN differs from `float.max` (which uses `cmp`); the
  spec says so.

### Option B — Left-to-right axis sums (rejected)
Simpler, but a column sum would differ from `sum()` of the same column, and
couldn't vectorise without changing results (ADR 0329).

### Option C — N-D `transpose(axes)` (deferred)
Not needed by anything yet; 2-D covers matrices.

### Option D — `max` by `float`'s total order `cmp` (rejected)
Would make NaN the maximum; slower to vectorise identically on both ISAs.
IEEE `>` matches `if x > m then m = x`, the loop an agent would write.

## Decision

Option A, by ADR 0076 criterion 2 (one predictable result, the same as the
reference loop), then criterion 4 (vector kernels on both ISAs).

## Consequences

Reductions over several axes at once, `min`, `argmax`, and N-D transpose
are follow-ups.
