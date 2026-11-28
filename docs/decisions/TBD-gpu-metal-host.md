# TBD. `std.gpu`: the Metal host runtime

- **Status:** Accepted (number to be allocated by the integrator)
- **Date:** 2026-11-28
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** Standard library (`std.gpu`); GPU
- **Spec rules:** std.gpu.1-5 ([std-gpu.md](../spec/std-gpu.md))

## Context

Step 1 of ADR 0958 (kksimp/ail#1599): offload compute to the GPU so ail-llm can match mlx-lm on Apple Silicon. The first piece is the host side: device, queue, buffers, run-time MSL compile, dispatch.

## Options considered

### Option A — a std module over the Objective-C runtime with `extern c fn` (chosen)
`objc_msgSend`, `sel_registerName` and `objc_getClass` from libobjc and `MTLCreateSystemDefaultDevice` from Metal, as `std.term` uses libc. Every object is an `int`, and `objc_msgSend` is declared with four integer arguments after the receiver and selector, so no call passes more than six integer arguments (`ffi.decl.3`, #1596 is not needed). A `MTLSize` is 24 bytes and so is passed by reference on arm64, which a pointer to three words matches.
- **Pros:** no runtime or lowering change; the lazy binding of `ffi.load` means Linux and Windows programs never load Metal; the MSL generator of step 2 can emit calls to it.
- **Cons:** each message looks the selector up by name (a cache is a follow-up); float-typed Objective-C arguments are not reachable (none are needed); `f32` buffer access goes through `bytes` pushes until a native store is added.

### Option B — runtime functions in `compiler/rt/native/`
- **Pros:** one-time selector cache, direct `f32` stores.
- **Cons:** a global slot, a lowering entry and an arm64-only file for what the FFI already reaches.

## Decision

Option A as `std/gpu/gpu.ail`, marked experimental (ADR 1305). The CPU path is unchanged. Not yet run on a Mac: the first run is the Mac job.

## Rejected alternatives

- Wrap the Metal C++ header or Swift: no C++ or Swift in the toolchain (ADR 0010).
- A GPU library such as MoltenVK or wgpu: ADR 0381, no external backend.

## Consequences

Step 2 (MSL from `par` loops), the offload policy and no-copy buffers from `fs.map` build on this module.
