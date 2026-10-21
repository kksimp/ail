# 0748. `List[T].from_json(j)`: associated calls on builtin generic types

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20); the integrator approved the change on PR 294
- **Topic:** 17 — Standard library
- **Spec rules:** json.derive.2 ([std-json](../spec/std-json.md)), trait.assoc.4

## Context

`FromJson` is implemented for `List[T FromJson]`, and `T.from_json(j)` works
in generic code, but a program could not write `List[U].from_json(j)`: the
checker said "`List[...]` is a function, not a value". Reading a JSON array
of records needed a loop over `as_arr()` with one `U.from_json` per element.
In lane 192's twin t11 (JSON transform) that loop and its error plumbing cost
about 40 tokens.

## Decision

`List[T].name(args)` resolves like `Type.name(args)` (trait.assoc.4) when a
trait with an associated function `name` is implemented for `List[T]`: the
impl's parameters are bound from the written element type and the call is
checked and run as any associated call. Today the only such function is
`from_json`.

Like all `FromJson` uses, it needs the program to name `FromJson` (the
prelude's derive support is added only then), so a program with no
`impl FromJson` still gets the old message.

## Token measurement

Twin t11 (read users, add a year to adults, print JSON; Python 57.6 tokens): by hand with `as_arr()` and accessors 107.0; with `List[U].from_json` (`agent-r4/t11b_json_typed.ail`) **89.6 (-16%)**, still +56% against Python, whose `json.load` needs no type declaration.

## Rejected alternatives

- **A `Json.to_list(...)` method.** Needs a type parameter on a method of a
  non-generic type; the associated call already expresses it.
- **Calls on every builtin type (`int.from_json`).** No use found yet;
  `int` and `str` are read by the derive through `need_*`. Add when needed.
