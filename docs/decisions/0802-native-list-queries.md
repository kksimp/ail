# 0802. Native list queries: sum, min, max, first, last, index_of, swap, Set.sorted

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** type.coll.7, type.coll.9, type.coll.11, type.coll.12, type.coll.14, type.num.11, expr.arith.1
- **Builds on:** [ADR 0605](0605-native-list-methods.md) (list methods), [ADR 0574](0574-native-optional-layout.md) (optionals), [ADR 0568](0568-native-map-and-set-layout.md) (tables)

## Context

[ADR 0605](0605-native-list-methods.md) left `sum`, `min`, `max`, `first`,
`last`, `index_of` and `swap` to the native subset's reports. The compiler's
own source and the perfbench programs use them.

## Decision

Each method is a runtime call in `compiler/rt/native/list.ail`, lowered in
`compiler/mir/lower_list.ail`:

- `swap(i, j)` reads both indexes with the usual bounds check
  (`type.coll.4`), then exchanges the elements.
- `sum()` on `List[int]` adds left to right and traps on overflow with the
  interpreter's `` `sum` overflows `int` `` under `expr.arith.1` (the trap
  texts are built at the call site, as for map reads). On `List[float]` it
  follows the fixed tree of `type.coll.12`: four partial sums up to 32
  elements, else a split at `32 * ceil(len / 64)`.
- `min()` and `max()` return the first least or greatest element as a `T?`,
  for `int`, `float` (`nan` last, `type.num.11`) and `str`. The runtime
  returns an index and the lowering builds the optional.
- `first()` and `last()` return the element as a `T?`.
- `index_of(x)` returns the first equal index as an `int?`, comparing as
  `contains` does.
- `Set[int].sorted()` and `Set[str].sorted()` sort the keys, in the set's
  insertion order for equal keys (they are distinct), as a list.

Reported under native.subset.1: `min` and `max` of `List[bool]`, `sum` of
other element types, `sorted` on a `Set[bool]`, `by:` key functions.

## Consequences

- `xs.min() else d` and friends run natively; printing an optional is still
  outside the subset (use `else`).
- `sum` of floats costs one recursive pass with four accumulators.

## Rejected alternatives

- **A single left-to-right float sum.** Differs from the interpreter's tree
  (`type.coll.12`), so output would not match bit for bit.
- **Lowering `min`/`max` as MIR loops.** More code per call site for no
  speed gain over one runtime call.
