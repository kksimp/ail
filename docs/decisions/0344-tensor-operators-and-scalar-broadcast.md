# 0344. Tensor operators: `+ - *` elementwise with scalar broadcast; matrix product stays `matmul`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 25 — Optimization strategy; 10 — Traits and operators
- **Spec rules:** [std.tensor.10](../spec/17-std.md)
- **Extends:** [ADR 0328](0328-std-tensor-cpu-api.md), [ADR 0329](0329-std-tensor-kernels-and-dispatch.md); keeps [ADR 0077](0077-operator-traits.md)'s fixed operator set

## Context

Numeric code written by agents reads `a.add(b).mul(c)` where every other
language writes `(a + b) * c`, and scaling by a number needs `map`, which
runs the interpreter once per element. The user-facing operator traits for
arithmetic aren't specified yet (spec 10, "not yet designed"), so
`std.tensor` gets the operators directly, the way `int` and `float` have
them, dispatched statically (`trait.op.3`).

## Options considered

### Option A — `+ - *` on two tensors or a tensor and a scalar (chosen)
Elementwise, equal shapes or a scalar of the element type on either side;
the scalar case has its own AVX2/NEON kernel (`set1` plus one operation),
so it rounds exactly like `map(|x| x * s)` and is bit-identical on every
target. `t * 2` works on a float tensor like `x * 2` on a float.
- **Pros:** Fewer tokens (tokbench, mean of five tokenizers): `a + b` is
  4.0 against `a.add(b)`'s 5.2; `t * 2.0` is 6.8 against
  `t.map(|x| x * 2.0)`'s 12.8; perfbench `tensor_ops.ail` is 163.2 written
  with operators against 171.0 with methods (−4.6%), and its Python twin
  246.6. Reads like the math. `t * 2.0` on a million elements runs in
  0.025 s against 0.50 s for the `map` form (Linux x86-64), since `map`
  runs the interpreter per element.
- **Cons:** A rule specific to one standard type until arithmetic traits exist.

### Option B — Full NumPy broadcasting (rejected for now)
`[3, 1] + [1, 4]` shapes. Rejected tonight: silent shape stretching is a
classic source of wrong results that AI agents produce and don't notice
(ADR 0076 criterion 2), and it needs its own kernels and rules.

### Option C — `a @ b` for matrix product (rejected)
`@` already starts attributes (`@inline`, `@deprecated`), so a binary `@`
makes a line that begins with `@` ambiguous to readers and to the parser,
and `trait.op.2` fixes the operator set (ADR 0077, a designer decision).
`matmul` is rare in the agent corpus, so the token gain is under the 2% tie
threshold. Left as [OQ-113](../open-questions.md#oq-113--a-matrix-product-operator).

### Option D — `/` and unary `-` too
Deferred: not in the brief, and `/` raises the division-by-zero question for
`int` tensors. `t.map(|x| x / d)` covers it.

## Decision

Option A. Decided by ADR 0076 criterion 1 (tokens), with criterion 2 against
Options B and C.

## Consequences

`a.add(b)` and `a.mul(b)` stay as methods. When arithmetic operator traits
are designed, tensors implement them with these semantics.
