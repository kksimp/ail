# 1062. Native: version counted loops that leave in more than one place, and drop the checks on `xs[a + i]`

- **Status:** Accepted
- **Date:** 2026-11-11
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 23 — Implementation (native optimizer)
- **Spec rules:** [native.bce.16](../spec/23-native.md)
- **Builds on:** [ADR 0577](0577-bounds-checks-in-counted-loops.md), [ADR 0622](0622-loop-starts-steps-and-index-spans.md), [ADR 0816](0816-native-codegen-entry-addressing-constants.md), [ADR 0994](0994-overflow-facts-in-multi-exit-loops.md) (takes its Option B)

## Context

lzr's match loop `while l < 66 and i + l < n and data[cand + l] == data[i + l]`
was 19 instructions a pass at 7b03e9df against gcc's 7; ADR 0994 removed its two
`jo` but left both bounds checks, because loop versioning (`version` in
`compiler/mir/bce.ail`) took only loops that leave from their header. The same
shape is every scan with an `and` condition, a `break` or a `return` (search
loops, parsers, palindromes). ADR 0994 rejected doing this then because a guard
per loop entry can cost what it saves when most entries run one or two passes.

## Options considered

### Option A — version multi-exit loops, with a cheap guard (chosen)
- `counted_multi` (`loops.ail`) is `counted` without the one-exit test; `counted`
  is unchanged for every other pass. `p < hi` still holds in each body block,
  since every pass enters the body from the header's true edge.
- `version` accepts a loop whose exit blocks are entered only from the loop.
  Each exit block dominates every use after the loop it leads to, so a value
  the loop defines and such a use reads becomes a parameter of that exit block,
  passed by the original loop and the copy. A use no exit block dominates
  rejects the loop.
- A new need, `xs[a + i]` with `a` fixed and `i` the counter (or `xs[a + j]`
  for the falling `j` of a two-pointer loop), guards `0 <= a` and
  `hi <= len - a` (`len - a - 1` for `j`); the sum loses its overflow check too.
  With a constant bound this is one test, `(a | (len - hi - a)) >= 0`, and it is
  computed before the outermost loop that keeps `a` and the list's length, so
  in lzr the `i` half runs once per `find` and each match entry tests only `cand`.
- An `a` used only in such sums is not given a range test of its own, and range
  tests are skipped when every proven site is already covered by a need.
- `AIL_OPT_OFF=mexit` turns multi-exit versioning off.
- **Pros:** small, in the two files that own the logic; covers break, return
  and `and` chains, and two-pointer scans with an early return.
- **Cons:** the guard tests the whole range the bound allows, so a loop that
  would stop early near a list's end runs the checked copy; code size grows by
  one loop copy per versioned loop.

### Option B — full SSA repair for exits that join
Accept uses reached from two exit blocks by adding parameters at their join.
- **Pros:** a few more loops.
- **Cons:** needs general SSA reconstruction; no loop in the benchmarks needs it.

### Option C — guard on the trip the loop really makes
- **Cons:** unknowable before the loop when it leaves early.

## Decision

Option A, rule native.bce.16, opt-out `AIL_OPT_OFF=mexit`.

## Consequences

- lzr's match loop loses both bounds checks and both sums' checks on entries
  far enough from the end of `data`; the entry costs one length load and two
  compares. Measured numbers go in the PR.
- Two-pointer loops were already versioned when they leave only from the header
  (`Counted.q`); with this rule, ones with an early `return` are too.
- Tests: `tests/native/run/bce_multi_exit.ail`, `trap_bce_multi_exit.ail`
  (middle pass), `trap_bce_multi_exit_first.ail` (first pass) and
  `trap_bce_multi_exit_sum.ail` (an overflowing `a + l`), each trapping where
  it does with `AIL_OPT_OFF=mexit`.

## Rejected alternatives

- **Option B:** SSA repair for a case no measured program has.
- **Option C:** not computable.
- **A per-entry guard with two compares per sum (`0 <= a`, `hi <= len - a`):**
  four compares per lzr match entry, the cost ADR 0994 worried about.
