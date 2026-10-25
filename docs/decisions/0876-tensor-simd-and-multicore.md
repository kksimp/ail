# 0876. Tensor kernels use SIMD, then all cores, automatically

- **Status:** Accepted
- **Date:** 2026-10-24
- **Decided by:** Designer ("auto multi-core + `par`, SIMD auto"; order set 2026-10-24: packed f32, then SIMD, then multi-core); details by AI (delegated, ADR 0076)
- **Builds on:** [ADR 0875](0875-packed-f32-tensors.md), [ADR 0815](0815-native-tensors.md), [ADR 0005](0005-x86-64-baseline-v3.md), [ADR 0006](0006-arm64-baseline-v8-2.md)
- **Topic:** 23 — Implementation (native tensors)

## Context

Tensor kernels were scalar and single-threaded. The designer wants speed without
annotations: SIMD and multi-core chosen by the compiler and runtime, with `par` only
where a program asks for it explicitly.

## Decision

- **SIMD:** element-wise kernels, reductions, matmul and softmax use AVX2 on x86-64
  (the v3 baseline) and NEON on ARM64, in lockstep (ADR 0008). Higher levels use
  runtime dispatch (ADR 0005).
- **Multi-core:** a kernel whose work passes a size threshold splits across the
  runtime's worker pool. Below the threshold it stays on the calling thread.
- **Determinism:** `--deterministic` runs give bit-identical results. Reductions use a
  fixed split and a fixed combine order, whatever the core count.
- No new syntax and no std API change.

Landed in #742 (head 30623975): matmul is 2.3x faster and softmax 1.8x faster on
ail-linux, with 35 tensor tests unchanged.

## Consequences

Tensor speed scales with the machine without source changes. The optimizer lane's
auto-vectorization (ADR 0890 pass 6) shares these SIMD encoders.

## Rejected alternatives

- **`par` required for multi-core:** costs tokens on every hot call and is easy to forget.
- **Thread count varies the reduction order:** results would differ between machines.
