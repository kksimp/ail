# 0176. Keep name-based local lookup in the interpreter

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Interpreter (Stage 2), [ADR 0032](0032-keep-rust-interpreter-as-oracle.md)
- **Spec rules:** none

## Context

The lane brief proposed resolving locals to slot indices in a pre-pass, on the
expectation that `lookup` (a reverse linear scan comparing names) was a hot spot.

## Options considered

### Option A — keep name-based lookup (chosen)
- **Pros:** the environment stays a simple stack that mirrors the source;
  no resolver that must agree with the checker's scoping rules.
- **Cons:** string compares remain.

### Option B — slot-resolved locals
- **Pros:** removes the scan.
- **Cons:** profiling showed lookup and its string compares were about 3% of
  a tight loop and 5% of `interp.ail` before the other changes; under the
  brief's 10% threshold. A pre-pass would be a second scoping implementation
  in the oracle.

## Decision

Do not resolve locals to slots now. Revisit if a profile after other work shows
lookup above 10%.

## Rationale

ADR 0076 criterion 4 (speed) gains too little; criterion 2 (understanding)
costs more. Strings, lists, maps and structs are already `Rc`-shared, so the
other proposed optimisation (cheap reads) was already in place.

## Consequences

- Plain-local reads and assignments are fast-pathed instead (ADR 0175).

## Rejected alternatives

- **Slots via the checker's `Resolved`:** couples the oracle to checker scoping
  for under 10%.
