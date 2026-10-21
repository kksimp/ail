# 0831. `tensor.from_fn(shape, |i| e)`

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the integrator approved a library-only constructor)
- **Topic:** 17 — Standard library (`std.tensor`)
- **Spec rules:** `std.tensor.25` — new

## Context

The NumPy twins in `tools/perfbench/numpy` use 8–40% more tokens than NumPy.
Part of it is building the input: `tensor.from([e for i in 0..n * n], [n, n])`
writes the length twice, and the list and the shape are two arguments.
ADR 0076 ranks tokens first; the integrator asked for a library change with no
new syntax and no new operators, and a cut of at least 2% on the twins.

## Decision

`tensor.from_fn(shape, |i| e)` builds a tensor whose element at flat
(row-major) index `i` is `e`. `shape` is an `int` (1-D) or a `List[int]`. The
element type is the function's result: `int`, `float` or `f32`. Errors are
std.tensor.25 at compile time, and a bad shape traps with std.tensor.2.

## Options considered

- **`tensor.range(n)` and arithmetic on the result.** Rejected: the twins need
  `i % 17`, which tensors do not have, and a new operator is out of scope.
- **`tensor.arange(lo, hi)`.** Rejected: the same, and the caller still needs
  `.map`.
- **A comprehension form inside `tensor.from` (`tensor.from(e for i in n)`).**
  Rejected: new syntax.
- **A 2-D index `|i, j|`.** Rejected for now: the flat index is what the
  comprehension used, `i / n` and `i % n` recover the rest, and one parameter
  keeps the closure form the same as `map`.

## Consequences

- The three twins' AIL files drop from 604.2 to 588.8 tokens (-2.5%, the
  five-tokenizer mean): the chain -0.4, the matmul -9.6, softmax -5.4.
- `from_fn` runs the function on the walker once per element, so it is about
  30% slower than a comprehension on the bytecode tier for 1,000,000 elements
  (about 0.13 s against 0.10 s). Tokens decide (ADR 0076); a faster path would
  be a later change.

## Measurements

tokbench, five-tokenizer mean of the three NumPy twins' AIL files, before
and after rewriting their constructors: 604.2 to 588.8.

## Rejected alternatives summary

`range`, `arange`, a comprehension form, and a 2-D index.
