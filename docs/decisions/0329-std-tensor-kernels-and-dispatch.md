# 0329. Tensor kernels: AVX2, NEON or scalar picked at run time, one specified summation order

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 25 — Optimization strategy; 2 — Targets
- **Spec rules:** [std.tensor.5–7](../spec/17-std.md)
- **Extends:** [ADR 0005](0005-x86-64-baseline-v3.md), [ADR 0006](0006-arm64-baseline-v8-2.md), [ADR 0008](0008-isa-priority-policy.md), [ADR 0026](0026-hardware-feature-acceleration.md), [ADR 0272](0272-interpreter-thread-pool.md)

## Context

`sum`, `dot`, `add` and `mul` should use the vector unit on both
architectures (lockstep, ADR 0008), and `matmul` the cores (lane 49's pool).
Floating-point addition isn't associative: a vector sum adds in a different
order from a left-to-right loop, and different vector widths (AVX2 holds four
`f64`, NEON two) would give different results on different machines.
ADR 0026 says acceleration never changes semantics.

## Options considered

### Option A — Specify one order every backend follows (chosen)
Sixteen partial sums, element `i` into `i % 16`, combined by halving, then
the tail in order; no fused multiply-add. AVX2 keeps four 4-wide registers,
NEON eight 2-wide ones, and the scalar fallback sixteen `f64`s. All three are
bit-identical (a unit test checks it); the result is a function of the input
only.
- **Pros:** Same output on x86-64, ARM64, with or without AVX2; enough
  independent chains to hide the adder's latency.
- **Cons:** Differs in the last bits from a plain left-to-right loop.

### Option B — Left-to-right order
- **Cons:** One dependency chain: no vector speedup for `sum` and `dot`.

### Option C — Whatever the hardware does
- **Cons:** Results vary by machine; breaks ADR 0026 and reproducible tests.

### Option D — Fused multiply-add in `dot` and `matmul`
- **Cons:** Rounds differently from `a * b + c`; the scalar fallback would
  need software FMA to match. Reconsider with an `approx` precision mode.

## Decision

Option A, with detection once per run (`is_x86_feature_detected!("avx2")`,
`is_aarch64_feature_detected!("neon")`) and `AIL_TENSOR_SCALAR` to force the
fallback for tests. `matmul` computes each row as `c += a(i, p) * b(p, :)`
for `p` in order (vectorised across `j`), which is exactly the triple loop,
and splits rows across `parallel::threads()` when `m * k * n >= 64³`. `int`
kernels are scalar and checked, because overflow traps (AVX2 has no 64-bit
multiply, and the checks dominate).

## Rationale

Criterion 2 (AI understanding): a result an agent sees once is the result
everywhere. Criterion 4 (speed), measured on a 4-core 2.1 GHz Xeon (x86-64),
bootstrap release build:

| Work | Plain AIL loop | Tensor | Python, no NumPy |
|---|---|---|---|
| perfbench `tensor` (160³ matmul + add, mul, sum, dot), whole program | 0.885 s | 0.015 s (59x) | 0.361 s (24x) |
| `dot` of 2,048 floats, per element, L1-resident | ~100 ns | AVX2 ~0.18 ns, scalar ~0.34 ns | |
| 512³ `matmul` | | 1 thread 31 ms, 4 threads 11–14 ms (2.5x), scalar 1 thread 38 ms | |

Memory-bound vectors (4M elements) run at memory speed on every backend.

## Consequences

- macOS ARM64 runs the NEON path; it compiles here (`--target
  aarch64-unknown-linux-gnu`), and `./ail selftest` on the Mac runs the
  bit-identity unit test on it.
- Kernels are Rust in the bootstrap; the native runtime will write them in
  AIL (ADR 0022), with the same order.
- Cache blocking for large `matmul`, and pooling elementwise kernels on very
  large tensors, are later work.

## Rejected alternatives

- **Left-to-right:** forgoes the vector unit for reductions.
- **Hardware order:** machine-dependent results.
- **FMA:** different rounding from the scalar path.
