# 0896. `ail test --mutate`: which tests never notice a break

- **Status:** Superseded by [1022](1022-mutation-testing.md)
- **Date:** 2026-10-24
- **Decided by:** Designer (2026-10-24, chose the feature on a decision card); the details by AI (delegated, ADR 0076), for integrator review
- **Builds on:** [ADR 0395](0395-ail-test-update-rewrites-snapshots.md), [ADR 0871](0871-native-instrumentation-layer.md)
- **Topic:** 20 — AI semantic API

## Context

Passing tests say little about whether they would catch a bug. Agents write tests quickly, and need a cheap way to tell a strong test from an empty one.

## Decision

- `ail test --mutate [SYM|FILE]` changes the code in small fixed ways, one at a time: flip a comparison, swap `+`/`-`, drop a `return`, negate a condition, change a literal by one.
- For each change it runs only the tests that reach that line (`ail test --affected`). A change no test fails on is a survivor.
- Output: the mutation score, and for each survivor the line, the change, and the tests that reached it, capped by `--budget`. Mutations run on the native build, in parallel, with a per-run time cap.
- Never writes the source. Survivors can be printed as a stub `@test` for the agent to fill.
- Planned rule keys: `mutate.op.*`, `mutate.out.*`.

Cost when unused (ADR 0868): none. The feature is a separate command or pass, so a program that does not use it gets no extra code, no runtime check and no slower build.

## Consequences

Depends on `--affected` (ADR 0868 item 3) to be fast enough to use. Mutants that cannot compile are skipped, not counted.

## Rejected alternatives

- **Line coverage only:** a covered line can still be unchecked.
- **Random byte edits:** most do not compile; fixed operators give readable survivors.
