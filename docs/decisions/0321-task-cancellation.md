# 0321. Cancellation: `cancel`, `task.cancelled()`, `task.check()`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 11 Concurrency
- **Spec rules:** `conc.cancel.1`, `conc.cancel.2`

## Context

Scope-bound tasks (ADR 0096) must be waited, so a worker that loops on a
channel nobody closes deadlocks its scope, and an agent tool that started
four probes has no way to stop the other three once one answers. The brief
asked for a cancel flag, a way to read it, and blocking calls that return
early with a prelude error `Cancelled`.

## Options considered

### Option A — cooperative flag; blocking calls end early in their own type (chosen)
`t.cancel()` and `g.cancel()` (the group is the scope) set the flag;
`task.cancelled()` reads it and `task.check()` fails with `Cancelled`. In a
cancelled task `recv` returns `none` (and `for v in ch` ends), `sleep_ms`
returns at once, and `proc.run` fails with its existing `ProcError`, kind
`"cancelled"`.
- **Pros:** no existing signature changes; the commonest worker shape,
  `for v in jobs`, stops with no new code; `task.check()?` is one line where
  a task wants an error.
- **Cons:** `recv` can't tell "closed" from "cancelled" without
  `task.cancelled()`.

### Option B — blocking calls fail with `Cancelled`
`recv() T? ! Cancelled`, `sleep_ms() ! Cancelled`.
- **Pros:** the brief's literal wording; the reason is in the value.
- **Cons:** every existing `recv`, `sleep_ms` and `for v in ch` would have
  to handle a failure it can't get outside tasks: more tokens on every call
  in the corpus, and a breaking change to ADRs 0188 and 0300.

### Option C — pre-emptive cancellation (stop the task at any point)
- **Pros:** works for tasks that never block or check.
- **Cons:** can stop a task halfway through a mutex update or a file
  write; no language that has tried it keeps it (Java's `Thread.stop`).

## Decision

Option A. Cancelling doesn't consume a handle; scope exit still waits every
task (`conc.task.4`). A task spawned by a cancelled task starts cancelled. A
cancel wakes the task if it is blocked, so a blocked `recv` returns.
`task` is a standard module like `time` (in scope without `use`, ADR
0277), with no capability.

## Rationale

Tokens decide (criterion 1): Option B would add an `else` or `?` to every
`recv` and `sleep_ms` in the corpus; Option A adds nothing to existing code
and a single `g.cancel()` where it is used. AI understanding (criterion 2):
a cancelled worker's loop ends the way a closed channel's does, the
behaviour an agent already reasons about.

## Consequences

- `send` on a full channel still waits in a cancelled task, and cancelling a
  task doesn't reach tasks it already started: OQ-097.
- `proc.run` callers see `kind == "cancelled"` among the kinds of `std.err.1`.

## Rejected alternatives

- Fallible blocking calls: changes three settled signatures and costs
  tokens on every call for a case most programs never meet.
- Pre-emptive stop: unsafe with mutexes and side effects.
- A separate `CancelToken` value passed by hand: more tokens, and the group
  already is the scope.
