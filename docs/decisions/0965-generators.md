# 0965. Generators: `yield` in a function returning `Seq[T]`, lowered to a state machine

- **Status:** Superseded by [1073](1073-generators-and-user-iteration.md)
- **Date:** 2026-11-05
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 05 — Expressions; 11 — Concurrency
- **Spec rules:** planned `gen.yield.1-4` (spec 05), added with the implementation; the rules are the Decision items below
- **Builds on:** [ADR 0095](0095-colorless-tasks.md), [ADR 0096](0096-scope-bound-tasks.md)

## Context

No lazy sequences exist: loops go over built `List`s. Design: [generators.md](../design/generators.md).

## Decision

1. A function containing `yield e` and returning `Seq[T]` is a generator (gen.yield.1). No keyword, no colour; calling it returns a `Seq[T]` and runs no body code.
2. Lowering is an unmarked state machine, no stack, task or channel (gen.yield.2). Colorless tasks (ADR 0095) are unchanged, so no designer decision is touched.
3. `Seq[T]` is single-use (gen.yield.3) and `for x in seq` is allowed (gen.yield.4, extends expr.loop.4).
4. `yield` inside `par`, `spawn` or a closure is an error in slice 1.

## Rejected alternatives

Task plus channel; coloured async functions (ADR 0095); stackful coroutines; a `gen` keyword. Reasons in the design note.

## Consequences

If review wants a general `Iterator` trait, that waits on OQ-037 and goes to the designer.
