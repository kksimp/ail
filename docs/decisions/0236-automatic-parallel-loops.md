# 0236. Run provably independent loops in parallel, with results identical to sequential

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 11 — Concurrency; 25 — Optimization strategy
- **Spec rules:** [conc.par.1–4, conc.par.6](../spec/11-concurrency.md)
- **Extends:** [ADR 0026](0026-hardware-feature-acceleration.md) (accelerators never change semantics), [ADR 0163](0163-inferred-effect-sets.md) (effect sets)
- **Design:** [automatic-parallelism.md](../design/automatic-parallelism.md)

## Context

The designer (2026-10-20) wants AIL to use multi-core CPUs automatically,
optimised for at least four cores, and chose "automatic where provable, plus
a `par` keyword". Mutable value semantics (ADRs 0060–0063) and effect
inference (ADR 0163) make independence provable from the loop body alone.

## Options considered

### Option A — Parallel only when the result is bit-identical to sequential (chosen)
- **Pros:** Parallelism is invisible: programs, tests and `--deterministic` runs mean the same on any core count. An AI reasons about every loop sequentially.
- **Cons:** Hand-written float accumulations stay sequential; integer reductions need overflow bookkeeping to trap at the same iteration.

### Option B — Allow reassociation of reductions (like `-ffast-math` or OpenMP `reduction`)
- **Pros:** More loops qualify, simpler runtime.
- **Cons:** Float results depend on the core count and scheduling; tests flake; contradicts ADR 0026's rule that accelerators never change semantics.

### Option C — No automatic parallelism; only explicit tasks
- **Pros:** Nothing to prove.
- **Cons:** Ignores the designer's direction; programs written the obvious way use one core.

## Decision

An implementation MAY run the iterations of an independent loop concurrently
and MUST give the sequential result, including the lowest-index trap
(conc.par.1). Independence is decided by the obligations of conc.par.2–4:
no `io`, `unsafe`, `spawn` or capability effect; no early exit; no
synchronisation; writes only to iteration locals, to elements indexed by a
range loop's variable, or to `int` accumulators added to with `+=` and not
read. `ail effects --parallel` prints each loop's verdict, rule and first
blocking write (conc.par.6).

## Rationale

ADR 0076 criterion 1 (tokens): automatic parallelism costs zero tokens.
Criterion 2 (AI understanding) decides between A and B: a loop whose result
depends on the core count can't be understood locally. Criterion 4 (speed) is
still served: integer reductions, element-wise loops and `sum()` (ADR 0239)
cover the common parallel loops.

## Consequences

- The bootstrap only analyses; it runs every loop in order (values are `Rc`, ADR 0238).
- Native backend: the analysis result feeds loop parallelisation and vectorisation (ADR 0246).
- Not yet recognised: `acc = acc + e`, `-=`, `*=`, min/max patterns, writes through `ref T`. Each stays sequential until added.

## Rejected alternatives

- **Reassociating reductions:** results would depend on the machine.
- **Tasks only:** leaves the obvious loop on one core.
- **Whole-program alias analysis:** unnecessary; value semantics make the loop body sufficient.
