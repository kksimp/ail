# 0958. SPIR-V and MSL compute first, then rendering

- **Status:** Deferred past 0.1.0
- **Date:** 2026-11-05
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI; GPU
- **Spec rules:** none yet; rule IDs are added to [std-gfx.md](../spec/std-gfx.md) when the code lands
- **Requested by:** the designer, who wants AIL to replace C, C++ and Rust, with real-time rendering that uses modern hardware well. Design: [gpu-frame-loop.md](../design/gpu-frame-loop.md). Not implemented yet.

## Context

AIL decided on its own SPIR-V and MSL backends (ADR 0381). The first GPU use is offloading `par` and tensor work, then rendering.

## Decision

Order: (1) compute kernels from `par` loops and tensor ops, lowered to MSL on Metal and SPIR-V on Vulkan, with a cost model that offloads only when transfer plus launch beats the CPU; (2) render batches (`rect`, `sprite`, `text`, `mesh`) on Metal, then Vulkan and Windows; (3) user shader functions written in AIL. Offload is automatic and reported (`ail explain`), and the CPU result must equal the GPU result bit for bit on integer kernels and within a stated tolerance on floats.

## Rejected alternatives

- **Render first.** Compute offload serves more programs and shares the backend.
- **Link an external shader compiler.** ADR 0010.

## Consequences

Large; each step lands behind its own gate. The CPU path stays the reference.
