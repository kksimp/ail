# 0212. Conditions are pure, so `--no-contracts` can't change a program

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 8 — Error handling
- **Spec rules:** [contract.decl.3, contract.run.3](../spec/20-contracts.md)

## Context

Release builds want to skip the checks. That is only safe if evaluating a
condition can't do anything a program could observe. The effects lane
([ADR 0163](0163-inferred-effect-sets.md)) already infers every function's
effects.

## Options considered

### Option A — reuse the effect sets; allow only `fail` (chosen)
A condition's own effects, and the closed effect set of every function it
may call, must be empty apart from `fail` (a handled failure is a value).
`mutates` is rejected too: a call with a `mut` argument would change the
caller's variable.
- **Pros:** no new analysis; the diagnostic names the chain
  (`io via noisy -> print`), as `@pure` does.
- **Cons:** `fail`-only conditions must handle the failure with `else`.

### Option B — require `@pure` on every function a condition calls
- **Cons:** more annotations (tokens) for what inference already knows.

### Option C — no purity rule
- **Cons:** `--no-contracts` could change output; checks could have side
  effects a prover can't model.

## Decision

Option A, and `ail run --no-contracts` / `ail test --no-contracts` skip
all checks. Checks are on by default in the bootstrap.

## Rationale

AI understanding: a condition means one thing whether or not it runs.
Tokens: no annotation. Speed: release-like runs pay nothing.

## Consequences

- A future optimising build can default to off, or keep checks the prover
  can't discharge; that choice is open (OQ-060).

## Rejected alternatives

- **`@pure` everywhere:** redundant with inference.
- **No rule:** turning checks off would no longer be safe.
