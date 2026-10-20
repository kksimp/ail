# 0290. `ail test` prints the trap snapshot under a failure

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling for agents
- **Spec rules:** test.run.5

## Context

A failed `assert_eq` printed only the two values. The snapshot (ADR 0267) and the failed contract (ADRs 0210–0213) existed for `ail run` but not for `ail test`.

## Options considered

### Option A — print the snapshot under the failure (chosen)
- **Pros:** the same text an agent already reads from `ail run`; JSON gets one more field.
- **Cons:** a few more lines per failure.

### Option B — a flag to show it
- **Pros:** quieter by default.
- **Cons:** the agent needs a second run to see why.

## Decision

When a trap fails a test, print its snapshot lines under the failure, and add `"snapshot"` to the failing `--json` line.

## Rationale

AI understanding: one run gives the cause. Tokens: a failure costs a few lines, saving a rerun.

## Consequences

- Failing-test JSON lines gain a field; existing expectations were updated.
- Property failures keep their shrunk input and carry no snapshot.

## Rejected alternatives

- A flag: costs a rerun on every failure.
