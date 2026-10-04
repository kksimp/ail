# 0067. `ref` cells are accessed automatically and are always mutable through any copy

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 7 — Ownership; 2 — Core syntax
- **Spec rules:** [own.ref.3–4](../spec/07-ownership.md)

## Options considered

- **Automatic access, always mutable (chosen):** fewest tokens and one simple
  rule. `ref` stays visible in types and at construction. Con: an immutable
  binding to a ref can see its contents change, which is inherent to `ref`
  and stated by the type.
- **Explicit access (`.get()`, `^`):** more tokens on every access.
- **Read-only and mutable ref kinds:** an extra concept, plus conversions.

## Decision

1. Field access and method calls on a `ref T` value apply to the `T` in the
   cell, with no explicit dereference.
2. The contents of a `ref` cell may be mutated through any copy of the
   reference. A binding's immutability fixes which cell it refers to, not
   the cell's contents.
3. A `ref` cell is created with `ref` followed by the initial value
   (`ref Node(...)`; final spelling subject to the grammar).

## Consequences

- To check whether an expression can mutate shared state, look for `ref`
  in its type. The AI API can answer this directly.
- Read-only sharing of data across tasks uses ordinary immutable values,
  which are shareable without `ref`.

## Rejected alternatives

- **Explicit dereference:** noisy.
- **Two ref kinds:** extra complexity for little gain, since immutable
  values already cover read-only sharing.
