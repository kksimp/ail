# 0056. Matches must be exhaustive; a catch-all that hides enum variants is warned about

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 4 — Type system; 5 — Expressions
- **Spec rules:** [expr.match.2–3](../spec/05-expressions.md)

## Context

Exhaustive matching is how the compiler finds every place to update when a
new enum variant is added. That's the most reliable multi-file edit for AI
and humans alike ([ai-author-wishlist.md 2.1](../design/ai-author-wishlist.md)).

## Options considered

### Option A — Always exhaustive; `_` allowed but warned when it hides enum variants (chosen)
- **Pros:** Adding a variant still flags every match that hides it behind `_`.
  No fall-through bugs.
- **Cons:** A few more arms. The warning can be noisy for very large enums
  (it can be suppressed per match).

### Option B — Exhaustive; `_` silences it
- **Pros:** Concise (Rust practice).
- **Cons:** `_` silently absorbs new variants.

### Option C — Not required; unmatched values trap
- **Pros:** The least typing.
- **Cons:** Bugs surface at run time.

## Decision

1. A `match` must cover every possible value of its scrutinee. A
   non-exhaustive match is a compile-time error, and the diagnostic lists the
   missing cases.
2. A catch-all arm `_` is allowed. When the scrutinee is an enum and `_`
   covers one or more named variants, the compiler warns and names them.
   The warning can be suppressed for a specific match (suppression syntax
   is to be designed with attributes, OQ-027).

## Consequences

- The diagnostic's fix inserts arms for the missing variants, with typed
  holes ([ai-author-wishlist.md 2.2](../design/ai-author-wishlist.md)) as
  their bodies.
- Matches over open-ended types (integers, strings) still need `_`, which
  isn't warned about.

## Rejected alternatives

- **Silent `_`:** loses the main benefit exactly where it matters.
- **Non-exhaustive:** runtime failures.
