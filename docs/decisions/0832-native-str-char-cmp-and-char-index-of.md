# 0832. Native `.cmp()` on `str` and `char`, `.index_of()` on `List[char]`

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** trait.std.2, trait.std.4, type.coll.11
- **Builds on:** [ADR 0602](0602-native-enum-layout.md) (flat enums are `int`s); lane 174's `.cmp()` on `int` and `float` (PR 224), which registers the prelude's `Ordering` natively

## Context

`examples/sheet.ail` calls `COLS.chars().index_of(c)` and compares values
with `.cmp()`. PR 224 gives `int` and `float` a native `.cmp()` and a flat
`Ordering`; `str`, `char` and `List[char].index_of` were still refused.

## Decision

- `a.cmp(b)` on two `str`s calls `rt.str_cmp` (bytes, so scalar values,
  trait.std.4) and adds 1 to turn -1, 0, 1 into the `Ordering` index. On two
  `char`s it reuses `rt.int_cmp`, since a `char` is its scalar value.
- `xs.index_of(c)` and `xs.contains(c)` work on `List[char]`: the element
  compares like an `int` (equality kind 0).
- The lowering is in `lower_cmp.ail`; `lower.ail` is unchanged (the dispatch lines are in `lower_str.ail`).

## Rejected alternatives

- **A separate `Ordering` registration**: PR 224 already does it.
- **Comparing `char`s with `lt`/`eq` branches**: three blocks per call for what
  `rt.int_cmp` does in one call.

## Consequences

`sheet.ail` loses three more errors; the rest are struct keys, trait `impl`s
and `ToStr`.
