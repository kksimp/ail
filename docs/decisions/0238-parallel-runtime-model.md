# 0238. One work-stealing pool for tasks and parallel loops, with a minimum-work threshold

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 11 — Concurrency; 22 — Runtime architecture
- **Spec rules:** [conc.par.7](../spec/11-concurrency.md) (implementation policy, MAY)
- **Relates to:** [ADR 0095](0095-colorless-tasks.md), [ADR 0096](0096-scope-bound-tasks.md), [ADR 0236](0236-automatic-parallel-loops.md)

## Context

Parallel loops need a scheduler. Tasks (conc.task.3) already need one. The
designer asked to optimise for at least four cores; machines range from 4 to
128 hardware threads, often with performance and efficiency cores.

## Options considered

### Option A — One pool, work stealing, lazy binary splitting (chosen)
- **Pros:** No oversubscription when loops nest or run inside tasks; adapts to heterogeneous cores; splits only when a worker is idle.
- **Cons:** A more complex runtime than a fixed split.

### Option B — Static split into one chunk per core
- **Pros:** Simple.
- **Cons:** Unbalanced on P/E cores and data-dependent bodies; nested loops oversubscribe.

### Option C — A separate pool for loops
- **Cons:** Two schedulers competing for the same cores.

## Decision

The native runtime keeps one worker per hardware thread (capped by
`AIL_THREADS`), shared by tasks and parallel loops. Loops split lazily into
contiguous chunks; reductions combine in an order fixed by iteration ranges.
A loop whose estimated work is under about 20 µs runs sequentially; unknown
costs start sequentially and split after timing the first grain. A parallel
loop is fork-join inside its task: it finishes before the next statement, and
a trap reports the lowest-index trapping iteration. The static cost model
assumes at least four cores.

## Rationale

Criterion 4 (speed): work stealing is the best-studied model for nested and
irregular parallelism (Cilk, Rayon, Go). Criterion 2: fork-join inside the
statement keeps conc.task.4's scope rule without a `TaskGroup`.

## Consequences

- The bootstrap interpreter runs loops in order: its values are `Rc`, so a parallel interpreter would deep-copy captured values per chunk, which costs more than the perfbench loop bodies. No `ail run --parallel`.
- The performance report lists parallel loops, chunk counts and the active SIMD level (ADR 0026).

## Rejected alternatives

- **Static split:** unbalanced on heterogeneous cores.
- **Separate loop pool:** oversubscription.
- **Programmer-chosen schedules:** results could come to depend on them.
