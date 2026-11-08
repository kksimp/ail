# 0946. Native: a pool sized to every core, and a region uses the workers its work repays

- **Status:** Accepted
- **Date:** 2026-11-05
- **Decided by:** AI (delegated, ADR 0076; the designer asked on 2026-11-05 for threading across as many cores as are available, never more than is efficient)
- **Topic:** 11 — Concurrency; 22 — Runtime
- **Spec rules:** conc.par.9, conc.par.17, native.task.1
- **Amends:** [ADR 0599](0599-native-threads-and-task-pool.md)

## Context

ADR 0599 capped the pool at 8 threads. On a 16-core machine a scaling loop used
half the cores. Raising the cap alone would make small loops pay to wake threads
they cannot use.

## Decision

1. The pool's size is the cores the process may use, up to `MAXW` (64, the
   stack region ADR 0599 already reserves). `AIL_THREADS` still overrides it.
2. A region (an auto or written `par for`, a push-builder loop, a tensor kernel)
   uses `n / ceil(65536 / work)` workers, at least 2, at most the pool size,
   where `work` is the compiler's per-iteration cost (conc.par.14) and 65536
   operations is about 50 microseconds. Chunks stay four per worker used.
   When the cost is only a floor (the body has a loop, or calls a function
   that loops or recurses), the region uses the whole pool: a static count
   of 16 per nested loop would otherwise put a stencil's rows on 3 workers.
3. `AIL_PAR_LOG` prints one line per region: iterations, work, workers used of
   the pool, chunks. This is what an agent reads to see why N threads ran.

4. The pool block grows to 32 KiB and its per-thread tables are laid out for
   65 slots: the text builders (`P_SB`) and pending failures (`P_ERR`) moved
   past the parked registers (`P_REGS`, 80 bytes a slot), and the Windows
   scratch (`P_WTMP`) no longer folds threads past 29 onto one block. With
   the old layout a 22nd thread's text builder overlapped the first failure.

Workers still start lazily, one per queued chunk, so unused cores cost nothing.

## Alternatives rejected

- **Raise the cap only:** small loops would spread over threads for no gain.
- **Measure scaling at run time and adapt:** adds noise and nondeterministic
  timing; the static cost model is reproducible. Left open for a later measured
  feedback step.
- **Per-worker deques now:** the single locked queue holds 4 chunks per worker,
  not per-iteration tasks; revisit if profiles at 32+ cores show contention.

## Consequences

Output never depends on worker count (conc.par.1). The queue lock and the
parallel collector see more threads on big machines; gate on ail-linux (4
vCPU) cannot show 16-core gains, so scaling numbers need a many-core host.
