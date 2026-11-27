# 1209. Task groups of tasks that can fail, and result groups as fields

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer approved `TaskGroup[T]` as a language change on 2026-11-26 and asked for its gaps to be filled
- **Topic:** 9 — Concurrency
- **Spec rules:** [conc.task.8](../spec/11-concurrency.md), [conc.task.10](../spec/11-concurrency.md)
- **Amends:** [ADR 1204](1204-task-group-results.md), [ADR 0187](0187-trapped-and-task-groups.md), [ADR 0996](0996-try-recv-and-group-fields.md)

## Context

[ADR 1204](1204-task-group-results.md) left two gaps. A task whose function returns `T ! E` couldn't go in a
group, so "run N fallible jobs, stop at the first error" needed one handle per task again. And a `TaskGroup[T]`
couldn't be a field of a `unique` struct, so a type that starts jobs couldn't collect their results.

## Decision

- `TaskGroup[T ! E]()` holds tasks that return `T`, or `T ! F` with every error of `F` among `E`. `TaskGroup[! E]()`
  is the same for tasks that return nothing. The type argument is written as a function's result type, which the
  grammar already accepts (`TypeArg = INT | Type`): no new syntax.
- The errors are named. `TaskGroup[!T]` is an error whose help shows `TaskGroup[int ! ParseError]()`, and a task
  with more errors than the group's gets a help that names the wider group.
- `g.wait_all()` returns `List[T] ! E | Trapped`. Its failure is that of the first task, in add order, that failed
  or trapped; the tasks added after it are cancelled (`conc.cancel.1`) and still waited, so none is left running.
  After a failure there are no results.
- A trap in a task of such a group ends the program, as `t.wait()` of a task with errors of its own already does
  natively.
- A failed task whose error `wait_all` never reported ends the program when the group's block ends (or its owner is
  released), reported as an error that reached `main`, with the context line "a task of a `TaskGroup` failed and
  `wait_all()` never read its error". An error is never dropped silently; the same holds for traps (ADR 0187).
- A field of a `unique` struct may be a `TaskGroup[T]` or `TaskGroup[T ! E]`; `wait_all()` is called through the
  field. The release still waits every task (conc.task.10), so the must-be-waited guarantee is unchanged.

## Rejected alternatives

- **Inferring the group's errors from the tasks added.** Flow-dependent, like inferring `T` (rejected in ADR 1204):
  the type of `wait_all` would depend on every `add` before it. Named errors are read at the declaration.
- **`wait_all` returning a list of per-task results.** `List[T ! E]` isn't a type in AIL (a result is handled where it
  is made, err.type.1), and every caller would loop to find the first error; the common case is "all or the first
  failure", as in a `par` comprehension.
- **The first failure in completion order.** Depends on scheduling (conc.task.9); add order gives the same error on
  every run.
- **Cancelling the other tasks the moment one fails.** Needs a task to know its group; cancelling the tasks after the
  first failure in add order, when `wait_all` reaches it, keeps the runtime record unchanged and the result the same.
- **Dropping an unread error at the block's end.** A silent failure is the worst outcome for an agent reading the
  program's output.

## Consequences

Native: `Ty.GroupOf` carries the tasks' error type (`Unit` when they can't fail). `rt.group_wait` takes whether the
tasks have errors of their own, restores the first failed task's error, and cancels the tasks after it;
`rt.group_finish` reports an unread error. A struct's release waits for a `TaskGroup[T]` field as for a `TaskGroup`.
Not done: cancelling the remaining tasks as soon as any task fails, before `wait_all` reaches it.
