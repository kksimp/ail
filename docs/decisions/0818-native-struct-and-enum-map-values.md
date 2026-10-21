# 0818. Native maps with struct and enum values

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** type.map.*, type.coll.*, own.val.1
- **Builds on:** [ADR 0568](0568-native-map-and-set-layout.md) (tables), [ADR 0602](0602-native-enum-layout.md) (enum values), [ADR 0392](0392-native-structs-and-methods.md) (structs)

## Context

`Map[str, Def]` with an enum `Def`, and a `var_ty = {}` field of that type,
were the next blockers for the compiler's own source and several examples.
Struct values already built, but a copy of such a table shared its structs, so
`b := a; b["k"].x = 1` changed `a` too (own.val.1).

## Decision

- An enum is a valid map value type. A value is an `int` tag or a pointer to
  an immutable shared object (ADR 0602), so a table stores it as one word and
  a copy of the table shares it, as for `str`.
- A table whose values hold a struct (a struct, a list of structs, or a table
  of those) is copied by a generated function `copy.Map[K, V]`, like the
  generated copies of lists of structs: the runtime copies the table, then
  each live entry's value is replaced by its own copy. The table's value mode
  is 0 for these, so the runtime does not copy them a second time.
- `rt.map_used` and `rt.map_live` give the generated loop the number of slots
  in use and a live entry's address.

Printing a table of enums or structs is still reported under native.subset.1,
as printing an enum or struct is.

## Addendum: tables as list elements

A list's elements may be maps and sets (`List[Map[str, str]]`, the result of
`csv.records`), as parameters, results, fields and literals. A list of tables
is copied by the generated `copy.List[...]`, which copies each table
(`copy.Map[...]` or `rt.map_copy`), so a copy of the list is independent.
This is a one-line change in `lower.ail` (list element types and list
literals) plus `holds_struct` counting a table element.

## Consequences

- `Map[str, Def]`, `Map[int, Point]` and `Map[str, List[Point]]` build and
  copy independently. A struct field `vars Map[str, Def]` accepts `{}`.
- Copying a table of structs costs one copy per live entry, as for a list.

## Rejected alternatives

- **Sharing the structs and copying on write.** The language has no
  copy-on-write; every other struct copy is eager.
- **Copying structs in the runtime by byte size.** A struct may hold lists or
  other structs; the generated function already knows how to copy those.
