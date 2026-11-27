# TBD. `std.jobs`: an in-process job queue and scheduler

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for a fuller std on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [std.jobs.1 to std.jobs.12](../spec/std-jobs.md)

The number is a placeholder: the integrator allocates it and renames this file.

## Context

Programs that serve requests or watch files need work done off the main flow: send an email, rebuild an index, sweep a
directory every minute. Agents write this by hand with `spawn`, a `Channel` and `time.sleep_ms`, and the usual first
draft loses jobs on a trap, retries without a delay, or busy-waits. Python has `queue`, `sched` and `celery`; AIL has
tasks, channels and `Heap` but no queue with the policies on top.

## Decision

- A pure AIL module, `std/jobs/jobs.ail`, in scope without `use` as `jobs` (std.use.4). Registration lines only.
- A job is data: a name and a `str` payload. One handler `fn(Job) Outcome` per pool runs it and answers `done`,
  `retry(why)` or `abandon(why)`. Data rather than closures because `spawn` needs a declared function, and a plain
  value can be saved to a file.
- One `Mutex[State]` holds a `Map` of waiting jobs and two `Heap`s of keys: ready jobs by (priority, queue order) and
  delayed jobs by due time. A cancelled job leaves its key in a heap, which is skipped when it comes up. Submit and run
  are O(log n).
- Workers poll: they look at the queue, run a job or sleep until the next due time (at most 20 ms). No timer thread and
  no wake-up channel, so there is nothing to tear down in `stop` but the workers.
- Each run is a task of its own, so a handler that traps fails the job and not the worker.
- Retry: the first retry waits `retry_base_ms`, each later one doubles, capped at `retry_cap_ms`; no jitter, so tests
  are exact. `attempts` bounds the runs, per pool and per job.
- Recurring jobs take an interval. Missed ticks are skipped. Cron expressions are not used because `std.cron` was not
  on main when this was written (PR 1514 open); an entry would only need a function from now to the next due time.
- Persistence is a snapshot, not a log: `jobs.save` writes the waiting jobs to a `std.kv` file, `jobs.load` reads them,
  `resubmit` queues them again. A `std.kv` `Store` is `unique` and cannot live inside the shared `Mutex`, and a snapshot
  needs no change to the hot path.

## Rejected alternatives

- **Handlers by name in a map.** More API (`register`), and the same single-function dispatch inside the handler is
  clearer for an agent to read.
- **Condition variables or a wake channel.** Lower latency for idle pools, but more states to get wrong (lost wake-up,
  stop while asleep); a 20 ms poll costs almost nothing.
- **A write-ahead log through `std.kv` on every change.** Survives a crash exactly, but puts file I/O under the lock
  and needs a `unique` store in shared state. Left for a later lane if a program needs it.
- **Linear scan for the next job.** Simpler, O(n) per run; rejected for queues of thousands.
- **Jitter in the backoff.** Useful against thundering herds across processes; this queue is in one process.

## Consequences

- Handlers run as tasks under the one-at-a-time scheduler rules (conc.par.10); a CPU-bound handler does not spread over
  cores through this module.
- First compile and run are at the Mac gate: the module was written without a toolchain in the authoring environment.
