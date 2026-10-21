# 0636. Native `Tensor[int]`, indexing, clamp, argmax, round and conv2d

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Standard library; 23 — Compiler architecture
- **Spec rules:** [native.tensor.1](../spec/23-native.md), std.tensor.6, std.tensor.15-19
- **Builds on:** [ADR 0633](0633-native-tensor-kernels.md), [ADR 0634](0634-native-tensor-traps.md), [ADR 0635](0635-native-tensor-subset.md)

## Decision

1. **`Tensor[int]` is a second struct**, `tensor.TensorInt` (shape, `List[int]`
   elements), so the lowering knows the element type from the value's type.
   Shape-only operations (`zeros`, `reshape`, `transpose`, `t[i]`, slices,
   `shape`, `len`) are shared with the float tensor: both store 8-byte
   elements. `tensor.zeros` takes its element type from the expected type
   (a return type, a typed binding); with none it is `float`.
2. **Integer kernels are scalar and checked** (std.tensor.6): add, subtract
   and multiply test for overflow before they compute, in the order the
   interpreter computes (`dot` checks every product, then the sum). The
   failure is code 19 of ADR 0634; since the rule of its trap differs
   (std.tensor.6) from the operation's own (std.tensor.4, .7, .11), the
   lowering emits two traps and `rt.tensor_alt` picks one.
3. **Indexing** `t[i]`, `t[lo..hi]`, `t[lo..=hi]`, `t[lo..]`, `t[..hi]` and
   `item()` as std.tensor.15. A second index on a 1-D tensor (`t[0][1]`)
   indexes a tensor of shape `[]`, and traps at run time with "`index` of a
   tensor of shape [], which has no rows", as `ail run` does today
   (OQ-122: the checker accepts it).
4. **`clamp`, `argmax`/`argmin` (flat and along an axis), `round` and
   `conv2d`** as std.tensor.16-19. `conv2d` runs four neighbouring outputs
   at a time with `v4axpy`, each output adding its products in kernel order
   from 0.0, so it is bit-identical. An axis outside the shape traps under
   std.tensor.11 and an empty axis under std.tensor.17, as in the
   interpreter. The `clamp` and `round` trap messages print floats as
   Rust's `{}` does (`t_out_display`).

## Not in the subset

`map` (needs closures, lane 174), `sigmoid` and `tanh` (the interpreter uses
the host's `exp` and `tanh`; bit-identical native results need a port of
that exact libm routine, a decision for the std.math work, ADR 0591), fused
chains (std.tensor.20, a pure speed-up), and threaded `matmul`.

## Alternatives rejected

- **One struct with an element-kind field.** Every operation would test it
  at run time, and the lowering could not give `item()` or `sum()` a type.
- **Approximate `sigmoid`/`tanh`** with a polynomial: results would differ
  from `ail run` in the last bit, which the native tests forbid.
