# 0727. A time limit for native programs, polled at the allocator

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** run.limit.1, run.limit.6
- **Builds on:** [ADR 0726](0726-native-memory-limit.md)

## Context

A runaway native program must stop at `--timeout`. Signals need a handler's
address, and the native subset has no primitive for a function address.

## Decision

With a time limit on, `rt.alloc` counts down from 256 and reads
`clock_gettime` (`gettimeofday` on macOS) when it reaches zero, and also reads
it whenever the heap takes a chunk. Past the deadline the program flushes
its output, prints `trap[run.limit.1]` with the interpreter's message and
exits 1. Without a limit the fast path is one load and one untaken branch.

Measured on Linux x86-64 (`binarytrees`, depth 16, five runs each): 0.36 s
before and after, within noise. It also passes under qemu aarch64.

The time is wall-clock and the macOS clock is untested until the Mac gate
runs it.

## Loop safepoint

A loop that never allocates is not stopped by the allocator poll, so
`lower_while` can emit a check at the loop head. It is emitted only when the
runtime defines `lim_loop_polls`, which `ail build --timeout` adds to the
runtime source; any other build is byte-identical to one without the check.
The check loads the runtime's `LIM_TICK` slot and calls `rt.limits_poll` when
it is nonzero; the poll shares the allocator's countdown, so the clock is read
every 256 turns. Only `while` is a safepoint, because a `for` ends with its
range or list.

Measured on Linux x86-64, always emitted (mean of five runs):

| program | none | load of the slot | down-counter in a register |
|---|---|---|---|
| `collatz` | 27 ms | 36 ms (+30%) | 37 ms (+37%) |
| `fannkuch` | 55 ms | 59 ms (+8%) | 61 ms (+11%) |
| `sieve` | 21 ms | 22 ms (+3%) | 24 ms (+10%) |

A register counter is no cheaper: the loops are a few instructions long, so
any check adds a comparison and a branch to each turn. Both exceed 3%, so the
check stays opt in (an executable built with `--timeout` pays it). Making it
the default is the integrator's call.

## Rejected alternatives

- **A timer signal.** It needs a function address and a restorer stub on
  x86-64, and the runtime has neither.
- **A down-counter in a register at every loop head.** Measured above; no
  cheaper than the slot load.
- **A watchdog process that kills the program.** The program's buffered
  output is lost and its exit status is 137, not 1.
