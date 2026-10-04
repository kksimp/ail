# 0061. A `mut` parameter must be marked `mut` at the call site

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 7 — Ownership; 2 — Core syntax
- **Spec rules:** [own.param.1–2](../spec/07-ownership.md)
- **Evidence:** a call-site marker costs about one token per mutating call (cl100k 25 vs 24 on the example)

## Options considered

- **`f(mut x)` (chosen):** every reader of a call sees which arguments can
  change, without opening the function. Con: slightly more typing.
- **Signature only:** one token less, but readers must look up each callee
  (action at a distance).
- **No `mut` parameters:** the purest model, but verbose for in-place algorithms.

## Decision

1. A parameter written `mut p T` may be mutated by the callee. Its mutations
   are visible in the caller's variable after the call.
2. The corresponding argument must be a mutable place (a `mut` binding, or a
   mutable part of one) and must be written with a leading `mut`:
   `shift(mut b, 5.0)`.
3. Parameters without `mut` are read-only in the callee.

## Consequences

- The AI semantic API answers "does this call mutate its arguments?"
  syntactically.
- Mutable parts of a value can be passed: `scale(mut shapes[3], 2.0)`.
  Interaction with exclusivity is OQ-032.

## Rejected alternatives

- **Signature-only marker:** hides mutation from readers of the call.
- **No `mut` parameters:** too verbose for in-place work.
