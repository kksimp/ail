# 0507. Tensor `/` and the activations `relu`, `sigmoid`, `tanh`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 11 — Operators; standard library (`std.tensor`)
- **Spec rules:** `std.tensor.5` (now `add`, `mul` and `/`), `std.tensor.10` (`/` added), `std.tensor.13` (new)
- **Extends:** [ADR 0344](0344-tensor-operators-and-scalar-broadcast.md), [ADR 0328](0328-std-tensor-cpu-api.md)

## Context

Lane 154 of batch 2 closes the CPU gaps of `std.tensor` (ADR 0381 allows
device backends later). This slice covers the parts that need no new
syntax: division of `float` tensors, which the checker rejected, and the
three activations that neural-network code needs most. `conv2d`, `f32`,
indexing and slicing, broadcasting, unary `-` and operator fusion are
later slices.

## Decision

1. `a / b` is elementwise division on `float` tensors, with the same
   shape and scalar rules as `+ - *` (`t / 2.0`, `8.0 / t`). IEEE division
   gives `inf` and NaN, with no trap, as `float` `/` does.
2. `/` on a `Tensor[int]` is a compile-time error (`std.tensor.10`).
   Integer division is not elementwise-defined: it needs a choice of
   rounding and a rule for zero, which the designer has not decided.
3. Methods `relu()`, `sigmoid()` and `tanh()` on `float` tensors, with no
   arguments, return a tensor of the same shape (`std.tensor.13`).
4. The activations run scalar code on every ISA. `relu` is exact.
   `sigmoid` and `tanh` use the platform's `exp` and `tanh`.

## Options considered

### Division
- **Option A — elementwise `/` on floats (chosen).** Same rule as `*`;
  one more kernel case in the AVX2 and NEON `apply` matches.
- **Option B — keep the error and ask for `t.map(|x| x / d)`.** Longer to
  write on every use; the token count was not measured.
- **Option C — `int` division too.** Rejected: rounding and zero rules are
  open questions, and the checker's int-tensor overflow trap would need a
  second trap for zero.

### Activation form
- **Option A — methods `t.relu()` (chosen).** Matches `sum()`, `max()`,
  `transpose()`; one way to call an operation on a tensor.
- **Option B — free functions `tensor.relu(t)`.** Two call styles for one
  type; more tokens to read the same operation.
- **Option C — `t.map(|x| ...)` only.** Works, but `map` runs an AIL
  closure per element. Not measured against the method.

### Kernel for activations
- **Option A — scalar code on every ISA (chosen).** Lockstep (ADR 0008) is
  trivially true, and the same code path is used everywhere.
- **Option B — AVX2 and NEON `exp` polynomials.** Rejected for now: a
  hand-written `exp` needs its own error analysis and tests, and the
  activations are not yet the bottleneck. A later slice can add them with
  a bit-identity test against the scalar path.
- **Option C — `libm` vector calls.** Rejected: not bit-identical across
  operating systems.

## Consequences

- `sigmoid` and `tanh` may differ in the last bit between Linux and macOS.
  `std.tensor.13` says so. Tests print only values that are exact on both
  (`0.5` for `sigmoid(0.0)`), or compare with a tolerance.
- The `std.tensor.5` summation order is unchanged. Division adds no sums.

## Measurements

Tokens: no syntax changed, so `tools/tokbench` is not run for this slice.

Speed (`ail run`, release build, Linux x86-64; Python 3, `python3 -I`).
Each program builds a 1,000,000-element list and does four passes
(`relu`, `sigmoid`, `/`, `tanh`), with equal work on both sides:

| Program | Time |
|---|---|
| AIL `tensor` methods | 0.25–0.32 s |
| CPython, list comprehensions | 0.39–0.50 s |

## Rejected alternatives summary

Integer `/` on tensors (open rounding and zero rules), free-function
activations (second call style), activation-by-`map` only (per-element
closure cost, not measured), vector `exp` now (needs its own accuracy and identity tests).
