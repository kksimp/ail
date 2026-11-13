# 1080. One guard test per list for indexes bounded by the counted loops around

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 25 — Optimization strategy
- **Spec rules:** native.bce.10 (amended)
- **Builds on:** [ADR 0816](0816-native-codegen-entry-addressing-constants.md) item 13 (`xs[y + k]`), [ADR 0577](0577-bounds-checks-in-counted-loops.md) (counted loops)

## Context

native.bce.10 versions a loop on `0 <= y` and `y < len - k` for each access
`xs[y + k]` whose `y` the loop doesn't change. N-body's inner `j` loop reads
`s[bi]`, `s[bi + 1]` .. `s[bi + 6]` with `bi = i * 7`, so its guard tests
`bi` once per offset group, and the guard runs once per `i`. The outer loop
`i in 0..5` already bounds `bi` to `0..=28`, so a single test `28 + 6 < len`
covers every access, and it holds for the whole run.

## Options considered

### Option A — use the range of `y` from the counted loops around (chosen)

When `y` is built by `+`, `-` and multiplication by a constant from constants
and counters of enclosing counted loops whose bounds are constants, its range
`lo..=hi` is known. A counter is below its bound everywhere in its loop but
the header, so inside the loop body `hi` is the bound minus one. If `lo >= 0`
and `hi + k` is no more than the existing per-list reach limit, the access
joins the list's constant-index test (`K < len`, `K` the largest such
`hi + k`) instead of adding its own pair of tests.

- **Pros:** about 40 lines in `bce.ail`, no new pass; the guard shrinks to one
  compare per list; the trap path is unchanged (the original loop runs when the
  test fails, and traps at the same access with the same message).
- **Cons:** only constant ranges; a variable `n` keeps the old tests.

### Option B — hoist the guard out of the enclosing loop

- **Pros:** also covers variable bounds.
- **Cons:** needs a guard on the outer loop's own range and a second copy of
  the outer loop; more code than nbody needs before 0.1.0.

## Decision

1. In `version` (bce), an `inv` or `invoff` need whose index is not the
   loop's counter and whose range from the enclosing counted loops is
   `lo..=hi` with `lo >= 0` and `hi + k <= MAX_REACH` becomes a constant-reach
   need `hi + k` on its list; the other needs are unchanged.
2. Ranges: a constant is its value; a counter of a counted loop with constant
   `lo` and bound, seen from inside that loop and outside its header, is
   `lo..=bound - 1`; `+`, `-` and `* constant` combine the ranges. Recursion
   is depth-limited.
3. The spec rule native.bce.10 says the test may be `hi + k < len`, citing
   this ADR.

## Soundness

`lo <= y <= hi` holds at every point the access runs, so `hi + k < len` and
`lo >= 0` imply `0 <= y + k < len`. When the test fails, the unversioned loop
runs and traps exactly where it would have.

## Rejected alternatives

Option B, for the reason above. Tightening only nbody's shape (a multiply by
7) was rejected for the general range rule, which costs no more code.
