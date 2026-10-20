# 0428. `proc.wait` lets other tasks run, and the time limit kills the child

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 15 — Concurrency; 21 — Standard library architecture
- **Spec rules:** [std.proc.4, std.proc.6](../spec/17-std.md#processes-stdproc)
## Context

`wait` could block the whole interpreter on the OS (`Child::wait`). With
cooperative tasks, the task that would close the child's stdin may never run.

## Options considered

### Option A — poll `try_wait`, yielding the turn between polls (chosen)
`wait` loops: has the child exited? If not, hand the turn to another ready
task, or sleep up to 5 ms on the event counter (woken at once by the child's
output or end of stream).
- **Pros:** no deadlock between a waiter and the task that feeds the child;
  a child's exit is noticed within one event (its pipes close).
- **Cons:** up to 5 ms of latency for a silent child that exits without
  closing its pipes (rare).

### Option B — block in the OS
- **Cons:** the deadlock above.

### Option C — a watcher thread per child that posts an event on exit
- **Pros:** exact wakeups.
- **Cons:** one more thread per child for 5 ms of worst case. Revisit if
  perfbench ever shows it.

## Decision

Option A. The time limit is a detached thread that kills the child after
`timeout_ms` if it is still registered; `wait` then fails with
`timed_out`. A cancelled task's `wait` kills the child and fails with
`cancelled` (`conc.cancel.2`).

## Consequences

- `proc.wait` is a switch point only through `yield_turn`; schedule seeds
  replay task order, not child timing.
