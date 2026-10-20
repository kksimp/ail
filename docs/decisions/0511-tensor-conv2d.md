# 0511. `conv2d` on 2-D tensors: valid, stride 1, no padding

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 11 — Operators; standard library (`std.tensor`)
- **Spec rules:** `std.tensor.16` (new)
- **Extends:** [ADR 0328](0328-std-tensor-cpu-api.md), [ADR 0345](0345-tensor-axis-reductions-and-transpose.md)
- **Note:** lane 154's block (0507–0509) was used up by slices 1–3. The plan allocates no numbers after 0509 and 0510 is the licence, so this is the next free number. The coordinator has been asked to confirm it.

## Context

Lane 154 lists `conv2d` among the CPU gaps. Convolution is the first
operation that needs a window over the data, so its shape rule is the part
that needs deciding.

## Decision

`t.conv2d(k)` is the valid 2-D convolution of an `h × w` image with a
`kh × kw` kernel: stride 1, no padding, output `(h − kh + 1) × (w − kw + 1)`.
Both are 2-D `float` tensors, and the kernel must fit. Each output adds its
products in the kernel's row-major order, one at a time, so every target
gives the same sum (std.tensor.5's order rule, applied to a window).

## Options considered

- **Option A — valid, stride 1, 2-D, one method (chosen).** Smallest rule
  that covers the common filter case; the output shape is a sum of the
  inputs' sizes minus one, which an agent can check by reading.
- **Option B — `conv2d(k, stride, pad)`.** Covers more, but every call would
  spell out defaults, and padding modes add rules. Deferred until a program
  needs them.
- **Option C — 4-D `[batch, channel, h, w]` input.** Needed for real networks.
  Rejected for this slice: the channel and batch axes need their own
  broadcasting rules and a kernel layout decision. A later ADR can add it on
  top of this rule.
- **Option D — FFT-based convolution.** Rejected: a different summation order,
  so results would differ from the direct sum, and it needs a larger kernel to
  pay off.

## Consequences

- `conv2d` on `Tensor[int]` is a compile-time error (`std.tensor.16`), as for
  the activations (ADR 0507): `int` has no rounding rule to define here.
- The direct loop is scalar. A vector path can come later and must give the
  same bits.

## Measurements

No syntax changed, so no tokbench run. Not measured against Python in this
slice; the direct loop is O(oh · ow · kh · kw) and the benchmark belongs with
the batch's drag race (lane 143).

## Rejected alternatives summary

Stride and padding arguments (defaults spelled out every call), 4-D input
now (channel and batch rules not decided), FFT (different summation order).
