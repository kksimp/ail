# 1320. A list held in a struct field is loaded once before a loop that cannot change it

- **Status:** Accepted
- **Date:** 2026-11-28
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer made ail-llm speed top priority (kksimp/ail#1598, item 4)
- **Topic:** 25 — Optimization strategy
- **Spec rules:** [native.bce.20](../spec/23-native.md)
- **Builds on:** [ADR 0577](0577-bounds-checks-in-counted-loops.md) (bounds checks in counted loops), mutable value semantics ([ADR 0060](0060-mutable-value-semantics.md))

## Context

ail-llm's kernels read weights through struct fields (`m.pk[i]`). Each pass reloaded the field, then the list's
length and element pointer, because the loop's stores to elements looked like they might change the field.

## Decision

In a loop that calls nothing, makes no raw store and stores to no field at that offset, the field load moves in
front of the loop, and `native.bce.11` then reads the list's length and data pointer once, as for a list parameter.
Values are independent, so only a store to that field or a call that pushes to the list could change it, and either
rules the loop out. Checks keep their place, index and site, so traps are unchanged. The lane measured 11.5% fewer
instructions on the ail-llm inner loop on x86-64. Test: `hoist_field_list`.

## Rejected alternatives

- **Alias analysis over all stores.** Not needed: value semantics already rule out aliasing between distinct values.
- **Ask authors to copy the field into a local.** Works today, but makes the plain form slow; AI understanding and
  readability come first.
