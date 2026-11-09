# 0996. `try_recv`, `recv_for`, and a `TaskGroup` field of a `unique` struct

- **Status:** Accepted
- **Date:** 2026-11-08
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 11 — Concurrency
- **Spec rules:** conc.share.8, conc.task.10
- **Builds on:** [ADR 0096](0096-scope-bound-tasks.md), [ADR 0187](0187-trapped-and-task-groups.md), [ADR 0188](0188-channel-api.md), [ADR 0191](0191-cleanup-with-drop.md)

## Context

A long-lived subsystem (Garlic's Web Workers) has to start tasks at any time
and poll for their messages from an event loop. The native subset had no
non-blocking or timed receive, and a struct couldn't hold a `TaskGroup`, so
Garlic spawned a fixed pool of eight threads in `main` and passed tokens
between them with sleeps.

## Decision

1. **`ch.try_recv() T?`** returns a queued value at once, or `none`.
   **`ch.recv_for(ms int) T?`** waits as `recv` does, for at most `ms`
   milliseconds. A timed wait is not counted as blocked, so it never reports a
   deadlock. Under `--deterministic` no time passes: `recv_for` waits as
   `recv` does, and its time is up only when every other task is blocked
   (where `recv` would report a deadlock). An earlier draft made it
   `try_recv`. Then a polling loop left early whenever the scheduler had not
   yet run the sender, and the gate's deterministic run of
   `conc_chan_try_recv` deadlocked.
2. **A `unique` struct may hold a `TaskGroup` field.** The must-wait rule of
   ADR 0096 holds in three ways:
   - The struct must be `unique`. Its value can't be copied, and it is
     released exactly once, when its owning scope ends on any path
     (own.uniq.3). That release waits for the group's tasks, as the end of a
     group's block does, and reports an unreported trap the same way
     (conc.task.8).
   - Building the struct moves the group to it. The block that called
     `TaskGroup()` no longer waits for it (`rt.group_own`), so the group can
     come from a constructor function and outlive that function.
   - The field can't be assigned, so the group that waits at release is the
     group the struct was made with. A task added, through an earlier copy of
     the group, to a group whose owner was already released is waited at once
     (`group_add`). No path leaves a task unwaited.
3. A **task handle** still can't be a field (conc.task.5). Use a group, which
   gives the same lifetime for any number of tasks. An enum variant can't hold
   a group: there is no single place for its release to wait.

## Alternatives rejected

- **Detached `spawn`:** it contradicts ADR 0096, a designer decision, and
  Garlic doesn't need it once a struct can own a group.
- **`TaskGroup` unique everywhere:** that changes every local group's
  copy-and-pass rules for one field case.
- **A task handle as a field, waited at release:** this turns a compile-time
  error into an implicit wait. Groups already cover the need.
- **`recv_for` returning `T ! Timeout`:** this costs more tokens at every
  poll site. `none` already means "nothing now", as it does for `try_recv`.

## Consequences

- A group record grows to 40 bytes, with an owner word.
- Polling loops use `try_recv` or `recv_for` instead of sleep-and-token
  protocols. Garlic's eight-thread pool can become a `unique struct Workers`
  with a group.
