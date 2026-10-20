# 0272. The interpreter runs `par for` on a pool of copied values

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 11 — Concurrency; 25 — Optimization strategy
- **Spec rules:** [conc.par.8–9](../spec/11-concurrency.md)
- **Extends:** [ADR 0236](0236-automatic-parallel-loops.md), [ADR 0237](0237-par-for.md), [ADR 0238](0238-parallel-runtime-model.md)

## Context

The designer asked that AIL use at least four cores. Lane 39 made `par for`
an analysis only, and lane 22's scheduler runs one task at a time, because
the interpreter's values are `Rc` and can't be shared between threads.

## Options considered

### Option A — Copy what the body uses into a thread-safe form (chosen)
Each worker gets its own interpreter and a `Plain` copy (no `Rc`) of the
variables the body reads or writes; it runs chunks of iterations and sends
back only the list elements it changed and its `int` partial sums, which the
main thread applies in chunk order.
- **Pros:** No change to `Value` or to sequential speed; value semantics (ADR 0060) already say each iteration sees copies; measured 3.1× on 4 threads.
- **Cons:** Copying costs O(captured data) per worker, so small loops stay sequential.

### Option B — Make `Value` use `Arc`
- **Cons:** Atomic reference counts slow every program, parallel or not; a large change to every interpreter file.

### Option C — A work-stealing library (rayon)
- **Cons:** A dependency for what `std::thread::scope` and one atomic counter do; ADR 0009 prefers custom tooling, and the native runtime (ADR 0238) is written separately anyway.

### Option D — Speculative parallelism (run in parallel, detect conflicts)
- **Cons:** Needs read and write logs per iteration; the analysis already proves what speculation would check.

## Decision

The pool has `min(hardware threads, 8)` workers, or `AIL_THREADS`. A loop is
split into four chunks per worker, taken in order from a shared counter so a
slow chunk doesn't idle the rest. A loop with fewer than twice as many
iterations as workers runs in order. Nested loops inside a worker run in
order.

## Rationale

Criterion 4 (speed) decides, at no cost to tokens or understanding: the
result is identical (conc.par.1) and the sequential path is unchanged.
Measured on Linux x86-64 with 4 cores, `tools/perfbench/programs/parsum.ail`:
best of five, 2.41 s on 1 thread, 1.43 s on 2, 0.78 s on 4 (3.1×), output
byte-identical; Python 3 takes 1.05 s.

## Consequences

- Only loops whose writes are list elements or `int` accumulators run on the pool (conc.par.8); others stay in order.
- Captured lists are copied per worker; a loop over a huge shared table pays that copy.

## Rejected alternatives

- **`Arc` values:** slows every program.
- **rayon:** a dependency for little code.
- **Speculation:** redundant with the proof.
