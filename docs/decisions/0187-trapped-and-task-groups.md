# 0187. `wait()` fails with `Trapped`; groups hold tasks that return nothing

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Concurrency ([status.md](../status.md) topic 11)
- **Spec rules:** conc.task.5, conc.task.6, conc.task.8

## Context

`conc.task.6` says `wait()` fails with a `Trapped` error when the task
trapped, and `conc.task.4` says a `TaskGroup` waits at scope exit, but
neither the error's shape nor the group's API was decided.

## Options considered

### Option A — `error struct Trapped { rule str, message str }` in the prelude (chosen)
`wait()` on a `Task[T]` returns `T ! Trapped`; on a `Task[T ! E]` it returns
`T ! E | Trapped`. `message` starts with the trap's `LINE:COL`.
- **Pros:** ordinary error handling (`?`, `else err`) applies; the rule ID
  is a field an agent can match on.
- **Cons:** one more prelude name.

### Option B — nest results: `(T ! E) ! Trapped`
- **Cons:** the bootstrap has no nested results, and `??` is noise.

### Option C — re-trap in the waiter
- **Cons:** `conc.task.6` requires a recoverable failure.

For groups:

### Option A — `TaskGroup()`, `g.add(t)`, `g.wait_all() ! Trapped`, only `Task[void]` (chosen)
- **Pros:** results can't be dropped silently; a group is for fire-and-wait
  work that reports through channels or mutexes.
- **Cons:** a task with a result must be waited by itself.

### Option B — any `Task[T]`, results discarded
- **Cons:** loses results and failures silently.

## Decision

Option A for both. A group whose block ends with an unwaited trapped task
re-raises that trap in the owner, since nothing else would report it. A task
handle can't be named as a type, so it never leaves its block, and the
checker requires exactly one `.wait()` or `g.add(t)` on every path
(`conc.task.5`).

## Rationale

AI understanding (ADR 0076, criterion 2): failures stay visible and typed,
and the consumption rule is checked locally, within one block.

## Consequences

- `Trapped` is a reserved prelude type name.
- Tasks can't be stored in lists; a variable number of tasks goes into a
  group, and results come back through a `Channel`.

## Rejected alternatives

- Nested results: not supported and noisy.
- Re-trapping in `wait`: contradicts `conc.task.6`.
- Groups of any task type: drop results silently.
