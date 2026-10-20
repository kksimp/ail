# 0509. Tensor indexing and slicing: `t[i]` drops the first axis, `t[lo..hi]` keeps it

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 11 — Operators; standard library (`std.tensor`)
- **Spec rules:** `std.tensor.15` (new)
- **Extends:** [ADR 0328](0328-std-tensor-cpu-api.md), [ADR 0345](0345-tensor-axis-reductions-and-transpose.md)

## Context

Lane 154 lists indexing and slicing as CPU gaps. A tensor's shape is known
only at run time, so the checker can't say whether `t[i]` is a scalar
(1-D) or a row (2-D). The type must not depend on the rank.

## Decision

1. `t[i]` returns the tensor of rows after the first, with axis 0 removed.
   For a 1-D tensor its shape is `[]`, and `t.item()` reads that element.
2. `t[lo..hi]` and `t[lo..=hi]` return the rows of the range, with every
   axis kept. A missing bound is the start or the end, as for lists
   (`type.coll.10`).
3. The checker types every tensor index as `tensor.Tensor[T]`. A non-`int`
   row index is a compile-time error.
4. Out-of-range indexes and slices trap (`std.tensor.15`). A tensor of
   shape `[]` has no rows.
5. Assigning to an element (`t[i] = v`) is a compile-time error. Tensors
   are values (ADRs 0060–0063); the message says to build a new tensor.

## Options considered

- **Option A — `t[i]` always a tensor; `t.item()` for scalars (chosen).**
  One static type for every rank; the checker needs no shape. Costs one
  method call to read a scalar from a 1-D tensor.
- **Option B — `t[i]` is `T` for 1-D and a tensor otherwise.** Needs the rank
  at compile time. Rejected: the rank is a run-time value, so the checker
  would have to guess or reject common code.
- **Option C — `t.at(i, j, ...)` for elements, no `t[i]`.** Clear for
  scalars, but `t[i]` is what list code uses, and rows are needed anyway.
  Rejected as the only form.
- **Option D — element assignment `t[i] = v` with copy-on-write.** Rejected
  for now: it adds a mutation path to a type whose values are independent
  and whose kernels assume a fixed buffer. The error keeps the value rule
  simple.

## Measurements

No syntax changed. Tokbench not run. The tree-walking and bytecode tiers
both index tensors (`interp.rs` `eval_index`, `interp/bytecode.rs`
`Ins::Index`); the bytecode tier falls back for `lo..hi` as it does for list
slices.

## Rejected alternatives summary

Rank-dependent index type (needs a rank the checker lacks), `t.at(i, j)` as
the only form (not list-like), in-place element assignment (adds a mutation
path to a value type).
