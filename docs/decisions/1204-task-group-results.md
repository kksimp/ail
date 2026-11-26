# 1204. `TaskGroup[T]`: hold many tasks that return a value and wait them all

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer approved the change in principle on 2026-11-26
- **Topic:** 9 — Concurrency
- **Spec rules:** [conc.task.5](../spec/11-concurrency.md), [conc.task.8](../spec/11-concurrency.md), [conc.task.10](../spec/11-concurrency.md)
- **Amends:** [ADR 0096](0096-scope-bound-tasks.md), [ADR 0187](0187-trapped-and-task-groups.md)

## Context

A task handle is unique and must be waited exactly once (conc.task.5), so it can't go in a `List`. A hands-on trial
by an agent (2026-11-26) wrote "start N workers, then collect all results" the way every other language lets it,
`handles.push(spawn f(x))`, was rejected, and rewrote the code as a `par` comprehension or a worker pool. A
`TaskGroup` already holds many handles and waits them all, but only for tasks that return nothing.

## Decision

- `TaskGroup[T]()` makes a group of tasks that return `T`. `g.add(spawn f(x))` moves a handle in (the task's result
  must be `T`); `g.wait_all()` waits for every task and returns `List[T] ! Trapped`, the results in the order the
  tasks were added. `TaskGroup()` is unchanged: a group of tasks that return nothing, `wait_all` returns `! Trapped`.
- The same rules as before: one trapped task fails `wait_all` with its `Trapped`; the block that holds the group
  waits for the tasks not yet waited, and a trap there ends the owner's task; `g.cancel()` cancels every task.
- A group that holds results is a local or a parameter. A field of a `unique` struct is still a `TaskGroup` of tasks
  that return nothing (conc.task.10): `wait_all` can't be called through a field, so a result would never be read.
- Tasks that return `T ! E` aren't added to a group yet (the type argument is `T`); wait those with `t.wait()?`.
- No new syntax and no new rule ID: this extends conc.task.8.

## Rejected alternatives

- **A `List[Task[T]]` that must be fully waited.** It needs a flow rule over every list operation (push, index,
  remove, sort) to prove each handle is waited once; a group already holds that invariant by construction.
- **Inferring `T` from the first `add`.** A flow-dependent type is worse for local reasoning; `TaskGroup[int]()` says it.
- **`g.wait_all()` returning results in completion order.** Add order is deterministic and matches `[f(x) par for x in xs]`.

## Consequences

Native: a `TaskGroup[T]` is the same runtime record as before; `wait_all` collects each finished task's result word
into a list (`rt.group_count`, `rt.group_result`). The checker reports a task of another result type added to a group
with the group to write.
