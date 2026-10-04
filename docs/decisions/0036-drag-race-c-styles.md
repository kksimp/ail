# 0036. Each drag race compares AIL against both idiomatic and tuned C

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 28 — Testing strategy
- **Spec rules:** none
- **Resolves:** OQ-025
- **Extends:** [ADR 0030](0030-performance-benchmarks.md)

## Context

The style of the C side of each AIL-vs-C drag race determines what the result
means.

## Options considered

### Option A — Both idiomatic and tuned C per race (chosen)
- **Pros:** Honestly shows both "beats everyday C" and the remaining gap to
  expert C. Credible to outsiders.
- **Cons:** Twice the C code per race.

### Option B — Idiomatic C only
- **Pros:** Cheap.
- **Cons:** Flattering, so the results are easy to dismiss.

### Option C — Tuned C only
- **Pros:** The hardest bar.
- **Cons:** Expensive, and hides the everyday advantage.

## Decision

Every drag race contains an AIL program plus **two** C programs:
- **idiomatic C:** the code a competent C programmer would typically write
  (standard `malloc`/`free`, straightforward data structures);
- **tuned C:** a performance-focused version (a good allocator or arenas,
  careful layout, SIMD where sensible).

Results report AIL against each.

## Consequences

- The harness verifies that all three programs produce identical outputs.
- Each tuned C version documents what tuning was applied.
- This partially restores the "three-bar" idea from OQ-008 at drag-race scale.

## Rejected alternatives

- **One C version:** either flattering or hides the everyday advantage.
