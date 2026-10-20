# 0213. `ail api` shows contracts as part of the signature

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** [contract.api.1](../spec/20-contracts.md)

## Context

An AI reads a program through `ail api` ([ADR 0150](0150-ail-api-outline.md))
without its bodies. A contract is what a caller must know, so it belongs in
that view.

## Options considered

### Option A — `requires` and `ensures` arrays of condition text, only when present (chosen)
- **Pros:** the text is exactly what the source says; functions without
  contracts cost no extra output tokens.
- **Cons:** a consumer must treat a missing key as an empty list.

### Option B — always present, possibly empty
- **Cons:** two keys on every function, mostly empty.

### Option C — conditions as expression trees
- **Cons:** far more tokens; no consumer needs the tree yet.

## Decision

Option A. `ail api --text` prints each condition on its own line under the
signature, as it is written in the source.

## Rationale

Tokens (criterion 1): the outline stays as short as before for code without
contracts. AI understanding: the outline shows what a caller must satisfy.

## Consequences

- References inside conditions appear in `ail api --symbols`.

## Rejected alternatives

- **Always present:** costs tokens on every function.
- **Expression trees:** costs tokens for no current use.
