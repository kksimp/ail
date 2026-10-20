# 0240. Vectorise independent loops on both baselines; never contract to FMA implicitly

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 25 — Optimization strategy
- **Spec rules:** none (backend policy under conc.par.1)
- **Extends:** [ADR 0026](0026-hardware-feature-acceleration.md), [ADR 0005](0005-x86-64-baseline-v3.md), [ADR 0006](0006-arm64-baseline-v8-2.md), [ADR 0008](0008-isa-priority-policy.md), [ADR 0013](0013-compiler-chosen-layout.md)

## Context

Loops that conc.par.2–4 prove independent have no loop-carried dependence
except recognised reductions, which is what vectorisation needs. The
baselines are AVX2+FMA (x86-64-v3) and NEON (ARMv8.2-A), developed in
lockstep.

## Options considered

### Option A — Vectorise within each parallel chunk, width as a backend parameter (chosen)
- **Pros:** One analysis drives both cores and lanes; both ISAs get four f64 lanes in flight (NEON unrolled by two).
- **Cons:** Overflow checks need a vector path plus a scalar redo.

### Option B — Explicit SIMD types only (`f64x4`)
- **Cons:** Ties programs to a width; the obvious loop stays scalar.

## Decision

The backend vectorises independent loops inside each chunk: AVX2 on x86-64
(AVX-512 by runtime dispatch), NEON on ARM64 (SVE/SVE2 by dispatch). Integer
overflow is checked per lane and the chunk redone in scalar to trap at the
right iteration. The compiler never contracts `a * b + c` into a fused
multiply-add, because that changes bits; a future `fma` function is the
explicit form. Compiler-chosen layout (ADR 0013) may store a list of structs
as struct-of-arrays when the loops that use it read fields independently.

## Rationale

Criterion 4 (speed) with no cost to the others: no syntax, and conc.par.1 keeps results exact.

## Consequences

- Explicit SIMD types may still come later for kernels the analysis can't see; they are not needed for the common case.
- The performance report names the active vector level (ADR 0026).

## Rejected alternatives

- **Explicit SIMD types only:** the obvious loop would stay scalar.
- **Implicit FMA contraction:** changes results between targets.
