# 0426. The scheduler waits for a child's output instead of reporting a deadlock

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 15 — Concurrency
- **Spec rules:** [std.proc.5](../spec/17-std.md#processes-stdproc), [conc.task.7](../spec/11-concurrency.md#tasks)
- **Builds on:** [ADR 0186](0186-deterministic-task-scheduler.md)

## Context

The scheduler is deterministic and cooperative: one task runs at a time and
hands the turn on when it blocks (ADR 0186). A blocked task with nothing
ready is a deadlock (`conc.task.7`). A child's output arrives from outside,
so "nothing ready" may only mean "waiting for the child".

## Options considered

### Option A — external event queue, polled at scheduling points (chosen)
Reader threads put lines in a queue and bump an event counter. The
scheduler moves queued lines into the channel at every scheduling point
(`recv`, `block`, `finish`, `select`, `wait`). When no task is ready but a
task waits on a channel a live child feeds, it sleeps on the event counter
(woken by output, end of stream, or every 50 ms) rather than trapping.
- **Pros:** the interpreter's tasks still never run in parallel, so sharing
  `Rc` values stays sound; with no child, the code path is one failed
  `any()` and behaviour is unchanged (all deadlock cases still trap).
- **Cons:** the order of lines from two streams, and of a stream against
  other tasks, depends on the child's timing; tests make the child wait on
  its input to fix the order.

### Option B — one OS thread per blocking read, results sent as values
- **Cons:** values would cross threads (`Rc`), breaking the scheduler's
  soundness argument.

### Option C — non-blocking reads polled by the task
- **Cons:** busy loops, and every program must write the loop.

## Decision

Option A. A deadlock is reported only when no child could still wake a
waiter. Under `--schedule=random` the schedule of tasks stays seeded;
child timing is outside the seed (`--deterministic` denies `proc`).

## Consequences

- `select`, `recv` and `for` over a child's stream behave like any channel
  once data is queued.

## Rejected alternatives

- One OS thread per blocking read, with results sent as values (option B): rejected because values would cross threads (`Rc`), breaking the scheduler's soundness argument.
- Non-blocking reads polled by each task (option C): rejected because busy loops would be needed, and every program would have to write the loop.
