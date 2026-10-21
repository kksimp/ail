# 0798. `to_f32()` and `to_float()`; mixing and traces

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer's choice of `Tensor[f32]` is ADR 0796)
- **Topic:** 17 — Standard library (`std.tensor`)
- **Spec rules:** `std.tensor.21` (mixing), `std.tensor.22` (conversions) — new

## Context

ADR 0796 says mixed `f32` and `float` operands are an error, and names
`t.to_f32()` and `t.to_float()` as the conversions. This ADR fixes what they
accept and how the other tools treat `f32` tensors.

## Decision

- `t.to_f32()` is defined on a `Tensor[float]`: each element is rounded to
  `f32` once (std.tensor.22). On a `Tensor[f32]` it is a compile-time error,
  and on a `Tensor[int]` too.
- `t.to_float()` is defined on a `Tensor[f32]`: each element widens exactly
  (std.tensor.22). The other two cases are compile-time errors.
- A binary operator or `+ - * /` with one `Tensor[f32]` and one
  `Tensor[float]` tensor is a compile-time error (std.tensor.21). The message
  names both types and suggests the two conversions.
- The execution trace (`--trace`, ADR 0265) shows a tensor by its shape and
  element count, so an `f32` tensor traces the same way; no element is printed
  and no JSON encoding of tensor elements exists.

## Options considered

- **Implicit widening from `f32` to `float` in mixed expressions.** Rejected:
  it hides a precision change in the result type; the designer chose explicit
  conversions (ADR 0796).
- **Conversions on `int` tensors (`to_f32`).** Rejected: an `int` tensor
  converts with `map(|x| x.to_float())` first, as `float` ops already do.
- **A `Tensor[f32]` element print in traces.** Rejected: traces show shapes
  for every tensor, and an element dump would be large.

## Consequences

- Mixing needs one conversion call, which is visible in the source.
- The `std.tensor.21` error is the only new mixed-type diagnostic.

## Measurements

tokbench was not run: no syntax changed.

## Rejected alternatives summary

Implicit widening, `to_f32()` on `int` tensors, and element dumps in traces.
