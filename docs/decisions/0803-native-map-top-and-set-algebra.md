# 0803. Native `Map.top`, set algebra and map and set comprehensions

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** std.count.2, type.coll.16, expr.comp.1
- **Builds on:** [ADR 0568](0568-native-map-and-set-layout.md) (tables), [ADR 0802](0802-native-list-queries.md) (list queries)

## Context

`examples/map_top.ail` and `examples/snapdiff.ail` stopped at `m.top(n)`,
`m.keys().union(...)` and map comprehensions in the Haiku native-blockers report (day 3).

## Decision

Runtime calls in `compiler/rt/native/map.ail`, lowered in
`compiler/mir/lower_map.ail`:

- `m.top(n)` on a `Map[K, int]` with `int` or `str` keys: a stable merge sort
  of the live entry numbers by count, largest first, so equal counts keep
  their insertion order; the first `n` keys (none for `n <= 0`) as a list.
  `m` is not changed.
- `union`, `intersect` and `difference` on a `Set[int]` or `Set[str]` (the
  argument a set) and on a `List[int]` or `List[str]` (the argument a list or
  a set). A list operand becomes a set first; the result is a new set in
  `a`'s order, then, for `union`, the new elements of `b` in order.

- Set and map comprehensions (`{x for ...}`, `{k: v for ...}`) lower as the
  loops they mean, as list comprehensions do (ADR 0532): `add` or `m[k] = v`
  on a new table. They need the expected type (a function result, argument or
  field); a local binding with no type reports "a set comprehension whose type
  isn't known" rather than guessing from the first element.

Reported under native.subset.1: `top` on other key or count types, set algebra
on `bool` or other element types.

## Consequences

- `map_top` builds natively and matches its `.out`.
- `top` allocates an index array of the live entries each call.

## Rejected alternatives

- **Repeated selection of the largest count.** Quadratic in the number of keys
  for large `n`; the sort is `n log n` and stable.
- **Set algebra by copying and removing.** `intersect` and `difference` would
  need a remove-heavy pass that leaves dead entries; building the result
  from `a` is simpler and compact.
