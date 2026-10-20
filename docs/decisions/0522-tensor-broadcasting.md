# 0522. Tensor broadcasting: NumPy-style, in `+ - * /` and `add`/`mul`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** Designer (card chosen: "NumPy-style broadcasting", 2026-10-20). Recorded by AI.
- **Topic:** 11 — Operators; standard library (`std.tensor`)
- **Spec rules:** `std.tensor.4` and `std.tensor.10` amended
- **Amends:** [ADR 0344](0344-tensor-operators-and-scalar-broadcast.md) (equal shapes → broadcast shapes)
- **Note:** the lane's block ended at 0509; numbers come from the coordinator's 0520–0525 range.

## Context

ADR 0344 required two tensors in `+ - *` to have equal shapes, so `x + bias`
(a `[n]` vector against an `[m, n]` matrix) needed a `map` or an explicit
copy. That is the most common tensor pattern in agent code.

## Decision

Shapes broadcast NumPy-style. The shapes line up from the right; per axis
the sizes must be equal, or one must be 1, which stretches to the other size.
A missing leading axis counts as 1. The result has the broadcast shape.
Shapes that don't broadcast trap (`std.tensor.4`). `dot` keeps equal shapes.
The scalar forms (`t * 2.0`) are unchanged.

## Options considered

The designer chose from three cards:

- **Option A — NumPy-style broadcasting (chosen by the designer).** Saves
  tokens on bias and scale patterns (criterion 1 of ADR 0076); the rule is
  widely known.
- **Option B — keep equal shapes, add `t.broadcast_to(shape)`.** Explicit, but
  more tokens for every bias add.
- **Option C — only scalars and shape-`[1]` tensors broadcast.** Narrower; it
  would not cover `[n]` against `[m, n]`.

## Consequences

- Two operands that broadcast to a bigger shape allocate the expanded copies
  before the kernel runs. The kernel stays the same; a stride-based kernel can
  replace the copies later without changing results.
- The bytecode tier and the tree-walker both reach this through `zip`, so
  there is one rule.

## Measurements

No syntax changed, so tokbench is not run. The change is one shape check on
the equal-shape path, which skips the expansion.

## Rejected alternatives summary

`broadcast_to` only (more tokens per use), scalars-and-`[1]` only (does not
cover a bias row), and no change (leaves the `map` workaround).
