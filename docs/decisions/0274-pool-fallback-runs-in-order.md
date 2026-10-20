# 0274. When the pool can't give the sequential result, the loop reruns in order

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 11 — Concurrency
- **Spec rules:** [conc.par.11](../spec/11-concurrency.md), [conc.par.4](../spec/11-concurrency.md)
- **Relates to:** [ADR 0236](0236-automatic-parallel-loops.md) (results identical to sequential)

## Context

A pooled loop must give the sequential result, trap included (conc.par.1).
Reproducing "the lowest-index trap" across workers, or an overflow at an
interior running total, needs care.

## Options considered

### Option A — Discard and rerun in order (chosen)
The pool writes nothing to the program's variables until every chunk has
finished. On a trap, a handle in a captured value, a non-`int` accumulator,
or a running total that would leave `int` (checked from each chunk's offset
plus its least and greatest running total), the main thread discards the
chunks and runs the loop in order from the same iteration values.
- **Pros:** Exactly sequential, with one simple rule; bodies have no effects, so rerunning is invisible.
- **Cons:** A trapping loop costs its parallel time plus a sequential run, only on the way to a trap.

### Option B — Report the lowest-index trap from the workers
- **Cons:** Needs every lower chunk to finish and the writes before the trap applied; more code for the same answer.

## Decision

Option A. Workers stop taking chunks once one fails.

## Rationale

Criterion 2 (AI understanding): one rule, "the pool either gives the
sequential result or isn't used". Speed only matters on the trapping path.

## Consequences

- Tests `par_pool_trap` and `par_pool_overflow` show the sequential trap on 1 and 4 threads.

## Rejected alternatives

- **Worker-reported traps:** more code, same result.
