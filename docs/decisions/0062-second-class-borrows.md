# 0062. Borrows are second-class: they can't be stored or returned, so there are no lifetime annotations

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 7 — Ownership/lifetimes
- **Spec rules:** [own.borrow.1–2](../spec/07-ownership.md)
- **Background:** [design/ownership-options.md](../design/ownership-options.md#borrows-only-relevant-to-model-a)

## Options considered

- **Second-class borrows (chosen):** lifetime annotations never exist, there
  are no borrow-checker error loops, and the rules are simple. Con: some
  zero-copy patterns use indices, values, in-place `mut` paths, or `ref` instead.
- **First-class borrows with inferred lifetimes:** more zero-copy power, but
  lifetimes come back into errors, along with type-checker complexity.
- **Explicit lifetimes:** the most tokens and complexity.

## Decision

1. A parameter accesses its argument only for the duration of the call.
2. A borrow can't be stored in a struct, captured past the call, or returned.
   Storing or returning a parameter stores or returns a **value**.
3. The language has no lifetime parameters or annotations.

## Consequences

- Custom iteration, element access and in-place element updates must be
  designed within this rule. Candidate mechanisms include built-in `for`,
  accessor blocks or "projections" (Hylo-style), and `mut` element paths.
  To be designed with collections.
- Closures can't capture borrows that outlive the call. Closure capture
  rules are to be designed.
- Raw pointers in `unsafe` code are outside this rule (unsafe topic).

## Rejected alternatives

- **Inferred lifetimes:** complexity re-enters through error messages.
- **Explicit lifetimes:** contradicts token and simplicity goals.
