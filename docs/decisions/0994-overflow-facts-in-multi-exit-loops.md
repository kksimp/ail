# 0994. Native: drop overflow checks that bounds and index uses rule out, in loops that leave in more than one place

- **Status:** Accepted
- **Date:** 2026-11-08
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 23 — Implementation (native optimizer)
- **Spec rules:** [native.bce.14](../spec/23-native.md)
- **Builds on:** [ADR 0577](0577-bounds-checks-in-counted-loops.md), [ADR 0622](0622-loop-starts-steps-and-index-spans.md), [ADR 0890](0890-optimizer-roadmap-to-match-c.md)

## Context

On main 7b03e9df (ail-linux, best of 5) lzr runs 4098 ms against gcc -O2's
2264 ms (1.81x); 61% of its time is `find`, whose match loop
`while l < 66 and i + l < n and data[cand + l] == data[i + l]` is 17
instructions per pass, two of them `jo` after `i + l` and `cand + l`. The
range guards (native.bce.7-9) only take counted loops that leave from their
header, and this loop leaves in three places. Scans such as
`while j < n and s[j] != c` have the same shape and are common in parsers,
compressors and text code.

## Options considered

### Option A — facts on every path (chosen)
A pass (`compiler/mir/ivsum.ail`, after `bce`) collects two facts per value:
the edge where `x < y` holds (`x <= 2^63 - 2`) and a checked list index use
(`0 <= x < 2^62`). A checked `+`/`-` goes when a fact dominates it (`x + 1`
after `x < y`; `x +- c`, `c` in 0..2^62, and index + index, after index
uses). In a loop, `a + l` with `a` invariant and `l` stepping by 1 from 0 goes
when a fact on `a + l` holds at the end of every latch: iteration k+1's sum is
iteration k's plus 1. No guard, no loop copy, no runtime cost.
- **Pros:** small (one file, one line in `inline.ail`); covers any loop
  shape; never changes where a trap fires, since only checks that can't fire go.
- **Cons:** keeps the bounds checks; a sum that is bounded only after its
  first use is not covered.

### Option B — version multi-exit loops in bce.ail
Extend `counted` and `version` to loops with several exits and a constant
trip bound (`l < 66`), guarding `0 <= a` and `a + 65 < len` once.
- **Pros:** would also drop the two bounds checks.
- **Cons:** `version` assumes one exit edge; values live out of several exits
  need new joins. Most match attempts end after one or two passes, so a guard
  per loop entry costs about what it saves. Larger and riskier.

## Decision

Option A, rule native.bce.14, opt-out `AIL_OPT_OFF=ivsum`. Only 64-bit `int`
sums count, and runtime (`rt.`) functions are skipped.

## Consequences

- lzr's match loop loses two `jo` per pass; scan loops lose theirs. An index
  `x * c` bounds `x` as well, so vm's `code[pc * 2 + 1]` and `pc += 1` lose theirs.
- Bounds checks in multi-exit loops remain (Option B is still open). Option B
  was later taken by [ADR 1062](1062-version-multi-exit-loops.md).
- Tests: `tests/native/run/ivsum_scan.ail`, and
  `trap_ivsum_unbounded.ail`, where `a + l != n` bounds nothing and the sum
  overflows and traps on the same iteration as before.

## Rejected alternatives

- **Option B now:** more code for a gain the short match attempts mostly eat;
  revisit with a profile of trip counts.
- **Assume list lengths bound every index sum without a use:** unsound, `a`
  can be anything until it is used.
