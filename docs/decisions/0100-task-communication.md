# 0100. Tasks communicate by sending values over channels, or through explicit synchronised types

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 11 — Concurrency; 21 — Standard library
- **Spec rules:** [conc.share.1–3](../spec/11-concurrency.md)

## Deciding criterion

Token usage **ties** between the mainstream options. **AI understanding**
decides: with value semantics (ADR 0060), sending a value is always safe,
and the only shared mutable state is an explicitly synchronised type, which
is visible in its type.

## Decision

1. Values (immutable, or independent copies) may be passed freely to spawned
   tasks and sent between tasks. Under ADR 0060 the receiver gets an
   independent value. Storage sharing makes this cheap.
2. `ref` cells can't cross task boundaries (ADR 0063). Shared mutable state
   uses standard synchronised types: `Mutex[T]`, `Atomic[T]`, and similar.
3. `Channel[T]` is a bounded, typed channel. Sending blocks when the channel
   is full, and receiving blocks when it's empty (both colorless, per ADR 0095).

## Consequences

- Exact APIs (closing channels, `select` across channels, lock poisoning per
  ADR 0047) are designed with the standard library.

## Rejected alternatives

- Unsynchronised shared mutable state between tasks: rejected by the decision, since `ref` cells cannot cross task boundaries (ADR 0063) and data-race-free code is required (ADR 0025). The record gives no measured comparison. (reconstructed 2026-10-21.)
