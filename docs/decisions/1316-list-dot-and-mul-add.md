# 1316. `xs.dot(ys)` on lists of floats in the fixed 16-partial-sum order, and `a.mul_add(b, c)`

- **Status:** Accepted
- **Date:** 2026-11-28
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer approved opt-in
  vectorised float sums and fused multiply-add on 2026-11-28 (issue #1598, item 6) and left the spelling to the lane
- **Topic:** 04 — Types
- **Spec rules:** [type.coll.36, type.num.19](../spec/04-types.md)
- **Relates to:** [ADR 0329](0329-std-tensor-kernels-and-dispatch.md) (the order), [ADR 1142](1142-element-wise-vector-loops-and-list-alias-classes.md)
  (plain loops stay bit-exact), [ADR 0008](0008-isa-priority-policy.md)

## Context

A loop `s += xs[i] * ys[i]` adds left to right, and its result is defined by that order, so it runs one addition
per float-add latency (native.vec.4 leaves it scalar by design). ail-llm's attention scores and norms are such
loops. The designer approved an opt-in: code that asks for it may add in `std.tensor.5`'s fixed order (16 partial
sums, the same bits on every target), and may fuse a multiply-add only where the program says so. No new syntax
without his approval.

## Decision

- **`xs.dot(ys)`** on `List[float]` and `List[f32]` (type.coll.36) adds the products in `std.tensor.5`'s order,
  products rounded before they are added. It is the same value as `tensor.from(xs).dot(tensor.from(ys))`.
- **Slices are read in place.** `q[a..b].dot(k[c..d])`, each sliced operand a variable or a field, checks the
  bounds as a slice does and makes no list. This is the windowed form: a row of a flat matrix, one head of a
  key cache.
- **`a.mul_add(b, c)`** on `float` (type.num.19) is the fused multiply-add: one rounding. `a * b + c` with the
  operators is never fused. Both targets have the instruction in their baseline (FMA3 in x86-64-v3, ADR 0005;
  `fmadd` in ARMv8, ADR 0006), so the bits are the same everywhere.
- `dot` itself does not fuse. In the fixed order a fused sum is not faster (below), and unfused it equals the
  tensor's `dot` bit for bit, so there is one order to know.
- Implementation: `compiler/mir/lower_dot.ail`, `compiler/rt/native/dot.ail`, the kernel `rt.vec_dot16` (four
  ymm sums on x86-64, eight `v` sums on ARM64) and the MIR instruction `fma`.

## Measurements

Tokens (`tools/tokbench/run.sh`, mean of five tokenizers) for one attention score over windows of two lists:

| Spelling | Tokens |
|---|---|
| `q[qb..qb + hd].dot(kc[b..b + hd])` (chosen) | 44.6 |
| `q.dot_at(qb, kc, b, hd)` | 39.6 |
| `math.dot(q, qb, kc, b, hd)` with its `use` | 44.6 |
| the plain loop | 61.0 |
| the plain loop under an attribute `@sum16` | 64.4 |

Speed, x86-64 Linux (cloud Xeon, one core), `tools/perfbench/micro/dot.ail`, ns per multiply-add:

| n | plain loop | loop of `mul_add` | `dot` | `dot` of slices | `dot` against the loop |
|---|---|---|---|---|---|
| 64 | 0.40 | 0.59 | 0.15 | 0.15 | 2.6x |
| 1,024 | 0.51 | 0.99 | 0.105 | 0.105 | 4.8x |
| 4,096 | 0.52 | 1.05 | 0.18 | 0.18 | 2.9x |
| 1,048,576 (memory bound) | 0.59 | 1.19 | 0.56 | 0.56 | 1.1x |

A sum carried through `mul_add` is slower than multiply then add: the fused instruction's latency (4 cycles) is on
the sum's chain, and the plain add's is 2. That is why `dot` does not fuse. The ARM64 kernel and `fma` give the
x86-64 bits under emulation (`tests/native/run/list_dot.ail`, 70 lengths against the order written out); Mac
timings come with the gate run.

Known gap: a list's elements start 8 bytes past a 32-byte boundary, so half of the kernel's 32-byte loads cross a
cache line on x86-64. With both operands aligned the 1,024 case measures 0.076 ns. Rotating the lanes inside the
kernel removes this without changing the result; it is the next step.

## Rejected alternatives

- **An attribute on the function or loop (`@sum16`).** Most tokens, and which loops it changes would be decided
  by what the optimiser recognises: the result of a loop would no longer be readable from the loop (AI
  understanding, first criterion). A method's result is defined by its rule alone.
- **`q.dot_at(qb, kc, b, hd)`.** Five tokens fewer, but one more argument order to remember, and slices already
  say the same thing. The slice form costs nothing at run time.
- **A `std.math` function.** The same tokens as the method, with an import, and `sum` is already a method.
- **Fusing inside `dot`.** Different bits from `Tensor.dot` and no speed gain in this order.
- **Letting the compiler fuse `a * b + c`.** Changes results of existing programs; the designer's decision keeps
  fusing opt-in.
- **`mul_add` on `f32`.** An `f32` is held as a rounded `float`; fusing as `float` and rounding again is not the
  single-precision fused result. Left out until `f32` arithmetic has its own instructions.
- **Vectorising `xs.sum()` in the new order.** `sum` has its own fixed tree (type.coll.12), already independent of
  the target; changing it would change results.
