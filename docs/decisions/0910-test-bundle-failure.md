# 0910. `ail test --bundle-failure`: one file that reproduces a failing test

- **Status:** Accepted (2026-11-07, by the integrator after it landed through the gate with its spec rules; proposed 2026-10-25)
- **Date:** 2026-10-25
- **Decided by:** Designer (2026-10-25, chose the feature on a decision card); the details by AI (delegated, ADR 0076), for integrator review
- **Builds on:** [ADR 0398](0398-record-and-replay.md) (record and replay), [ADR 0199](0199-property-seeds-and-shrinking.md) (property seeds), [ADR 0186](0186-deterministic-task-scheduler.md) (schedule seeds)
- **Topic:** 22 — Tooling
- **Planned rule keys:** `bundle.write.*`, `bundle.run.*`

## Context

A failing test is cheap to see and expensive to reproduce: the agent needs the property seed, the schedule seed, the recorded effect results and the compiler version, then retypes them into commands.

## Decision

`ail test FILE --bundle-failure[=DIR]` records every failing test (`--record`, ADR 0398) and, for each, writes one file (in DIR, default `.ail/bundles/NAME.json`; a JSON header line followed by the recording's JSON lines, `bundle.write.1-2`) holding: the test name and program text hash; the property seed and the shrunk counterexample; the schedule seed; the ADR 0398 recording; the failure message; the compiler version. Nothing is written when all tests pass. `ail test --bundle PATH` (`bundle.run.1-2`) reruns exactly that failure and reports as `ail test` does, with a note on standard error when the program differs from the hash. Computed only when asked for.

## Rationale

Token usage first: one path replaces the seeds, flags and fixtures an agent would otherwise carry through its context. The recording already exists, so the cost is packaging.

## Consequences

Tests that read the disk replay against the real disk, as with ADR 0398; a bundle holds no disk contents. Bundles are plain text and may hold recorded inputs, so they are not for publishing without a look.

## Rejected alternatives

- **A directory of files:** many paths to pass; one file is one argument.
- **Bundling the whole program source:** large, and the hash plus the repository already identify it.
