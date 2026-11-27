# 1270. `std.resilience`: retry with backoff and jitter, circuit breaker, deadlines

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for a fuller std on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [std.resilience.1 to std.resilience.7](../spec/std-resilience.md)

## Context

The corpus task `retry_backoff_sim` hand-rolls a retry loop, and clients need a breaker and deadlines. `std.throttle`
(ADR 1235) landed rate limiting while this lane ran, so this module does not repeat it; its first draft had a token
bucket and sliding window, removed as a duplicate.

## Decision

- One AIL module, `std/resilience/resilience.ail`, in scope as `resilience` (std.use.4). No runtime or checker code; the one compiler edit is `resilience.` in `uses_tasks` (`lower_task.ail`), so a program that names the module links the task runtime (the breaker uses `Mutex` and `Atomic`).
- The breaker's state lives behind a `Mutex[int]` lock with `Atomic` fields (the native subset has `Mutex` of scalars only),
  so passing one breaker to several tasks shares it. Time is `time.mono_ns()` in ms, so it follows `test.clock`.
- `Backoff` doubles to a cap; `Full` jitter by default; `delay_ms` is public so callers with typed errors write their
  own loop. `retry` takes `fn(int) bool` because function values are not generic.
- Breaker: closed, open, half open with one probe; `call(f)` for the common case.
- Deadlines are cooperative: `Deadline`, `Deadline.sleep`, `within`.

## Rejected alternatives

- **Preemptive timeout of a running function:** needs a task per call and cancel-and-wait on every path; a function
  value cannot be spawned. Cancellation stays `TaskGroup.cancel`; revisit with a spawnable generic wrapper.
- **`retry` over `T ! E`:** function values are not generic, so one error type would be baked in.
- **Time as an argument (`_at` twins):** doubles the surface; the test clock already controls time.
- **A second limiter here:** `std.throttle` has the bucket, window and `pace`.
