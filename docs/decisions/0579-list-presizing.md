# 0579. Sizing a list before the loop that fills it

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.bce.3](../spec/23-native.md), native.build.1
- **Builds on:** [ADR 0391](0391-native-backend-optimisations.md) (MIR passes), [ADR 0384](0384-mir-shape-and-native-subset.md) (MIR)

## Context

A list starts with room for four elements and doubles, copying, when it
fills. With no collector yet (the allocator only bumps), every step leaves
the old block behind. Filling 10 elements allocates 4, 8, 16.

## Decision

Before a counted loop (ADR 0577) that pushes to a list defined outside it,
with the push in a block on every path back and in no inner loop, the pass
inserts `rt.list_reserve(xs, lo, hi, per, size)`: room for
`(hi - lo) * per` more elements of `size` bytes (`per` is the number of such
pushes, at most 8; `size` is 1 for `List[bool]`, else 8), capped at 2^20
elements per call. It only changes capacity, which a program can't see.

fannkuch (the permutation copy) and matmul's row builders allocate once.
Loops that reassign the list (`line = []` in fasta) or push conditionally
aren't sized.

## Consequences

- A loop that breaks early wastes the unused room (untouched pages of a
  bump chunk; at most 8 MB per call).
- The bound and the start are evaluated before the loop, as the loop
  itself does.

## Rejected alternatives

- **Start every list larger.** Costs a block per small list, which
  binarytrees and fasta make many of.
- **Grow by 4x while small.** Fewer copies, still a copy per step; reserving
  has none.
- **Size lists in the front end from literal bounds.** Misses `n` from
  `argv`, which is every perfbench program.
