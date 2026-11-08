# 0942. Native: vector kernels for swapped reduction nests (ADR 0845 stage 3, first step)

- **Status:** Accepted
- **Date:** 2026-11-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 23 — Implementation (native optimizer)
- **Spec rules:** [native.vec.1](../spec/23-native.md)
- **Extends:** [ADR 0845](0845-vectorization-order-and-idioms.md) (vectorization order), [ADR 0941](0941-data-range-loop-guards.md) (data-range guards and interchange), [ADR 0246](0246-simd-vectorisation.md)

## Context

ADR 0845 put integer lanes after float outer loops and said they need a
per-lane overflow test; AVX2 has no 64-bit multiply (`vpmullq` is AVX-512).
ADR 0941 changes both: its guard proves, before the nest, that no product or
sum in a swapped matmul-shaped nest can overflow and that every element is
within +-2^20, and the swap leaves an inner loop `row[j] += x * b[k][j]` over
contiguous memory: an axpy, which the tensor runtime already has as a kernel
for floats.

## Decision

1. **Idiom, not a general vectorizer.** When the swapped nest's term is
   `x * y`, `y` the element `j` of a list that doesn't depend on `j`, and
   nothing else in the body depends on `j`, `compiler/mir/interchange.ail`
   emits, per `k`, the body's `j`-free part once and one call:
   `rt.vec_axpyi(c, b, n, x, row)` for `int`, `rt.vec_axpyf(c, b, n, row; x)`
   for `float` (`compiler/rt/native/vecint.ail`, AIL bodies as the reference).
   `row` is passed so the optimizer sees the call change it.
2. **Kernels on both ISAs** (ADR 0008). x86-64: eight lanes a pass in two
   ymm registers; `int` by `vpmuldq` (the low 32 bits of each lane, which hold
   the whole value under the guard) and `vpaddq`; `float` by `vmulpd` and
   `vaddpd`. ARM64: four lanes a pass; `int` by `xtn`/`xtn2`, `smull`/`smull2`
   and `add .2d`; `float` by `fmul`/`fadd .2d`. Scalar tails.
3. **Bit-identical.** Each element gets the same operations in the same order
   (`s + x * y`, no FMA, no reassociation, ADR 0246). The float kernel keeps
   the operand order `x * y` and `s + t`, so a NaN keeps the payload it would
   have; a nest written `y * x` or `t + s` runs the scalar swapped loop.
   `AIL_OPT_OFF=vec` turns the kernels off (the swapped scalar loop runs).

## Rationale

Criterion 4 (speed), with no token, semantic or readability cost. An idiom
call reuses the kernel style the tensor runtime already proves on both ISAs,
and the guard does the hard part (no per-lane overflow test). A general
loop vectorizer in the code generators is the larger, later step (ADR 0845
stage 2: masked float outer loops).

## Rejected alternatives

- **Per-lane overflow tests in the kernel** (ADR 0845's plan): two extra
  compares and a scalar redo per pass, unnecessary once the guard holds.
- **Emulating a 64-bit multiply** (three `vpmuludq` and shifts): slower than
  scalar `imul` for these sizes, and not needed within 2^31.
- **Reusing `rt.vec_axpy` for floats:** it computes `b[i] * x`, which can pick
  a different NaN payload than `x * y`.
