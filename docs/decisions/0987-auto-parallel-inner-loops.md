# 0987. Native: a proven loop inside a loop run in order may go to the pool

- **Status:** Accepted
- **Date:** 2026-11-08
- **Decided by:** AI (delegated, ADR 0076; the designer asked for threads on every core that pays off)
- **Topic:** 11 — Concurrency; 25 — Optimization strategy
- **Spec rules:** conc.par.14
- **Amends:** [ADR 0911](0911-default-automatic-parallel-loops.md) (outermost loops only)

## Context

ADR 0911 sent only a function's outermost proven loop to the pool, so the
dispatch would not be paid on every run of an outer body. That left the most
common shape of numeric code in order: a time-step, solver or frame loop (a
`while`, or a `for` that carries a value) around a proven loop over the data.
Stencils, Jacobi and Game of Life steps, k-means passes and per-frame pixel
loops all have it. The runtime work test (`rt.par_worth`, 65536 operations,
about 50 microseconds) already holds per region, so each dispatch is repaid
wherever the loop sits.

## Decision

1. A loop the checker proves (`conc.par.2-4`) goes to the pool from any depth,
   unless it is inside a chunk of a loop already on the pool (there it runs in
   order, as before).
2. Inside a loop, the lowering first compares the iteration count with the
   count its per-iteration work needs (`ceil(65536 / work)`, or 2 past 4096)
   inline, so a short inner run costs one compare and no call. A range with
   literal bounds below that count is lowered as a plain loop, with no test.
3. `rt.par_worth` asks for twice as many iterations as the workers the region
   uses (`par_width`, ADR 0946), not twice the pool size, so a large pool does
   not send short heavy regions back in order.

## Alternatives rejected

- **Keep outermost only:** the time-step shape stays on one core.
- **Hoist the pool region around the outer loop** (one dispatch, workers
  synchronised per step with a barrier): fewer wake-ups, but it needs a barrier
  in the runtime and a proof that the outer body's other statements are
  single-threaded; revisit if measurements show the per-step dispatch costs.
- **Decide at compile time only:** bounds are usually run-time values.

## Consequences

- Each pooled inner region re-enters the pool once per outer iteration; the
  gate measures that overhead against main (outer loops of many inner loops
  just above and below the threshold, interleaved best of 11).
- Code size: each proven inner loop gets a chunk function and keeps its
  in-order copy.
- Results and traps are the sequential ones (conc.par.1), as for every loop on
  the pool.
