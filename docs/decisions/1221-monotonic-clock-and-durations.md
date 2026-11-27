# 1221. A monotonic clock and durations as text

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the std library to be filled out on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** std.time.3, std.time.4, std.date.21, std.date.22, [test.stub.4](../spec/19-testing.md)
- **Amends:** [ADR 0157](0157-standard-os-modules.md) (adds to `std.time`), [ADR 1101](1101-test-stubs.md) (`test.clock` covers `mono_ns`); closes the clock caveat of [ADR 0836](0836-native-tls.md)

## Context

`std.time` had only the wall clock, which steps when the system time is set, so a program could not measure
elapsed time or keep a deadline correctly. The native runtime had its own monotonic clock for deadlines, but on
macOS it read the wall clock. Agents that write timing code reach for `Instant`, `time.monotonic()` or
`performance.now()` and found nothing. Durations are `int` milliseconds (std.date), with no text form in either
direction.

## Decision

- `time.mono_ns() int` and `time.since_ms(start_ns int) int` in `std.time`, with the `clock` effect. The clock is
  `CLOCK_MONOTONIC` on Linux, `clock_gettime_nsec_np(CLOCK_UPTIME_RAW)` through libSystem on macOS (ADR 0534) and
  the performance counter on Windows. The runtime's own deadline clock is the same function, so it is now monotonic
  on macOS too. The zero is undefined; only differences mean anything.
- `date.duration_text(ms) str` writes `1h 2m 3s` (units `d h m s ms`, zero parts left out, `0s` for zero, a leading
  `-` for a negative). `date.parse_duration(text) int ! DateError` reads `90s`, `1h30m`, `2d`, `1h 2m 3s`, `250ms`
  in any order with a leading `-` allowed, and fails with a `why` for each mistake. A sum over 10^18 ms is refused,
  so it never overflows. The two are inverses.
- `time.mono_ns()` follows `test.clock` (the clock in nanoseconds), so a test controls elapsed time. The runtime's
  own deadlines (a channel timeout, a network timeout) keep the real clock, so a stopped test clock can't hang them.
- Both new calls are recorded and replayed like `time.now_ms` (`rec_time_user_mono_ns`, `rec_time_since_ms`).
- No syntax change.

## Consequences

- A program can time work and set deadlines with a clock a wall-clock change can't break.
- macOS timeouts of the network and channel code are monotonic; `CLOCK_UPTIME_RAW` does not count time the machine
  sleeps, which for a timeout means a sleeping laptop does not expire it.
- The Mac gate checks the libSystem call; no Linux or Windows behaviour changed beyond the new functions.

## Rejected alternatives

- **A `Duration` type** (Swift, Rust, Go): a second number type for something an `int` of milliseconds already
  covers across std.date, std.net and `sleep_ms`; it would touch every time-taking function. Revisit if a unit
  mistake shows up in agent trials.
- **A `time.Instant` value from `mono_ns`:** the same cost, and an `int` reading composes with `since_ms` and
  subtraction.
- **`mach_continuous_time` on macOS:** counts sleep, but has no libSystem call as simple as
  `clock_gettime_nsec_np`; a timeout that expires while the lid is shut is the less common wish.
- **Fractional parts and `µs`/`ns`/`w`/`y` units in `parse_duration`:** a week or a year has no fixed length in
  the calendar sense, and fractions need rounding rules; milliseconds are the precision of every other call.
- **`since_ms` as an AIL function in `std.time.date`:** `time` has no AIL part, and the call needs the stubbed clock.
