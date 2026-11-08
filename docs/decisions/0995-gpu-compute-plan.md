# 0995. GPU compute: eligibility report, kernel IR, Metal helper process, cost-gated offload

- **Status:** Proposed
- **Date:** 2026-11-08
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI; GPU
- **Spec rules:** `std.gpu.1` to `std.gpu.4` in `docs/spec/std-gpu.md`, added with the code that cites them (layer 1: `std.gpu.1` eligibility, `std.gpu.2` reason codes)
- **Amends:** [0958](0958-gpu-compute-first.md), which this ADR fills in. Design: [gpu-compute.md](../design/gpu-compute.md).
- **Requested by:** the designer, who wants AIL to use modern hardware well, automatically where it pays.

## Context

ADR 0958 fixed the order (compute, then render batches, then shaders) but not how compute is built, how Metal is reached from a native binary with no Objective-C runtime, or how offload stays safe.

## Decision

1. **Four layers**, each testable without a GPU where possible: eligibility report (`ail effects --gpu`, `std.gpu.1-2`), kernel IR with MSL and SPIR-V emitters checked against golden text (`std.gpu.3`), device host, and a cost-gated offload policy (`std.gpu.4`).
2. **Metal through a helper process** (`tools/gpu-host/macos`, same line protocol and shared-mapping transport as ADR 0957; Apple silicon wraps the mapping in an `MTLBuffer` with no copy). `objc_msgSend` through `extern c fn` is rejected. Vulkan is C and is called through FFI.
3. **Precision**: integer kernels and plain `f32` `+ - * /` are bit-exact with the CPU and offloaded unasked; anything else offloads only when the call asks for approximate precision. `--deterministic` and `--deny accel` force the CPU.
4. **Offload only when estimated faster** by a margin (default 1.5x) including launch and transfer; bounds violations come back as the same trap and rule ID the CPU gives.
5. **Lockstep (ADR 0008)**: the CPU path is the reference and the fallback on every target. x86-64 and ARM64 share the compiler layers 1-2 unchanged. Linux and Windows get the SPIR-V emitter and a Vulkan host in step 6 of the plan; until then they run the CPU path and the report says `no-device`. No target loses a feature because the GPU is missing.

## Rejected alternatives

See the design note: `objc_msgSend` FFI, MPS or Core ML kernels, always-offload, silent `f64` to `f32`, a GPU dialect.

## Consequences

Layers 1-2 land first as pure AIL compiler work and gate anywhere. The helper process is also the future render path. No runtime global slot is needed so far.
