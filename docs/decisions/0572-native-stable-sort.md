# 0572. Native stable sort over a list of indices

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [expr.key.4](../spec/05-expressions.md), [type.coll.11](../spec/04-types.md)
- **Builds on:** [ADR 0571](0571-key-functions-inline.md)

## Context

`sorted` and `sort` are stable (type.coll.11, expr.key.4). The runtime has
no sort.

## Decision

`rt.sort_perm(keys)` in `compiler/rt/native/sort.ail` returns the indices
`0..n` in the stable order of the `int` keys: a bottom-up merge sort
between two buffers, where the left run wins a tie. The caller then builds
the result by indexing the original list, so one routine serves `int`,
`float` and `str` elements, and elements are copied as 8-byte words.
O(n log n), no recursion, one extra buffer.

## Rejected alternatives

- **Sorting the elements and keys together.** Needs a variant per element
  type and a parallel swap of two lists.
- **Quicksort.** Not stable; the indices would need a tie-break, which a
  merge sort gets for free.
