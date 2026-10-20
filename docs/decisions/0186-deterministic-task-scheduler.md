# 0186. Run tasks in the interpreter on a deterministic, one-at-a-time scheduler

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Concurrency ([status.md](../status.md) topic 11)
- **Spec rules:** conc.task.1-4, conc.task.7, conc.task.9

## Context

ADRs 0095 and 0096 decided colorless, scope-bound tasks; the bootstrap
rejected `spawn`. The interpreter is the semantic oracle (ADR 0032), so
repeatable results matter more than parallel speed. Its values are `Rc` and
not thread-safe, and it is a recursive tree walker, so a task can block
(`wait`, a full channel) anywhere in its call stack.

## Options considered

### Option A — one OS thread per task, one running at a time (chosen)
Each task runs its own interpreter on its own thread. A mutex and condition
variable pass a single turn: a task runs until it blocks or finishes, then
hands the turn on. A new task runs at once; its spawner resumes next; other
ready tasks run first in, first out.
- **Pros:** blocking works at any call depth with no change to the evaluator;
  output is repeatable; `Rc` stays sound because only the turn holder
  touches values and the handoff orders every access; about 400 lines in one
  new file.
- **Cons:** a thread per live task (stacks are reserved, not committed); a
  switch costs a few microseconds.

### Option B — rewrite the evaluator as a state machine or continuation-passing
- **Pros:** true green threads in one OS thread.
- **Cons:** rewrites every evaluator function, which all lanes edit tonight;
  slower for programs without tasks.

### Option C — run a task to completion at `spawn`, or nested on the waiter's stack
- **Pros:** trivial.
- **Cons:** wrong: a producer filling a bounded channel, or two tasks that
  talk both ways, deadlock although the program is correct.

### Option D — real parallel threads
- **Cons:** `Rc` values would race; output would not repeat.

## Decision

Option A, in `bootstrap/src/interp/tasks.rs`. When every unfinished task is
blocked, the task that finds it traps with `conc.task.7` and a message that
names what it waits for. The order of ready tasks is unspecified by the
language (`conc.task.9`); native code schedules onto OS threads in parallel
(`conc.task.3`), and programs must not depend on the bootstrap's order.

## Rationale

AI understanding (ADR 0076, criterion 2): a repeatable run gives the same
output and the same diagnostics every time, so an agent can test concurrent
code like sequential code, and a deadlock is a located trap rather than a
hang. Speed (criterion 4) is unaffected for programs without tasks: the only
cost is one `Option` check when a block ends and when printing
(perfbench medians within noise: `loops` 0.726 s before, 0.721 s after;
`interp` 0.247 s before, 0.240 s after). Starting a task costs about 0.2 ms
(a thread), and a channel handoff about 1 µs per value.

## Consequences

- Concurrent examples and tests have exact expected output.
- Bootstrap limits: `spawn` takes a declared function or associated
  function call, with no `mut` arguments (`conc.share.1`).
- Tasks' output is buffered in the scheduler and copied to standard output by
  the main task, in the order printed.
- Follow-up: `ail run --schedule random:SEED` to shake out order
  dependence (`conc.task.9`) cheaply on the same machinery.

## Rejected alternatives

- B: too invasive for the shared evaluator, and slower for every program.
- C: deadlocks correct programs.
- D: unsound with `Rc` values and not repeatable.
