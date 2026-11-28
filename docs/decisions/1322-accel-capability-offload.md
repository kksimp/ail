# 1322. `--accel` and the `accel` capability: offload of parallel comprehensions to the GPU

- **Status:** Accepted
- **Date:** 2026-11-28
- **Decided by:** the designer chose the `accel` capability (2026-11-28); the rest AI (delegated, ADR 0076)
- **Topic:** GPU; Effects
- **Spec rules:** std.gpu.6 ([std-gpu.md](../spec/std-gpu.md)); `eff.cap.2`
- **Extends:** [ADR 0247](0247-std-tensor-and-accel-capability.md), [ADR 0958](0958-gpu-compute-first.md), ADR 1319

## Context

ADR 0247 rejected offloading ordinary loops because device results differ from the CPU's (single precision) and would silently change exact programs, and reserved an `accel` capability for approximate work. kksimp/ail#1599 asks for exactly that for `par` loops, so ail-llm can run on the Mac GPU while the kernel stays an AIL function.

## Decision

A parallel comprehension `[f(a, b, r) par for r in 0..n]`, where `f` returns a `float` and its body is in the subset of `compiler/msl`, runs as one Metal kernel when the program is built with `--accel` (`AIL_ACCEL=1`) on macOS. The comprehension is kept as the fallback: the runtime host (`compiler/rt/native/gpu.ail`) declines when Metal is missing, `n` is under 256, the kernel does not compile or the run fails, and the CPU loop runs. `accel` joins the capabilities `--deny` takes: `--deny accel` and `--deterministic` keep every loop on the CPU, whatever `--accel` says. Without `--accel` nothing is offloaded (opt-in, because `f32` rounding changes results; making it default is the designer's later call). `AIL_PAR_LOG=1` prints one line per decision. The build cache key includes the flag.

## Rejected alternatives

- **On by default:** changes the float results of existing programs on macOS.
- **A source attribute (`@gpu`):** new syntax, which the project does not add for this.
- **A std module the program calls (`std.gpu`):** kept for explicit use (ADR 1319), but the loader only includes a module a program names, and the offload needs a cache of pipelines and weight buffers across calls, which needs a runtime global (slot 69).

## Consequences

Only `float` results over `float` lists, `bytes`, and `int`/`float` scalars (struct fields flattened) are offloadable; integer-list kernels, `f64` precision and inclusive ranges are later. A kernel's integer math is 64-bit, so it matches the CPU except where the CPU would trap.
