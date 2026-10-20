# 0303. `old(e)` in `ensures`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** [contract.ens.3, contract.api.2](../spec/20-contracts.md)

## Context

Lane 28's `ensures` sees parameters as they are at return
([ADR 0210](0210-contract-syntax.md)). For a `mut` parameter or `mut self`
that is the new value, so "the list grew by one" or "the balance went up by
`n`" can't be written ([OQ-060](../open-questions.md) item 1).

## Options considered

### Option A — `old(e)` for any pure expression over the parameters (chosen)
- **Pros:** one word covers `old(n)`, `old(xs.len())` and `old(self.total)`;
  it reads like the condition it sits in; the interpreter evaluates each `e`
  once on entry and binds the value under a hidden name, so nothing is copied
  when checks are off.
- **Cons:** `old` becomes special in `ensures` (it isn't a keyword; outside
  `ensures` it is an unknown name with a dedicated message).

### Option B — `old(x)` only for a bare parameter name
- **Pros:** simplest to check.
- **Cons:** `old(xs.len())` is the common case and would need a copy of the
  whole list; the length is what the condition needs.

### Option C — a snapshot block, `before` lines naming locals
- **Cons:** an extra declaration form; more tokens than `old(...)` at the
  point of use.

## Decision

Option A. In an `ensures` condition `old(e)` is the value `e` had on entry.
It takes exactly one expression (`contract.ens.3`). A trap lists each
`old(e)` among the condition's values under the name `old(e)`. `ail api`
shows the condition as written. The hidden name is the call's source text, so
it can't clash with a user name. `old` inside a string interpolation in a
condition isn't supported.

## Rationale

Criterion 2 (AI understanding): the snapshot is named at the place the
condition reads it, and the trap shows both values. Tokens: `old(e)` adds 3
tokens per use and nothing to functions without it.

## Consequences

- `ensures` on `mut` parameters and `mut self` methods can state change.
- Entry values cost one evaluation per `old` per call, only with checks on;
  `--no-contracts` evaluates none (`contract.run.3`).

## Rejected alternatives

Option B (too weak: forces copies), Option C (more syntax, more tokens).
