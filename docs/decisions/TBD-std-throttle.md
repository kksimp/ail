# TBD. `std.throttle`: token bucket, sliding-window limiter and paced channels

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); wave 3 of the std plan. The number comes from the integrator (rename this file and the rows that say TBD).
- **Topic:** 17 — Standard library
- **Spec rules:** [std.throttle.1 to std.throttle.10](../spec/std-throttle.md)

## Context

Agents asked for a rate limiter (the corpus task `rate_limiter_sim`) write a refill by hand each time, and the usual
first draft mixes the wall clock with the monotonic one, loses fractions of a token, or sleeps a busy loop. The
monotonic clock (`time.mono_ns`, ADR 1221) and channels (ADR 0096) exist; the pieces above them do not.

## Decision

- A pure AIL module, `std/throttle/throttle.ail`, in scope without `use` as `throttle` (std.use.4). Registration lines
  only, no runtime or checker code. The name has no local clash: no `.ail` file binds `throttle`.
- `Bucket`: a token bucket with integer `rate` and `burst`, kept in billionths of a token, so there is no float
  error and no drift. Clamped refill avoids overflow for any gap between readings.
- `Limiter`: a sliding window of `limit` events in `window_ms`, stored as a ring of the last `limit` event times: fixed
  memory, O(1) per call, and exact (not the two-window approximation).
- Every clock-reading call has an `_at(now_ns)` twin. Tests and simulations pass made-up readings; the plain form is
  one line over it.
- Bad numbers give `none` or `false`, as in `std.stats`: there is one way to fail per call.
- `pace` is the channel piece: a task that forwards `input` to `output` through a bucket and closes `output`. The
  caller spawns a declared function that calls it.

## Rejected alternatives

- **A rate-limited `Channel`** (a new channel type): a runtime change for what a task and a bucket already do.
- **Float rates and tokens:** `0.1 + 0.2` style drift in a refill, and `==` on a balance. Whole tokens per second
  cover the cases; slower rates are a bucket of burst 1 and a longer interval, or a `Limiter`.
- **A leaky bucket as a separate type:** it is the same state machine as the token bucket seen from the other side.
- **`T ! ThrottleError` for bad numbers:** an enum for one condition that every caller would answer with `else`.
- **A fixed-window counter:** lets twice the limit through at a window edge.
- **`wait` that returns nothing:** its only failure (a request above the burst) would then sleep forever or trap.
