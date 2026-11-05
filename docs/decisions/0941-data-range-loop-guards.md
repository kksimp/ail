# 0941. Native: data-range guards for loop nests, and ikj interchange of reduction nests

- **Status:** Accepted
- **Date:** 2026-11-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 23 — Implementation (native optimizer)
- **Spec rules:** [native.bce.12, native.bce.13](../spec/23-native.md)
- **Builds on:** [ADR 0577](0577-bounds-checks-in-counted-loops.md) (loop versioning), [ADR 0622](0622-loop-starts-steps-and-index-spans.md) (ranges), [ADR 0890](0890-optimizer-roadmap-to-match-c.md) (optimizer roadmap)

## Context

matmul-1400 on nested lists runs 3.5x slower than its C twin (BIGRACE). The
inner loop `s += a[i][k] * b[k][j]` keeps two overflow checks per pass,
because `ranges.ail` bounds only values held in registers: an element loaded
from a list, and an accumulator, are unbounded. The same checks forbid
reordering the nest (ADR 0890 rejected interchange because trap order would
change). AIL knows what C cannot: a list that no call, `push` or assignment
reaches in the nest can't change while the nest runs (value semantics).

## Decision

1. **Data-range guard** (native.bce.12, `compiler/mir/bce.ail`,
   `ranges.ail`). For an innermost counted loop, each `int` element read from
   a list (or a row of a list of lists) that is unchanged in a loop at least
   two levels around it gets the span +-2^20 (`DATA`). One call to
   `rt.datarange_absmax` (`compiler/rt/native/datarange.ail`) scans the list
   in that outer loop's preheader; the inner loop's guard tests the result
   against `DATA`. Two levels make the scan's cost (one pass over the data)
   small next to the nest's.
2. **Accumulators.** A header parameter that every path back passes
   unchanged or as `p + e` / `p - e`, with `e`'s span known without `p`, gets
   its start's span widened by the trip count times `e`'s. The trip count is
   the constant bound, else a limit `(2^62 - 2^30) / max|e|` (at most 2^30)
   that the guard tests against the bound. A tested limit is used only when
   it proves every checked operation of the loop, since otherwise the test
   could send a large loop to the checked copy that the first pass would
   have relaxed.
3. When a guard fails, the original loop runs, so traps keep their place and
   message (as for every bce guard). `AIL_OPT_OFF=datarange` turns 1 and 2 off.
4. **ikj interchange** (native.bce.13, `compiler/mir/interchange.ail`, run
   before bce). The nest `for j: s := s0; for k: s += t; row.push(s)`, with
   `row` made empty just before it and used nowhere else, `s0` from before
   the `j` loop, and `t` built from `get`s of lists from before the nest
   (indexed by `k`, `j`, a value from before or a constant) and `+ - *` (or
   float operations), becomes: push `nj - lj` copies of `s0`, then
   `for k: for j: row[base + j - lj] += t`. Guard blocks test the bounds of
   every index (row floors by `rt.list_rows_min`), and for `int` the
   data-range scans and a trip limit as in 1-2; then nothing in the swapped
   nest can trap and it has no checks. Each element adds the same terms in
   the same order, so `int` and `float` results are bit-identical: no float
   reassociation (ADR 0246). `AIL_OPT_OFF=interchange` turns it off.

## Rationale

Criterion 4 (speed), at no cost to the others: no syntax, no new semantics,
traps unchanged. The scan is linear in the data and runs once per outer
loop, against a nest that is at least quadratic. 2^20 keeps the products of
two elements below 2^40 and leaves room for a 2^22-pass accumulator, which
covers common matrices and counters; larger data runs the checked loop.

## Rejected alternatives

- **A symbolic guard (`max_a * max_b * n < 2^62` computed at run time).**
  Needs checked or wide arithmetic in the guard itself; the fixed limits
  cover the programs measured with a simpler guard.
- **Scanning before a loop one level up.** For `for i: for j: s += g[i][j]`
  the scan costs as much as the loop.
- **Interchange without a guard.** Changes which trap fires first.
- **Interchange limited to `int`.** Not needed: the swap keeps each element's
  order of additions, so float results don't change.
- **A transposed copy of `b`** (ADR 0890's earlier idea). Allocates n^2 per
  call and still walks two rows per element; the swap needs no memory and
  gives a contiguous inner loop that vectorizes (ADR 0942).
