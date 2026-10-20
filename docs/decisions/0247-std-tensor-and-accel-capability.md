# 0247. NPUs and GPUs through a `std.tensor` module and an `accel` capability (design)

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 21 — Standard library; 25 — Optimization strategy; 18 — Effects
- **Spec rules:** none yet (design only; the module isn't implemented)
- **Extends:** [ADR 0026](0026-hardware-feature-acceleration.md), [ADR 0165](0165-capability-denial.md)
- **Open question:** [OQ-090](../open-questions.md) (driver compilers and ADR 0010)

## Context

The designer wants NPUs used automatically. Every reachable NPU API (Core ML
on macOS, Windows ML/ONNX Runtime on Windows, per-vendor stacks on Linux)
runs **operator graphs**, usually in fp16 or int8, not arbitrary code. GPUs
run kernels (Vulkan SPIR-V, Metal MSL). Device results differ from the CPU's
in rounding and accumulation order. See the survey in
[automatic-parallelism.md](../design/automatic-parallelism.md) section 6.

## Options considered

### Option A — `std.tensor` with pluggable backends, accelerators only for approximate requests (chosen)
- **Pros:** The operator set matches what NPUs accept; a CPU backend always exists and is exact; automatic device choice for code that asks for approximate results.
- **Cons:** Ordinary loops don't reach the NPU; code has to be written with tensors.

### Option B — Offload ordinary loops automatically
- **Cons:** NPUs don't run arbitrary code; fp16 results would silently change exact programs, breaking ADR 0026.

### Option C — Bind vendor APIs directly (Core ML, DirectML) as libraries
- **Cons:** Per-platform programs; no CPU fallback; no capability control.

## Decision

Design a `std.tensor` module: `Tensor[T]` with `matmul`, `conv2d`,
elementwise operations and activations, axis reductions, `reshape` and
`transpose`, built as a lazy graph and evaluated by a backend (`cpu`,
`coreml`, `winml`, `vulkan`, `metal`). The `cpu` backend is exact and
uses parallel loops and SIMD. A graph goes to an accelerator only when the
program requests approximate precision (`eval(precision: .approx)` or
`f16`/`i8` tensors); the runtime then picks the device. Reaching a device
gives a new capability, `accel`: `--deny accel` forces the CPU, and
`--deterministic` denies it.

## Rationale

Criterion 2 (AI understanding): exact code stays exact, and device use is
visible in the effect set. Criterion 4 (speed): ML-shaped work reaches the
NPU with no per-platform code.

## Consequences

- First prototype: `std.tensor` on the CPU in the bootstrap (about one lane-night). Then Core ML (one to two weeks after Objective-C FFI), Vulkan compute (two to three weeks, needs an AIL SPIR-V emitter), Windows ML (one to two weeks after FFI).
- `accel` joins `eff.cap.*` when the module lands; until then nothing gives it.
- OQ-090 must be settled before any device backend.

## Rejected alternatives

- **Automatic offload of loops:** NPUs can't run them, and results would change.
- **Direct vendor bindings:** no portability or fallback.
