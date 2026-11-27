# TBD. `std.resilience`: rate limiters, retry with backoff, circuit breaker, deadlines

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for a fuller std on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [std.resilience.1 to std.resilience.14](../spec/std-resilience.md)
- **Number:** the integrator assigns it; rename this file and add the index row then.

## Context

The corpus tasks `retry_backoff_sim` and `rate_limiter_sim` hand-roll a retry loop and a limiter, and servers and
clients need both. Neither `std.time` nor the concurrency primitives ship them.

## Decision

- One AIL module, `std/resilience/resilience.ail`, in scope as `resilience` (std.use.4). No runtime or checker code.
- State lives in a `Mutex` inside the value, so passing one limiter or breaker to several tasks shares it. Time is
  `time.mono_ns()` in ms, so it follows `test.clock`.
- Token bucket in exact integers: the level counts 1/`period_ms` tokens, refill per ms is `tokens`, so no float drift.
  `take` reserves tokens under the lock and sleeps outside it, which serves waiters in order.
- Sliding window as a log of grant times (exact, no edge burst).
- `Backoff` doubles to a cap; `Full` jitter by default; `delay_ms` is public so callers with typed errors write their
  own loop. `retry` takes `fn(int) bool` because function values are not generic.
- Breaker: closed, open, half open with one probe; `call(f)` for the common case.
- Deadlines are cooperative: `Deadline`, `Deadline.sleep`, `within`.

## Rejected alternatives

- **Preemptive timeout of a running function:** needs a task per call and cancel-and-wait on every path; a function
  value cannot be spawned. Cancellation stays `TaskGroup.cancel`; revisit with a spawnable generic wrapper.
- **`retry` over `T ! E`:** function values are not generic, so one error type would be baked in.
- **Float token levels:** drift and platform-dependent rounding; integers are exact.
- **Time as an argument (`try_take_at(now)`):** doubles the surface; the test clock already controls time.
- **Rate limiter "over channels":** a bucket guarded by a `Mutex` needs no helper task and costs no thread.
