# 0026. Portable GC and runtime core; ISA-specific hardware features only as optional accelerators

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 6 — GC design; 22 — Runtime architecture
- **Spec rules:** none (implementation policy)
- **Resolves:** OQ-003
- **Extended by:** [ADR 0236](0236-automatic-parallel-loops.md) (parallel loops give sequential results), [ADR 0242](0242-simd-vectorisation.md) (SIMD), [ADR 0243](0243-std-tensor-and-accel-capability.md) (NPUs and GPUs)

## Context

ARM64 offers Top Byte Ignore (free pointer tags), memory tagging (MTE) and
pointer authentication (PAC). x86-64 offers Intel LAM / AMD UAI (pointer
masking, not universal) and AVX-512 on some CPUs. Building the runtime around
either set would penalise the other ISA. This was guardrail 2 in [ADR 0008](0008-isa-priority-policy.md).

## Options considered

### Option A — Portable core plus optional accelerators (chosen)
- **Pros:** Each ISA uses its strengths, and neither penalises the other.
- **Cons:** More code paths to test, and results differ by platform.

### Option B — Design around ARM features
- **Pros:** The fastest ARM runtime.
- **Cons:** A constant x86 overhead. Contradicts lockstep parity.

### Option C — Lowest common denominator
- **Pros:** One code path.
- **Cons:** Leaves measurable performance unused on both ISAs.

## Decision

The GC, allocator and runtime are **correct using only baseline ISA features**
(ADR 0005, ADR 0006). Hardware-specific features may be used only as
**accelerators** behind a single runtime abstraction, selected at build time
(by explicit target flags) or at run time (by feature detection).

## Consequences

- Every accelerated path has a baseline fallback, and both are tested in CI.
- The performance API reports which accelerators were active, so
  cross-platform results are explainable.
- Accelerators may never change program semantics. They only change speed,
  memory use, or extra safety checks (e.g. MTE-based detection in debug runs).

## Rejected alternatives

- **ARM-centric design:** penalises x86. Violates ADR 0008.
- **Lowest common denominator:** wastes available performance.
