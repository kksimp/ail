# 0065. Copies are implicit in source and reported by the compiler

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 7 — Ownership; 25 — Optimization; 20 — AI semantic API
- **Spec rules:** [own.copy.1](../spec/07-ownership.md)
- **Resolves:** OQ-033

## Options considered

- **Implicit, and reported (chosen):** clean code, with costs fully visible
  in reports and the AI API. Con: a costly copy doesn't stand out in the source.
- **Explicit `.copy()` when not provably free:** every potential copy is
  visible, but there's more noise, and the line moves whenever the
  analysis changes.
- **Implicit, unreported:** hidden costs. Contradicts brief §20.

## Decision

Copies of values are never written explicitly. The compiler reports every
site where a real (deep) storage copy may occur, marks those on hot paths
(once profile data exists), and exposes them through the AI semantic API.

## Consequences

- Copy sites become part of the performance report, next to allocations,
  bounds checks and overflow checks.
- Editor and language-server hints can annotate possible copy sites inline.
- Assertions such as "this function must not copy" are a candidate
  attribute (OQ-027).

## Rejected alternatives

- **Explicit `.copy()`:** noise, and fragile.
- **Unreported:** violates performance transparency.
