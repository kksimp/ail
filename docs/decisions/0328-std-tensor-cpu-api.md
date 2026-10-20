# 0328. `std.tensor`: an eager, value-typed `Tensor[T]` of `int` or `float`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 21 — Standard library; 25 — Optimization strategy
- **Spec rules:** [std.tensor.1–4, std.tensor.8–9](../spec/17-std.md)
- **Extends:** [ADR 0247](0247-std-tensor-and-accel-capability.md) (its first CPU slice)

## Context

The designer wants AIL to use modern CPUs and NPUs automatically. ADR 0247
routes accelerators through a `std.tensor` module, because NPUs run
operator graphs, not loops. This lane builds the CPU slice so programs can
use the API now and the speed numbers are real. Plain AIL loops over lists
run in the tree-walking interpreter at about 10 million elements a second
(measured below); a kernel over a contiguous buffer runs hundreds of times
faster.

## Options considered

### Option A — An opaque value type with methods, built eagerly (chosen)
`tensor.from(xs, shape)`, `tensor.zeros(shape)`; methods `add`, `mul`,
`dot`, `sum`, `matmul`, `map(|x| e)`, `reshape`, `shape`, `len`,
`to_list`. The value holds a contiguous `f64` or `i64` buffer shared on
copy, like a list (copy-on-write, ADRs 0060–0063).
- **Pros:** Methods cost fewer tokens than module calls (`a.dot(b)` against
  `tensor.dot(a, b)`); value semantics need no new rules; eager results
  equal what a lazy graph would give on the exact CPU backend, so laziness
  can come later without changing any program's output.
- **Cons:** No fusion yet: `a.add(b).mul(a)` makes an intermediate buffer.

### Option B — A lazy graph now, with `t.eval()`
- **Cons:** Twice the code for the same CPU results; an `eval` the CPU
  doesn't need costs tokens in every program until a device backend exists.

### Option C — A struct of `shape` and `data List[T]` in `std/types.ail`
- **Pros:** No new interpreter value.
- **Cons:** Each element is an interpreter `Value` (24 bytes, tagged), so
  every kernel would first copy into a buffer; SIMD would buy little.

### Option D — Operators `a + b`, `a * b` on tensors
- **Cons:** Needs operator traits for a std type, a larger checker change;
  `*` would be ambiguous between elementwise and matrix product. Deferred.

## Decision

Option A. `Tensor[T]` with `T` `int` or `float`, declared in
`std/types.ail` with one placeholder field the checker rejects (std.tensor.1)
so it is `Eq` but neither ordered nor a key. The interpreter holds it as
`Value::Tensor(Rc<Tensor>)`. Its text form is the call that rebuilds it.
Mismatched shapes and overflow trap, like an out-of-range index.

## Rationale

Criterion 1 (tokens): `tools/tokbench` on the perfbench program, comments
removed, mean of five tokenizers: tensor version 128.6, plain AIL loops
213.0, Python loops 189.6, so the tensor form is 0.68x Python. Criterion 2
(AI understanding): one type, ten methods, shape errors as traps with both
shapes in the message, and a text form an agent can paste back as code.
Criterion 4 (speed): see ADR 0329.

## Consequences

- Programs can write numeric work as whole-array operations now; later
  backends (ADR 0330) keep the API.
- `zeros` gives `Tensor[int]` only where the expected type says so, since
  the bootstrap has no annotated bindings (`expr.bind.1`); `from` infers from
  its list.
- Not yet: axis reductions, `transpose`, `conv2d`, activations, `f32`,
  operators, broadcasting, indexing (`t[i, j]`). Listed in OQ-090's follow-up.

## Rejected alternatives

- **Lazy graph now:** no CPU benefit, more tokens per program until a device exists.
- **Struct of lists:** each kernel would copy every element first.
- **Operators:** ambiguity of `*` and a larger checker change for a prototype.
