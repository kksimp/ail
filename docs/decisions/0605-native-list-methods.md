# 0605. Native list methods: editing, searching and sorting

- **Status:** Accepted (`first`, `last`, `index_of`, `min`, `max`, `sum` and `swap` followed in [ADR 0802](0802-native-list-queries.md))
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** type.coll.4, type.coll.9, type.coll.11, type.coll.14, type.num.11, own.val.1
- **Builds on:** [ADR 0384](0384-mir-shape-and-native-subset.md) (lists), [ADR 0532](0532-native-comprehensions-and-slices.md) (the MIR `trap` operation)

## Context

The compiler's own source calls `extend` (38 uses), `contains`, `remove_at`,
`pop`, `sort`, `sorted` and `reverse` about 60 times; building
`compiler/main.ail` natively reported each of them.

## Decision

Each method is a runtime call in `compiler/rt/native/list.ail`, lowered in
`compiler/mir/lower_list.ail` on lists of `int`, `float`, `bool` and `str`
(`extend`, `remove_at`, `insert`, `clear`, `reverse` also on lists of lists
and structs, which are words):

- `extend(ys)` appends `ys`, copied first when it is a place (own.val.1).
- `remove_at(i)` reads `xs[i]` with the usual bounds check (`type.coll.4`,
  `index I is out of bounds ...`), then shifts the tail down.
- `insert(i, x)` checks `0 <= i <= len` with a MIR check and the `trap`
  operation, message kind 8: `insert index I is out of bounds for a list of
  length N`.
- `pop()` as a statement drops the last element when there is one;
  `pop() else X` (binding or operand) yields it or the fallback.
- `contains(x)` compares by word for `int` and `bool`, by bytes for `str`,
  by IEEE equality for `float`.
- `sort()` and `sorted()` are a stable merge sort of 8-byte words with a
  comparison per kind: integers, strings by bytes, floats by `cmp`
  (`nan` last, type.num.11). `sorted()` sorts a copy.

Reported under native.subset.1: `by:` key functions, `sort` of other
element types, `contains` on structs, `pop()` used as a bare value, `first`,
`last`, `index_of`, `take`, `min`, `max` and `sum`.

## Consequences

- Native problems in the compiler's list-method uses fall from about 40 to
  those on values whose type is still unknown (structs, generics).
- Sorting allocates a scratch array of the list's length each time.

## Rejected alternatives

- **Insertion sort.** Fewer lines, quadratic: the compiler sorts lists of
  thousands.
- **Sort in place with a key array for `str`.** Needs a second pass; the
  word comparison reads the string objects directly.
- **Generic monomorphised `contains` through `==`.** Waits on lane 120's
  generics and operator lowering for structs.
