# 1188. A channel change wakes one sleeping task, after the lock is released; the task queue stays one queue

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); asked for by the designer ("fast track on the mac", edge idea 2, 2026-11-25)
- **Topic:** 11 — Concurrency
- **Spec rules:** none changed (implements [native.chan.1](../spec/23-native.md), [conc.share.3](../spec/11-concurrency.md))
- **Builds on:** [ADR 0599](0599-native-threads-and-task-pool.md), [ADR 0943](0943-per-thread-allocation-caches.md), [ADR 1197](1197-bytes-loops-in-place-and-joined-map-keys.md)

## Context

The hands-on trial's worker pool (a feeder, K workers, two channels) ran no
faster with 4 workers than with 2. Edge idea 2 proposed three changes: wake
one sleeper per channel change, a task queue per worker with stealing, and
batches across a channel.

Measured on the Mac (8 cores, 4 fast), 2026-11-26, with a twin of the trial's
program (`tools/perfbench/pool/pool.ail`) and a spawn benchmark
(`spawn.ail`, 128000 tasks of about 2000 operations):

- The pool did not scale because `for b in bs` over a `bytes` built a list at
  every loop start, 200 times a job, so the workers met at the allocator and
  the collector. ADR 1197 removed that. After it the pool with 200 hash
  rounds a job (about 20 microseconds of work) runs in 2.05, 1.07, 0.54 and
  0.33 s with 1, 2, 4 and 8 workers.
- The one task queue is not a limit: the spawn benchmark runs in 0.43, 0.23,
  0.14 and 0.12 s on 1, 2, 4 and 8 threads, and `parsum` in 0.43, 0.22, 0.12
  and 0.07 s.
- Channels are the limit when a job is short. With 10 rounds a job (about
  1 microsecond) the pool took 0.84, 0.53, 0.66 and 0.94 s: every `send` and
  `recv` made a wake-up system call under the channel's lock, woke every
  sleeper, and a thread that found a lock held slept at once.

## Decision

1. **Two lines of sleepers.** A channel keeps the threads waiting for a value
   and those waiting for room in two lines, oldest first. Each sleeper sleeps
   on a word of its own. A put wakes the oldest of the first line, a take the
   oldest of the second; a close or a cancel wakes both lines. A timed
   receive that gives up leaves its line. With no sleeper there is no system
   call.
2. **Wake after the release.** The sleeper is taken out of its line under the
   lock and woken after the lock is released (`chan_unlock`).
3. **Look before sleeping.** With other threads running, a thread looks at
   the channel up to `CHAN_LOOKS` (8000) times without the lock before it
   joins a line, and `lock` looks at a held lock `LOCK_LOOKS` (64) times
   before it sleeps.
4. **Deterministic runs are unchanged.** They keep no lines: threads wait on
   the channel's wake word as before, so every schedule replays.
5. **`select` is unchanged.** Its sleepers wait on the global counter, which
   every change still wakes.
6. **One task queue stays.** Per-worker queues are not built: the
   measurements show no gain to be had on these programs.

Mac, tip 666ecef1 against main 919e33e0, best of 5, seconds:

| Pool | Workers | main | this |
|---|---:|---:|---:|
| 200 rounds, 100000 jobs | 1 / 2 / 4 / 8 | 2.05 / 1.07 / 0.54 / 0.33 | 2.10 / 1.05 / 0.44 / 0.29 |
| 10 rounds, 400000 jobs | 1 / 2 / 4 / 8 | 0.84 / 0.53 / 0.66 / 0.94 | 0.41 / 0.22 / 0.34 / 0.39 |

The spawn benchmark, the five `tools/perfbench/parallel` programs on 1 to 8
threads and `binarytrees` are within 2% either way.

## Consequences

- With short jobs, more than 2 workers still do not help (0.22 s with 2,
  0.34 with 4): the feeder and the collecting task are one thread each and
  every job crosses two locks. Passing several values under one hold of the
  lock would help; it is not built (see below).
- A thread that will wait long spends some microseconds looking first.
- Which of several waiting receivers gets a value is now the oldest sleeper
  or a running thread; the language promised no order and still does not.

## Rejected alternatives

- **Wake one sleeper on the shared wake word.** The other sleepers' recorded
  value goes stale, so the deadlock check (`woken_waiters`, conc.task.7)
  would take them for woken and never report a deadlock.
- **Wake a sender only when the channel is half empty.** A sender that could
  go on would stay asleep while its receiver waits for something else: a
  deadlock the program did not have.
- **Per-worker queues with stealing.** The largest and riskiest part of the
  idea, with no measured gain here. To be looked at again if a program with
  many short tasks shows the queue's lock.
- **Batches across a channel.** A receiver that takes several values holds
  ones another receiver could run, and loses them if it is cancelled; that
  changes behaviour, so it would be the designer's call.
