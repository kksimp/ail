# 0827. `softmax()` along the last axis

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library (`std.tensor`)
- **Spec rules:** `std.tensor.24` — new

## Context

The activations in std.tensor (ADR 0507) stop at `relu`, `sigmoid` and `tanh`.
`softmax` is the last common one and needs a reduction over an axis, so it
cannot be an elementwise call.

## Decision

- `t.softmax()` takes no arguments and works along the last axis, one row at a
  time. A 1-D tensor is one row. The result has the shape of `t`.
- Each row is `exp(x - m)` divided by the sum of the row's `exp(x - m)`, where
  `m` is the row's largest element. The shift keeps `exp` from overflowing.
  The sum runs in element order.
- A row that holds a NaN or an infinity gives NaN for that row. An empty last
  axis traps.
- An `f32` tensor is computed in `float` and rounded once, as the other
  activations do (std.tensor.21, ADR 0797).
- `Tensor[int]` is a compile-time error (std.tensor.24), with the same
  conversion hint as the other activations.
- Scalar code on every ISA. Fusion (std.tensor.20) does not cover it.

## Options considered

- **`softmax(axis)`.** Rejected: the last axis is the usual case, and one
  argument less is fewer tokens. A transpose reaches the other axes.
- **Whole-tensor softmax.** Rejected: for a batch of rows it mixes rows.
- **Skip the shift by the row maximum.** Rejected: large inputs overflow
  `exp` and give NaN.
- **Pairwise or vector-lane sum.** Rejected: element order keeps results
  identical on every path, and the sum is small next to `exp`.

## Consequences

- One new method and one new rule. The quick reference gains `softmax()` and
  drops the pooling and kernel notes, a net saving of 6.8 tokens.
- A SIMD `exp` would be a later change with its own rule.

## Measurements

tokbench was not run: no syntax changed. Quick reference five-tokenizer mean
12336.6 to 12329.8.

## Rejected alternatives summary

An axis argument, whole-tensor softmax, no shift, and a pairwise sum.
