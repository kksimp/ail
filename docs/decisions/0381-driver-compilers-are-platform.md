# 0381. Accelerator drivers may compile code that AIL generates

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** Designer
- **Topic:** 25 — Optimization strategy; 23 — Compiler architecture
- **Spec rules:** none yet (`std.tensor.*` device backends will cite this ADR)
- **Amends:** [ADR 0010](0010-no-llvm-backend.md) (narrowly: the OS driver's
  compiler counts as part of the platform)
- **Resolves:** OQ-090

## Context

ADR 0010 forbids external backends: nothing of LLVM or any other compiler
ships with AIL. GPUs and NPUs can only be reached by handing the OS driver
something it compiles itself: SPIR-V for Vulkan, MSL for Metal, a Core ML
model, an ONNX graph. The CPU slice of `std.tensor` exists (ADRs
0328–0329, 0344–0345), and [ADR 0330](0330-tensor-accelerator-backends-plug-in.md)
keeps device backends plug-in until this question is settled
([automatic-parallelism.md §6.3](../design/automatic-parallelism.md)).

## Options considered

### Option A — the driver's compiler is part of the platform (chosen)
- **Pros:** the only way to reach these devices at all. AIL generates the
  SPIR-V, MSL or graph with its own code generator, so no external
  compiler ships with AIL and nothing of AIL's architecture is shaped by
  one. The same reading already applies to CPU microcode and the OS kernel.
- **Cons:** device results depend on the driver; covered by the `accel`
  capability and the approximate-precision rule (ADR 0247).

### Option B — CPU only, no GPU or NPU ever
- **Pros:** ADR 0010 untouched.
- **Cons:** contradicts the designer's direction that AIL uses modern
  hardware automatically (ADR 0236 and following).

### Option C — vendor libraries as opaque kernels
- **Pros:** fastest to ship.
- **Cons:** AIL would depend on closed vendor code and its licences, against
  ADR 0009 and ADR 0010's intent.

## Decision

Option A. A device backend for `std.tensor` generates the device program
itself (SPIR-V, MSL, Core ML or ONNX) and hands it to the OS driver. The
driver's compiler is platform, like microcode. ADR 0010 stays in force for
everything else: no external compiler, library or IR shapes AIL's own code
generation, and vendor kernel libraries are not allowed.

## Consequences

- Device backends may start, under ADR 0330's plug-in shape and ADR 0247's
  `accel` capability and precision rules. Work order is the AI's (ADR 0076):
  finish the CPU slice gaps first (`conv2d`, activations, `f32`, indexing,
  broadcasting, elementwise fusion), then Vulkan (Linux and Windows) and
  Metal (macOS) in lockstep (ADR 0008), then NPUs through Core ML and ONNX.
- The docs index, status and roadmap are updated in the same change.

## Rejected alternatives

- CPU only, no GPU or NPU ever: it leaves ADR 0010 untouched, but contradicts the designer's direction that AIL uses modern hardware automatically (ADR 0236 and following).
- Vendor libraries as opaque kernels: rejected because AIL would depend on closed vendor code and its licences, against ADR 0009 and the intent of ADR 0010.
