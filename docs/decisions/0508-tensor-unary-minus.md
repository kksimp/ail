# 0508. Unary `-` on tensors is elementwise

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 11 — Operators; standard library (`std.tensor`)
- **Spec rules:** `std.tensor.14` (new); `trait.op.1` unchanged for scalars
- **Extends:** [ADR 0507](0507-tensor-divide-and-activations.md), [ADR 0344](0344-tensor-operators-and-scalar-broadcast.md)

## Context

Lane 154 lists unary `-` among the CPU gaps. `-t` on a tensor was a
compile-time error (`trait.op.1`: unary `-` needs `int`).

## Decision

`-t` negates each element of a `float` or `int` tensor, keeping its shape
and element type (`std.tensor.14`). A `float` is negated exactly (`-0.0`
results). An `int` element equal to the least `int` traps with
`std.tensor.6`, as the other `int` tensor operations do.

## Options considered

- **Option A — elementwise `-t` (chosen).** Same meaning as the scalar
  operator, and `-t` is the form agents write. It needs one kernel and one
  checker branch.
- **Option B — `t.map(|x| -x)` only.** Works, but costs a closure per
  element and more tokens per use. The token count was not measured.
- **Option C — `0 - t`.** Already works (`std.tensor.10`). It gives `0.0`
  instead of `-0.0` for a zero float, so it differs from `-t`. Rejected as
  the only form, kept as the way to write a subtraction from zero.

## Consequences

- The bytecode tier and the tree-walking interpreter both negate tensors
  (`interp/bytecode.rs`, `interp.rs`); both use `crate::tensor`'s `neg`.
- No new syntax: tokbench is not run for this slice.

## Rejected alternatives summary

`t.map(|x| -x)` only (closure cost), `0 - t` as the only form (differs on
`-0.0`).
