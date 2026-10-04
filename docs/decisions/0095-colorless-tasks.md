# 0095. Concurrency uses colorless tasks: any function may block; no async/await

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 11 — Concurrency; 22 — Runtime
- **Spec rules:** [conc.task.1–3](../spec/11-concurrency.md)
- **Background:** [design/concurrency-modules-options.md](../design/concurrency-modules-options.md) (tokens: colorless 76.8, async/await 78.0)

## Options considered

- **Colorless tasks (chosen):** no function coloring, any function calls any
  other, and no forgot-`await` bugs. Con: pause points aren't marked, and
  each task needs a stack.
- **async/await:** visible pause points and the smallest tasks, but two
  worlds of functions and a common AI bug class.
- **OS threads only:** the simplest runtime, but too heavy for servers.

## Decision

1. `spawn f(args)` starts a new task running the call and returns a task
   handle.
2. Any function may block (I/O, waiting on a task, receiving from a channel).
   The runtime suspends the blocked task and runs others. No function is
   marked asynchronous, and no call is marked `await`.
3. Tasks are scheduled by the AIL runtime onto OS threads (M:N).

## Consequences

- **Stacks:** whole-program analysis (ADR 0015) should compute exact stack
  sizes for tasks whose call graphs are bounded. Otherwise stacks grow on
  demand. The task-memory cost is reported.
- The standard library's I/O is blocking-style in source, and non-blocking
  underneath (io_uring, IOCP or kqueue, per performance-beyond-c.md §3.5).
- FFI calls that block in C must be marked so the runtime can hand off the
  OS thread. That's an FFI-topic detail.
