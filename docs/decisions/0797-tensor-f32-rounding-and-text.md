# 0797. `Tensor[f32]` rounding, scalars and text form

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer's choice of `Tensor[f32]` is ADR 0796)
- **Topic:** 17 — Standard library (`std.tensor`)
- **Spec rules:** `std.tensor.21` (`f32` elements) — new

## Context

ADR 0796 adds `tensor.Tensor[f32]` and leaves the details to lane 154: how
results round, what a scalar in an expression is, and how a tensor prints so
that the text rebuilds it.

## Decision

- **Rounding.** Each element of a result is one `f32` operation (`+ - * /`),
  rounded once. Accumulations (`sum`, `dot`, `mean`, axis reductions and
  `matmul`) add in `f32` in std.tensor.5's order: element `i` goes to
  accumulator `i % 16`, and the accumulators halve down to one. No fused
  multiply-add, so a product rounds before it is added.
- **Activations.** `relu` is exact. `sigmoid` and `tanh` are computed in
  `f64` from the `f32` element and the result is rounded once to `f32`.
  The `f64` transcendental functions are the platform's, as for `float`.
- **Scalars.** A `float` scalar (for example `t * 2.0`) is rounded to `f32`
  at the boundary, once per operation. Bounds of `clamp` are rounded the same
  way.
- **Reductions to a scalar.** `sum`, `dot`, `max`, `mean` and `item` give an
  `f32`, exact in the `f64` carried by the interpreter.
- **Text form.** A `Tensor[f32]` prints as its `float` text with `.to_f32()`
  appended: `tensor.from([1.5, 2.0], [2]).to_f32()`. The element text is the
  shortest digits that read back as the same `f32`, so the text rebuilds the
  tensor exactly.
- **Kernels.** AVX2 (8 lanes) and NEON (4 lanes) twins of each `f32` kernel
  sit beside the scalar reference, in lockstep (ADR 0008), with the same
  accumulator order. A unit test checks the dispatched kernels against the
  scalar ones bit for bit. `AIL_TENSOR_SCALAR` forces the scalar path, and
  the selftest can compare the paths.

## Options considered

- **Accumulate `sum` in `f64` and round once.** Rejected: it would make
  `Tensor[f32]` reductions differ from the lane order that `float` uses and
  from the vector kernels; the lane order is the contract (std.tensor.5).
- **Compute every operation in `f64` and round the result (widen the
  kernels).** Rejected: it stores 4-byte elements but gives no vector speed,
  which is the reason ADR 0796 adds the type.
- **Text as `tensor.from32([...])`.** Rejected: a second constructor name
  for one type; `.to_f32()` reads as the conversion it is.
- **A `float` scalar must be an `f32` literal.** Rejected: ADR 0796 keeps
  scalars `float`, so `t * 2.0` stays the same across element types.

## Consequences

- Results of `Tensor[f32]` differ from `Tensor[float]` by rounding only; the
  same program with `to_f32()` added to its inputs gives the `f32` results.
- The text form is longer by `.to_f32()` for `f32` tensors.

## Measurements

tokbench was not run: no syntax changed. The lockstep test is in
`bootstrap/src/tensor32.rs` and `bootstrap/tests/cases/tensor_f32.ail`.

## Rejected alternatives summary

`f64` accumulation for `f32` sums, widened kernels, a `from32` constructor,
and `f32` literals for scalars.
