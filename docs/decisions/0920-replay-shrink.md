# 0920. `ail replay --shrink`: the smallest recording that still fails the same way

- **Status:** Accepted
- **Date:** 2026-10-26
- **Decided by:** Designer (2026-10-25, put the agent-feature backlog in order, wishlist 7.7 among it); the details by AI (delegated, ADR 0076), for integrator review
- **Builds on:** [ADR 0910](0910-test-bundle-failure.md) (failure bundles), [ADR 0398](0398-record-and-replay.md) (record and replay), [ADR 0199](0199-property-seeds-and-shrinking.md) (property shrinking)
- **Topic:** 22 — Tooling
- **Rule keys:** `shrink.run.1-4`

## Context

A failure recorded from a real run carries the real inputs: a 5 MB log, a long configuration. The reviewer, human or agent, reads all of it to find the few lines that matter. Property tests shrink their inputs already (ADR 0199); a recorded failure does not.

## Decision

`ail replay --shrink [--time MS] [-o OUT] BUNDLE` cuts the texts that the bundle's recorded calls returned, line by line (delta debugging: halves first, then smaller pieces). After each cut it replays the bundle with `ail test --bundle` and keeps the cut only when the test still fails with the same `FAIL` line. It stops at a time budget (default 60 s) and writes the smaller bundle beside the original (`.min`). Computed only when asked for (ADR 0868).

## Rationale

Token usage first: the shrunk bundle is the failure an agent reads instead of the whole input. Reusing `--bundle` as the oracle means the tool needs no compiler change and no new replay path.

## Consequences

Each try is one build and run of the test, so large inputs take minutes; the time budget bounds it, checked between tries (a run overruns it by at most one try plus the first replay). Only texts are cut. Lists, maps and numbers in a recording stay as recorded; cutting them is a later slice. A cut that changes control flow so the replay finds a different call next fails with a different message and is undone, so the result always fails as the original does.

## Rejected alternatives

- **Cut inside the replay engine:** a second replay path to keep in step with `--replay`.
- **Cut the program's source instead:** that is delta-debugging code, a different tool.
- **Character-level cuts:** many more tries for little gain over lines.
