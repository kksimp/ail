# 0121. `for mut x in xs` changes list elements in place

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** Designer
- **Topic:** 2 — Core syntax (expressions); 7 — Ownership
- **Spec rules:** [expr.loop.6](../spec/05-expressions.md), [own.excl.5](../spec/07-ownership.md)
- **Resolves:** OQ-038
- **Evidence:** [design/program-measurements-r1.md](../design/program-measurements-r1.md): −3.2% tokens on the whole `inventory` program; −25% on its loop

## Context

Changing every element of a list needed indexes (`for i in 0..xs.len()`,
`xs[i].qty = ...`), which cost tokens and invite index mistakes.

## Options considered

- **`for mut x in xs` (chosen):** `x` is the element itself. Fewer tokens
  and no indexes. Con: a borrow that lasts a whole loop.
- **Keep indexes** (Go, Swift): nothing new, but more tokens and `[i]`
  noise.
- **A library method taking a closure:** closures aren't designed yet.

## Decision

1. In `for mut x in e`, `e` is a mutable place (`own.place.1`) of type
   `List[T]`. `x` is a mutable place that refers to each element in turn,
   so changing `x` changes the element.
2. The loop body must not use `e`, or any place that may overlap it, other
   than through `x`. This is checked at compile time. An overlap that would
   depend on index values counts as an overlap, so there's no run-time check.
3. Because the body can't use `e`, the list's length can't change during
   the loop. Each element is visited once, in order.

```ail excerpt
for mut item in items
    if item.qty < min
        item.qty = min
```

## Consequences

- This is the same exclusivity idea as a `mut` argument (ADR 0064), lasting
  for the loop instead of a call.

## Rejected alternatives

- **Keep indexes:** more tokens and index errors.
- **Closures:** not designed yet. They may still be added as library methods
  later.
