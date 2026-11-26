# 1225. A thread looks at a channel 50,000 times before it sleeps

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer's speed mandate of 2026-11-17 and his 2026-11-26 note that a little more resource use is fine when it buys speed
- **Topic:** Native runtime
- **Spec rules:** native.chan.1 unchanged (no observable behaviour changes)
- **Builds on:** [ADR 1188](1188-channels-wake-one-sleeper.md), [ADR 1197](1197-bytes-loops-in-place-and-joined-map-keys.md)

## Context

After ADR 1197 the single-thread byte hash (the trial's FNV loop with
`wrapping_mul`) runs as long as C: 0.650 s against 0.649 s on the Mac, in
the same cycles. The trial's 1.3x came from the pool of one worker around the
same loop: 0.893 s. A pool of one worker made 84,000 voluntary context
switches for 60,000 lines and spent 63% of its samples in `__ulock_wait`.
A job takes the worker about 11 microseconds; the feeder (waiting for room)
and the collecting task (waiting for a value) gave up their 8,000 looks
(ADR 1188) before the next change and slept, so almost every change cost a
wake-up system call on the worker's path.

## Options considered

- **A: a longer look before sleeping (chosen).** `CHAN_LOOKS` is a constant
  read at build time, so it was swept on one compiler (main 959b9b47, Mac,
  best of 5, pool of 1 worker, 200 rounds):

  | `CHAN_LOOKS` | time | voluntary context switches |
  |---|---:|---:|
  | 8000 (before) | 0.920 s | 83,775 |
  | 20000 | 0.726 s | 2,434 |
  | 50000 | 0.721 s | 95 |
  | 150000 | 0.720 s | 24 |

  50,000 is where the gain stops. Pools of 2 and 4 workers stay at about
  0.36 s and 0.21 s; resident memory is unchanged at 19 MB. The cost is CPU:
  cycles rise from 5.5 G to 6.9 G while the elapsed time falls, and only
  while a thread waits for a change that comes within the look. A thread
  still sleeps when the wait is long, when the collector asks to stop the
  world (`GC_REQ`), and when no other thread runs (`MULTI` off).
- **B: wake a sender only when the queue is half empty.** Fewer wake-ups for
  the feeder, but a sender could stay asleep with room in the channel when
  the receiver stops taking, which a program can observe as a hang. Rejected.
- **C: batched receive (several values under one hold of the lock).** Would
  help short jobs further; a larger change to the surface and to `select`.
  Not built.

## Decision

A. No syntax, no observable behaviour change.

## Consequences

- The pool of one worker runs 0.72 s against 0.668 s for the C twin on one
  thread (8% over, was 38%). What is left is not the look: the same work in
  one spawned task is 0.665 s, so about 5% is three threads sharing the
  channel's cache lines and the rest the hand-off itself.
- Programs that wait for a change that arrives after the look still sleep
  and are woken as before.

## Rejected alternatives

- B and C above.
- **Adaptive look length:** one constant already reaches the plateau; the
  adaptation would cost more code than the 24 to 95 switches it removes.
