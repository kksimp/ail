# 0261. The checker records locals and expression types only when `ail api` asks

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** api.loc.1, api.type.1 ([semantic-api.md](../design/semantic-api.md))

## Context

The queries need facts only the checker has. Normal `check`, `run` and `test` must not slow down. Criterion: **speed**, as a tie-breaker after AI understanding.

## Options considered

### Option A — a `record` flag; one hook per kind of fact (chosen)
- **Pros:** `check()` is unchanged for callers; the logic lives in `src/api/` and `check/facts.rs`; the final checking pass records once.
- **Cons:** a branch per expression and per scope exit.

### Option B — always record
- **Pros:** simpler.
- **Cons:** allocation on every run.

## Decision

`check::check_recording` runs the final pass with recording on and returns `Resolved.facts` (`check/facts.rs`). `check()` records nothing.

## Rationale

A keeps the hot path to one predictable branch; the speed check in the PR shows no change.

## Consequences

- Further queries (rename planning, call hierarchy) can add facts the same way.

## Rejected alternatives

- Option B: pays for what only the API uses.
