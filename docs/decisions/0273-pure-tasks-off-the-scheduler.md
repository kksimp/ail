# 0273. Pure tasks run on their own threads; tasks with handles keep the scheduler

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 11 — Concurrency
- **Spec rules:** [conc.par.10](../spec/11-concurrency.md)
- **Relates to:** [ADR 0186](0186-deterministic-task-scheduler.md) (lane 22's scheduler), [ADR 0272](0272-interpreter-thread-pool.md)

## Context

Lane 22's scheduler runs one task at a time in a fixed order, which makes
channels, mutexes, deadlock traps and `--deterministic` reproducible. A task
that only computes doesn't need that order: nothing can observe when it runs.

## Options considered

### Option A — Pure tasks off the scheduler, the rest on it (chosen)
A task whose function has no ordering effect (conc.par.2) and touches no
handle, through every call, with handle-free arguments, starts on its own
thread at `spawn`; `wait`, `wait_all` and the block's end join it.
- **Pros:** Compute tasks use every core; every guarantee of ADR 0186 stays for tasks that communicate.
- **Cons:** Two kinds of task in the runtime.

### Option B — Make the whole scheduler parallel
- **Cons:** Loses ADR 0186's determinism and needs `Arc` values.

### Option C — Only `TaskGroup` tasks
- **Cons:** `a := spawn f(); a.wait()` is the same computation; no reason to treat it differently.

## Decision

Option A, using lane 39's effect and handle analysis (`pure_fns`). The
pooled task's result or trap comes back on `wait` as on the scheduler
(`Trapped`, conc.task.6). A pooled task that returns a handle traps
(conc.par.10). `--deterministic` and `AIL_THREADS=1` keep every task on
the scheduler.

## Rationale

Criterion 4 (speed) without cost to criterion 2: the program's meaning,
output and traps are unchanged.

## Consequences

- `tasks::shutdown` joins pooled threads before the program is dropped.
- A pure task that creates its own channel and blocks on it has its own scheduler, whose deadlock trap (conc.task.7) still applies.

## Rejected alternatives

- **Parallel scheduler:** gives up determinism.
- **Groups only:** an arbitrary line.
