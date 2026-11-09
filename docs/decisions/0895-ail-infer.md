# 0895. `ail infer`: suggested `requires` and `ensures`

- **Status:** Accepted (built by [ADR 1023](1023-ail-infer-contracts.md), which amends the command's form)
- **Date:** 2026-10-24
- **Decided by:** Designer (2026-10-24, chose the feature on a decision card); the details by AI (delegated, ADR 0076), for integrator review
- **Builds on:** [ADR 0398](0398-record-and-replay.md), [property tests](0199-property-seeds-and-shrinking.md), contracts (wishlist 6.1)
- **Topic:** 20 — AI semantic API

## Context

Contracts help readers and the optimizer (ranges remove bounds checks), but writing them is work that agents skip. Tests and recordings already show the values each function sees.

## Decision

- `ail infer [SYM] [--from tests|replay FILE]` runs the tests, or replays a recording, with call and return value events on (the instrumentation layer of ADR 0871) and collects, per function, the parameter and result values seen.
- It proposes a small set of candidate conditions: numeric ranges, non-empty lists, `result >= 0`, `a < len(xs)`, non-null optionals. A condition is kept only when no seen call violates it and at least 20 calls were seen.
- Output is a patch of `requires`/`ensures` lines in `ail edit` hunk form, each with its call count. Nothing is written; the agent or reviewer applies it, then `ail test` fuzzes the new contracts.
- Planned rule keys: `infer.cand.*`, `infer.out.*`.

Cost when unused (ADR 0868): none. The feature is a separate command or pass, so a program that does not use it gets no extra code, no runtime check and no slower build.

## Consequences

A suggestion is evidence, not proof; the output says so. The optimizer lane may use only contracts that were accepted into the source.

## Rejected alternatives

- **Writing contracts into the source automatically:** guesses become promises the optimizer trusts.
- **Static inference only:** weak for numeric ranges that tests show immediately.
