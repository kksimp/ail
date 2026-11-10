# 0911. Native code runs proven independent loops in parallel by default, past a work threshold

- **Status:** Accepted (2026-11-07, by the integrator after it landed through the gate with its spec rules; proposed 2026-10-25)
- **Date:** 2026-10-25
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); direction from the designer 2026-10-25
- **Scope amended by:** [ADR 0938](0938-auto-parallel-push-builder-loops.md) (push-builder loops, conc.par.16)
- **Amended by:** [ADR 0987](0987-auto-parallel-inner-loops.md) (proven loops inside loops run in order)
- **Topic:** 11 — Concurrency; 25 — Optimization strategy
- **Spec rules:** conc.par.8, conc.par.14, conc.par.13
- **Builds on:** [ADR 0236](0236-automatic-parallel-loops.md), [ADR 0601](0601-native-par-for.md), [ADR 0239](0239-float-sum-fixed-tree.md)

## Context

ADR 0236 allows any provably independent loop to run concurrently; ADR 0601
built the native chunk runner for `par for`. Ordinary loops reached it only
under `ail run --parallel`, and with no cost test beyond "at least twice as many
iterations as workers", so a 16-iteration loop of one add would pay a dispatch.

## Decision

1. **On by default.** The checker's proven loops (`conc.par.2-4`) are lowered
   as `par for` unless `AIL_PARALLEL=0` or the run is deterministic. The task
   runtime links only into a program that has such a loop or uses tasks.
2. **Cost model (conc.par.14).** The lowering counts operations per iteration
   (binary op 1, index 2, method 2, call 8, nested loop body x16). A loop the
   compiler chose goes to the pool only when `iterations x work >= 65536`
   (about 50 microseconds); else it runs in order. A written `par for` keeps
   the ADR 0601 rule.
3. **Semantics unchanged.** Everything ADR 0601 says about traps (re-run in
   order), exact accumulators and atomics holds, so the result and the trap
   are the sequential ones (conc.par.1).
4. **Stage 2, not in this ADR:** fork-join of independent calls. It needs the
   effect sets of ADR 0163 and its own cost model; it gets its own ADR.

5. **Bounded proof.** The native build does not type-check, so the proof loads
   and checks the program a second time. A program over 1.5 MB of source (about
   30k lines; the compiler is 2.4 MB) skips it and runs its loops in order.

   A cache of the proof is not needed: `cache_key` already hashes every source
   file, so an unchanged program skips the whole build; only a changed program
   pays for the proof. Trade-off: a very large app (over 1.5 MB) gets no
   automatic loops until the build reuses a checked program; its `par for`
   still works.
6. **A written `par for` that falls back says why (conc.par.13).** The lowering
   scan names the first construct it refuses; `ail build` and `ail run` print a
   warning, and `ail check` warns the same way: the scan lives in `ast/parscan.ail`, shared with the
   checker.

## Rejected alternatives

- **Keep `--parallel` opt-in:** the designer wants multicore without flags.
- **Measure at run time and adapt:** the first run pays; a static threshold is
  deterministic and easy to explain. Revisit with data from the benchmarks.
- **Threshold on iteration count alone:** a loop of 10^5 trivial adds loses to
  dispatch cost, and a 100-iteration loop of heavy calls wins.

## Consequences

- The lowering's body scan is narrower than the checker's proof, so some
  proven loops still run in order (they stay correct).
- The 65536 constant is a first guess; tune it with `tools/perfbench`.
- `ail run --deterministic` and `AIL_PARALLEL=0` give the one-thread run.
