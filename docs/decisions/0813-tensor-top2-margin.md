# 0813. Tensor `top2`: best, second-best and margin along an axis

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 11 — Operators; standard library (`std.tensor`)
- **Spec rules:** `std.tensor.23` (top-2 with margin) — new
- **Depends on:** the tuple types (ADR 0791), for the three-part result

## Context

The PDF OCR tool rejects a glyph when its best and second-best scores are too
close. It needs, for each row of scores, the best index, the second-best index
and the margin between them, in one pass.

## Decision

- `t.top2(axis)` on a `float` or `f32` tensor returns a tuple
  `(Tensor[int], Tensor[int], Tensor[T])`: the best index, the second-best
  index and the margin (best minus second), each of the shape without `axis`.
- The best is the first largest element. A tie with the best is the second,
  so the margin of a tie is `0`. A NaN never replaces the best or the second.
- A NaN best traps, and so does a row without a second value. An `int` tensor
  is a compile-time error, and an `axis` with fewer than two elements traps.
  All are std.tensor.23.
- The margin is one IEEE subtraction, rounded once in the element type.

## Options considered

- **One method returning a tuple (chosen).** One pass, one call, and the OCR
  code destructures it (`b, s, m := t.top2(1)`).
- **Two methods, `argmax2` and `margin`.** Rejected: two passes, and the pair
  must be kept in step by the caller. This was the fallback in the prep note,
  used only if tuples had not landed.
- **Return the second value instead of its index.** Rejected: the index is
  what the glyph classifier needs, and the margin already gives the values'
  difference.
- **A NaN best gives a NaN margin instead of a trap.** Rejected: a NaN best
  means the row has no maximum, and the margin would hide it; a trap names
  the row.
- **Int tensors (`int` margin).** Deferred: the OCR use is on `float` scores;
  `argmax` covers the `int` best index.

## Consequences

- A tuple result is a value that needs no new type for the caller.
- The one-pass kernel is generic over `float` and `f32` (ADR 0796, 0797).

## Measurements

tokbench was not run: no existing syntax changed; the method is one name in
the tensor method list. The OCR thread measures the speed of its own call.

## Rejected alternatives summary

Two methods (fallback), second value instead of index, NaN margin for a NaN
best, and `int` margins (deferred).
