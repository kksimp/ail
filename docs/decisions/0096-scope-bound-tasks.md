# 0096. Tasks are scope-bound (structured concurrency)

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer (rules 3–4 delegated to the AI under ADR 0076)
- **Topic:** 11 — Concurrency
- **Spec rules:** [conc.task.4–6](../spec/11-concurrency.md)
- **Background:** [design/concurrency-modules-options.md](../design/concurrency-modules-options.md)

## Options considered

- **Scope-bound (chosen):** tasks can't leak, errors and traps always reach a
  parent (ADR 0047), and cancellation is natural. Con: fire-and-forget needs
  a task group.
- **Free-running (Go):** simplest, but leaked tasks and lost errors.

## Decision

1. Every task finishes before the block that spawned it exits.
2. Long-lived background work runs in an explicit `TaskGroup` value, created
   in an outer scope and passed (as `mut`) to where tasks are spawned. The
   group waits for its tasks when its scope ends.
3. *(Delegated: AI understanding.)* A task handle is a `unique` value
   (ADR 0066). It must be consumed with `.wait()` or handed to a group before
   its block ends, or it's a **compile-time error**. Structure is enforced
   statically, and no join or error happens implicitly.
4. *(Delegated: AI understanding.)* `handle.wait()` returns the task's
   result. If the task trapped, `wait()` fails with a `Trapped` error that
   carries the trap's full diagnostic (ADR 0047), so it joins the caller's
   error set like any other error.

```
a := spawn load_profile(1)
b := spawn load_profile(2)
print(a.wait()?)
print(b.wait()?)
```

## Consequences

- Cancellation propagates downward through scopes. Its API is to be designed.
- `TaskGroup` cleanup behaviour (how failures of grouped tasks are reported)
  is to be specified with the standard library.

## Rejected alternatives

- Free-running tasks (Go-style): simplest to start, but leaks tasks and loses their errors. Rejected because scope-bound tasks cannot leak, errors and traps always reach a parent (ADR 0047), and cancellation is natural.
