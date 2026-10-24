# 0322. `--schedule=random[:SEED]`: replayable random interleavings

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 11 Concurrency, 19 Testing
- **Spec rules:** `conc.sched.1`, `conc.sched.2`

## Context

The bootstrap's scheduler is deterministic (ADR 0186): one fixed order, so
a race that native code would hit (`conc.task.9`) never shows in tests.
Lane 49's pool runs pure tasks on real threads, where a race shows but
can't be replayed. An agent fixing a concurrency bug needs to see the bad
interleaving and rerun exactly it.

## Options considered

### Option A — seeded switch points in the scheduler (chosen)
Under `--schedule=random:SEED`, a seeded xorshift generator decides at each
`spawn` whether the new task or its spawner runs first, at every handle
method, atomic operation and `task.cancelled()`/`task.check()` whether to
yield, and which ready task runs next. The pool is turned off so every task
is on the scheduler. A failing `ail test` prints the seed.
- **Pros:** exact replay on every target (the generator is integer-only);
  switch points are exactly where tasks can observe each other; no cost
  without the option.
- **Cons:** no switches between two plain statements, so a race on
  unsynchronised state isn't explored; but safe code has none
  (`conc.race.1`).

### Option B — record and replay real thread schedules
- **Pros:** finds what real hardware does.
- **Cons:** needs a recorder in every synchronisation primitive and the
  pool; much larger, and the next run differs anyway.

### Option C — exhaustive exploration (model checking)
- **Pros:** finds every interleaving.
- **Cons:** exponential; belongs in a later tool that can reuse Option A's
  switch points.

## Decision

Option A, on both `ail test` and `ail run`. `--schedule=random` without a
seed picks one from the clock. The text report adds
`schedule seed N: replay with ...`; JSON adds `"schedule_seed":N` to the
summary object, only when a test failed.

## Rationale

AI understanding decides (criterion 2): a failing concurrent test becomes
a deterministic, replayable repro an agent can bisect, from one flag. No
syntax, so tokens don't apply; there is no runtime cost without the flag
(criterion 4).

## Consequences

- Each test gets a fresh scheduler seeded from the same seed, so a single
  test replays alone or in its file.
- A seed's interleaving can change when the program changes; the seed
  replays the same program.

## Native port (2026-10-24)

`sched.ail` implements Option A natively with the same draws (spawn coin,
yield coin, seeded pick of the next task). A seed replays the same program
on every target, but not the interpreter's interleaving. Yield points are in
the runtime calls (spawn, wait, cancel, channel send and receive, mutex
methods, `task.check()`); atomic operations are inline, so a build made for
`--schedule=random` (`AIL_SCHEDULE` set at compile time; part of the build
cache key) calls `rt.det_yield` before each one. `ail test` reads the seed
back from the environment for its report (`conc.sched.2`).

## Rejected alternatives

- Record and replay: large, and real threads still differ between runs.
- Model checking now: exponential; Option A's switch points are its basis
  later.
- Randomising only the ready queue: misses the spawn and yield choices
  that expose lost updates (seed 1 of `bootstrap/tests/schedule/race.ail`).
