# 1317. A parallel region times itself in order once it finishes fast on the pool, and stays in order while the pool would not repay

- **Status:** Accepted
- **Date:** 2026-11-28
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); asked for by the designer (top priority, make ail-llm fast in pure AIL, kksimp/ail#1598 item 7, 2026-11-28)
- **Topic:** 11 — Concurrency
- **Spec rules:** adds [conc.par.21](../spec/11-concurrency.md), amends conc.par.8 and conc.par.17
- **Builds on:** [ADR 0946](0946-adaptive-thread-pool.md), [ADR 1157](1157-counted-loops-need-more-work-for-the-pool.md)

## Context

The cost model (conc.par.14) counts a body's operations and prices a nested loop at 16 times
and a call of a function with a loop at 4096, because the trip counts are not known where the
loop is compiled. ail-llm's loops are `[row4(...) par for r in 0..m.rows]` and an attention
`par for` over heads: every iteration calls a function whose loop runs over a runtime length.
The count says "heavy", so the region takes the whole pool even when its total work is a few
microseconds: 12 iterations of attention over 8 positions took 80-120 us on the pool against
20 us in order, and a 256-row matrix-vector product gained nothing (issue #1598 item 7).

## Decision

Each region (a written `par for`, or a loop of the compiler's that no loop encloses) has a
slot in a table of 1024 entries in the pool block, by its task id. The first runs take the pool as before.
A pooled run that was over within 400 us marks the slot "to time"; the next run is in order and
timed (two monotonic clock reads), which stores the nanoseconds per iteration. From then on
the region goes to the pool only when its iteration count times that figure is at least 80 us
(a region costs 25 to 60 us to start), else it runs in order and is timed again, with the mean of
the last two runs in order as the figure, so a cost that grows from run to run (attention over
a longer cache) moves it back to the pool. A region that is big on its first run never times
anything. Results do not depend on any of this (conc.par.1); `AIL_PAR_LOG` shows the choice.

## Rejected alternatives

- **A better static estimate** (inner loop trips read from the source): ail-llm's trips are
  runtime lengths of a callee's loop, which the compiler does not see; a literal-trip estimate
  would leave its loops as they are.
- **Timing every run of every region:** two clock reads cost more than a small region's work in a
  hot loop; only regions that finished fast on the pool are timed, and loops nested in loops are left alone.
- **Probing a big region in order once:** a single huge region would run on one core for its whole
  duration; the first run goes to the pool, and only a fast one is probed (at most a few ms in order).
- **A runtime that adapts by chunk size alone:** wakes the workers anyway, which is the cost.

## Consequences

A hot small region pays a table read and a clock read twice per run in order; a region that
alternates sizes by orders of magnitude may spend a run on the wrong side before the estimate
follows (it is per iteration, so the count of iterations is taken fresh). The estimate of a
cost that shrinks 10 times while on the pool is not refreshed. Tests: `par_learn`. Measurements: see the pull request.
