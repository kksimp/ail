# 1319. `std.gpu`: the Metal host runtime

- **Status:** Accepted
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

## Addendum: the MSL emitter (step 2, first part)

`compiler/msl/msl.ail` turns the body of a `par for v in 0..n` into one MSL kernel, as pure text, so it is tested without a GPU: `ail gpu-msl FILE` prints each top-level `par for` as a kernel with its argument list, or why it is not offloaded, and `tools/msl/run.sh` checks that against `tests/msl/*.msl` (the gate suite for it is the integrator's to add). Types come from the caller's environment; the dev command takes them from the function's parameters, the lowering hook (next) will take them from the checker. Subset: scalar and list of `int`, `float`, `f32`, sized ints and `bool`; arithmetic, comparisons, `and`/`or`/`not`; `let`, assignment, `op=`; `if`/`elif`/`else`, `while`, counted `for`, `break`, `continue`; the methods `shr shl bit_and bit_or bit_xor abs min max sqrt to_float to_int to_f32` and `len`. `int` is `long`, so integer kernels equal the CPU bit for bit except where the CPU traps (overflow, division by zero), which the offload policy must rule out before it offloads. A list both read and written, a write at an index other than the loop variable, calls and field reads are rejected, and the loop stays on the CPU. Not yet: pure function calls, struct fields (`m.rows`), packed-weight lists from `fs.map`, the lowering hook and the offload policy.
