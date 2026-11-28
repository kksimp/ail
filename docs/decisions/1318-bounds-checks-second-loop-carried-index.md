# 1318. Bounds checks dropped for a second index that grows by a constant

- **Status:** Accepted
- **Date:** 2026-11-28
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer made ail-llm speed top priority (kksimp/ail#1598, item 3)
- **Topic:** 25 — Optimization strategy
- **Spec rules:** [native.bce.19](../spec/23-native.md)
- **Builds on:** [ADR 0577](0577-bounds-checks-in-counted-loops.md) (loop versioning with a guard)

## Context

Unpacking loops in ail-llm read `pk[c]`, `pk[c + 1]`, ... where `c` is not the loop counter but a second value
that grows by a constant each pass. `native.bce.1` only proves indices built from the counter, so every read kept
its bounds check and its `c + k` overflow check.

## Decision

A loop-carried `int` `m` that starts at `m0` before a counted loop stepping by 1 and grows by the same constant `S`
(1 to 64) every pass is `m0 + S * j` on pass `j`. Accesses `xs[m + k]` (`k` from 0 to 64) lose their checks
behind one guard before the loop: `0 <= m0`, bound and start within 2^30 of 0, and
`S * (hi - lo - 1) + m0 + k < xs.len()`. A loop that fails the guard runs the checked version and traps where it
did. `AIL_OPT_OFF=ivstep` keeps the checks. Tests `bce_ivstep` and three trap tests (first, middle and last pass).

## Rejected alternatives

- **Rewrite `m` into `m0 + S * j` in the source pass.** Adds a multiply per access where the guard costs one
  comparison per loop.
- **General induction-variable analysis.** Larger and slower to compile; this shape covers the ail-llm loops.
