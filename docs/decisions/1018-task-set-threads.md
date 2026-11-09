# 1018. `task.threads()` and `task.set_threads(n)`: the pool's cap at run time

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 11 — Concurrency; 22 — Runtime
- **Spec rules:** conc.par.18
- **Builds on:** [ADR 0946](0946-adaptive-thread-pool.md)

## Context

Clove's `--threads N` had to set `AIL_THREADS` before the process started.
A program could neither read the cap nor change it from a command-line flag.

## Decision

1. `task.threads() int` returns the cap. At start it is `AIL_THREADS`, or the
   cores the process may use, at most 64.
2. `task.set_threads(n int)` clamps `n` to 1..64 and stores it in the pool
   block's existing cap word (`P_TARGET`). There is no new global slot.
   `par_worth` and `par_width` already read that word once per region, so the
   par hot path is unchanged.
3. Lowering the cap: a worker checks the cap before it takes a queued task.
   Once more workers are running than the cap allows, it sleeps instead.
   Raising the cap wakes sleepers, and new workers start as before.
4. A call while a parallel region runs traps. Regions are counted in the pool
   block (`P_REGION`): one atomic add when a region starts and one when it
   ends.
5. `set_threads` has the `io` effect. An independent loop, a fork-join call or
   a pure function therefore can't call it, and the trap covers only the
   dynamic cases: a spawned task while `main` runs a region.

## Alternatives rejected

- **A new runtime global slot:** the pool block already holds the cap.
- **Chunk count only:** fewer chunks don't stop idle workers from taking them,
  so the cap wouldn't hold.
- **Ignoring calls inside a region:** a silent no-op hides a bug. The trap
  names it.

## Consequences

Results never depend on the cap (conc.par.1). `AIL_THREADS` remains the
startup default.
