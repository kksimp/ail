# 0635. What of `std.tensor` is in the native subset

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.tensor.1](../spec/23-native.md)
- **Builds on:** [ADR 0633](0633-native-tensor-kernels.md), [ADR 0634](0634-native-tensor-traps.md)

## Decision

native.subset.1 includes, for `tensor.Tensor[float]`:

- the type in signatures and locals; `tensor.zeros(shape)`,
  `tensor.from(xs)`, `tensor.from(xs, shape)` with `xs` a `List[float]`;
- `shape`, `len`, `to_list`, `reshape`, `sum`, `mean`, `max`, `dot`, `add`,
  `mul`, `matmul`, `transpose`, `relu`, and `sum`/`max`/`mean(axis)`;
- the operators `+ - * /` between tensors (broadcasting) and with a
  `float` or an `int` literal on either side, unary `-`, `==`,
  `!=`, `+=` and the other compound forms;
- `print` of a tensor.

Everything else (`Tensor[int]`, `tensor.from` of an `int` list, `map`,
`t[i]`, slices, `item`, `conv2d`, `argmax`, `argmin`, `clamp`, `round`,
`sigmoid`, `tanh`, fused chains) is reported at its position under
native.subset.1 and nothing is built. `Tensor[int]` waits on checked integer
kernels; `map` waits on lane 174's closures.

## Alternatives rejected

- **Building the unsupported calls with a fallback to the interpreter.**
  The native backend has no interpreter (ADR 0032, ADR 0022).
