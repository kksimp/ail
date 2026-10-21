# 0729. A `while` loop checks the time limit in builds that bake one

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** run.limit.1, run.limit.6
- **Builds on:** [ADR 0727](0727-native-timeout.md)

## Context

ADR 0727 polls the clock in the allocator, so `while true: n += 1` is never
stopped. A check at each loop turn fixes that but costs the tight loops of
the drag race.

## Decision

`lower_while` emits, at the loop head, a load of the runtime's `LIM_TICK`
slot and a branch to a call of `rt.limits_poll` when it is nonzero. It does so
only when the runtime defines `lim_loop_polls`, which `ail build --timeout`
adds to the runtime source. A build without `--timeout` is byte-identical to
before. The poll counts turns with the allocator's countdown, so the clock is
read every 256 turns. Only `while` is a safepoint; a `for` over a range or
list is bounded by its input.

Measured on Linux x86-64 with the check always emitted: `collatz` 0.027 s to
0.036 s (+30%), `fannkuch` +8%, `sieve` and `nbody` within noise. That cost
is why it is opt in: an executable built with `--timeout` pays it, any other
pays nothing.

## Rejected alternatives

- **Always emit it.** +30% on tight loops that users can't opt out of.
- **A signal.** The runtime has no function-address primitive (ADR 0727).
- **Poll `for` loops too.** A `for` ends when its range or list does.
