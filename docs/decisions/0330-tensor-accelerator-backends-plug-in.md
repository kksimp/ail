# 0330. How an NPU or GPU backend plugs into `std.tensor` (design)

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 25 — Optimization strategy; 18 — Effects
- **Spec rules:** none yet (design only)
- **Extends:** [ADR 0247](0247-std-tensor-and-accel-capability.md), [ADR 0328](0328-std-tensor-cpu-api.md), [ADR 0329](0329-std-tensor-kernels-and-dispatch.md)
- **Open question:** [OQ-090](../open-questions.md) (may driver compilers compile AIL-generated code?)

## Context

The CPU slice (ADRs 0328–0329) is eager and exact. NPUs (Core ML, Windows ML,
Linux `accel` drivers) take operator graphs in fp16 or int8; GPUs take
kernels the OS driver compiles. ADR 0010 forbids external backends, so a
device backend can't start until OQ-090 is answered.

## Options considered

### Option A — A backend interface behind the same API, opt-in by precision (chosen)
A backend implements: `supports(op, dtype, shape) bool`, `compile(graph)`,
`run(compiled, inputs) outputs`. Tensor methods record into a graph when the
program asks for approximate results (`t.approx()` or `f16`/`i8` tensors);
exact tensors keep running eagerly on the CPU kernels. At the first read
(`to_list`, `sum`, printing) the runtime partitions the graph: the largest
supported subgraphs go to the NPU, then the GPU, the rest to the CPU.
Reaching a device gives the `accel` capability (`--deny accel` forces the
CPU; `--deterministic` denies it).
- **Pros:** Programs don't change; exact programs never touch a device; one
  graph form serves every device.
- **Cons:** Two execution modes to test; device results need tolerances in tests.

### Option B — Each device as its own module (`std.coreml`, `std.vulkan`)
- **Cons:** Per-platform programs; agents must pick devices by hand.

### Option C — Offload exact tensors when the device is exact enough
- **Cons:** No NPU is IEEE `f64`; would change results silently (ADR 0026).

## Decision

Option A. What OQ-090 needs decided before any backend: (1) whether the
driver's compiler counts as part of the platform (OQ-090 option a), which
every GPU and NPU path needs; (2) which graph format AIL emits itself
(recommended: one AIL-side graph IR lowered to ONNX for Windows ML, to an ML
Program for Core ML, and to SPIR-V for Vulkan by AIL's own emitters); (3)
the precision API's spelling (`t.approx()` against `Tensor[f16]`, measured
with tokbench when built); (4) whether `accel` is denied by default in
`ail test`. Order of work: Core ML on macOS ARM64 first (the designer's
machine), after Objective-C FFI; then Windows ML; then Vulkan.

## Rationale

Criterion 2 (AI understanding): device use is visible as a capability and
never changes an exact result. Criterion 4 (speed): ML-shaped work reaches
the NPU with no change to the program.

## Consequences

- The CPU API is frozen in shape: new operations get CPU kernels first.
- OQ-090 stays the gate; it is the designer's decision (it reads ADR 0010).

## Rejected alternatives

- **Per-device modules:** non-portable programs.
- **Offloading exact tensors:** silent result changes.
