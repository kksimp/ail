# 0523. Tensor `argmax`, `argmin`, `clamp` and `round`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 11 — Operators; standard library (`std.tensor`)
- **Spec rules:** `std.tensor.17` (argmax, argmin), `std.tensor.18` (clamp), `std.tensor.19` (round) — new

## Context

The PDF OCR tool (`tools/pdf/ocr`) runs per-element interpreter loops for
three operations that a tensor method does in one kernel: the nearest shape
of each glyph (an argmax over a score matrix, about 1.3 million scores a
page), a clamp written as `1 - relu(1 - x)`, and pixel rounding to `int`
(about 4 million values a page). The batch's OCR thread asked for these.

## Decision

- `argmax(axis)` and `argmin(axis)` return the index of the extreme element
  along `axis` as a `Tensor[int]`; with no argument, the flat index as an
  `int`. Ties keep the first element; NaN never replaces the best, the same
  rule as `max` (ADR 0345).
- `clamp(lo, hi)` limits each float element to `[lo, hi]`. NaN stays NaN.
  Bounds that are NaN or with `lo > hi` trap.
- `round()` rounds each float half away from zero to a `Tensor[int]`. A NaN,
  an infinity or an out-of-range value traps.

## Options considered

- **argmax with `axis` as `Option`, matching `sum`/`max`/`mean` (chosen).**
  One method name for the whole reduction family.
- **A separate `argmax_axis` method.** Rejected: a second name for one idea.
- **`round` returning `Tensor[float]` (integral values).** Rejected: the OCR
  use needs `int` for indexing and `to_list` as `List[int]`, and a float
  result would need a second conversion.
- **`round` that wraps out-of-range values.** Rejected: a silent wrap hides
  bad pixel data; a trap names the element.
- **Round half to even.** Rejected: it differs from `f64::round` and from
  the usual rounding of pixel values; half away from zero is the simpler rule
  to state.
- **Top-2 (values and indices) for a confidence margin.** Deferred: the OCR
  thread listed it as "ideally". It needs a tuple result type, which no
  tensor method returns yet. Its own ADR when there's a use.

## Consequences

- argmax on `int` tensors is allowed (the comparison is exact).
- `clamp` and `round` are `float`-only, with compile-time errors for `int`.
- The OCR tool can drop its per-element loops for these three calls.

## Measurements

No syntax changed; tokbench not run. The OCR speedup (0.6 s per page for the
classify loop, the figure the OCR thread gave) is not re-measured here.

## Rejected alternatives summary

Separate `argmax_axis` name, float-valued `round`, wrapping `round`, round
half to even, and a top-2 tuple result (deferred).
