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

A loop that never allocates is not stopped, because it has no safepoint. A
back-edge check would need an edit to `compiler/mir/lower.ail`; that
is a follow-up for the integrator to approve. The time is wall-clock and the
macOS clock is untested until the Mac gate runs it.

## Rejected alternatives

- **A timer signal.** It needs a function address and a restorer stub on
  x86-64, and the runtime has neither.
- **A watchdog process that kills the program.** The program's buffered
  output is lost and its exit status is 137, not 1.
