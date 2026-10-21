# 0796. `Tensor[f32]`: a 32-bit float element type

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** the designer (option "Add Tensor[f32]" on lane 154's "Tensor gaps" card)
- **Topic:** 17 — Standard library (`std.tensor`)
- **Spec rules:** `std.tensor.*` additions, written by lane 154

## Context

`std.tensor` holds `int` and `float` (64-bit) elements. Model weights and
inference kernels are 32-bit; converting on load doubles memory and halves
the vector width of the AVX2 and NEON kernels. Lane 154 asked whether to add
a third element type; the integrator recommended deferring, and the designer
chose to add it.

## Decision

`tensor.Tensor[f32]` is a third element type: 32-bit IEEE floats stored
packed, with the same methods and operators as `Tensor[float]`. Scalars in
expressions are `float` and are converted at the boundary; `to_list()` gives
`List[float]`; `from(xs, shape)` with an `f32` request rounds each element.
Mixed `f32` and `float` operands are an error (`std.tensor.*`): convert with
`t.to_f32()` or `t.to_float()`. Native and interpreter kernels use 8-wide AVX2
and 4-wide NEON lanes for `f32`. ADRs 0797-0798 are lane 154's for the
delegated details (rounding at the boundary, the text form, JSON).

## Rejected alternatives

- **Convert on load, keep `float` only:** simplest, but doubles memory and
  halves kernel throughput for the common inference case.
- **A general `f32` scalar type:** not needed by any program; sized numeric
  types stay as decided in ADR 0194.
