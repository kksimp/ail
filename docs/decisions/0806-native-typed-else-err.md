# 0806. Typed error binding in native `else err`

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md), err.else.3
- **Builds on:** [ADR 0804](0804-native-error-values-are-structs-and-enums.md), [ADR 0805](0805-native-error-message-method.md)

## Context

`else err` binds the error (err.else.3): with one error type its fields
and `match` must work; with a union or an inferred set the spec gives field
access only after OQ-029.

## Decision

The lowering records the error type each function that can fail names
(`fn_errs`): the one type of `T ! E` or `! E`. Where there is exactly one,
`else err` takes the stored value (`rt.err_take_value`) and binds it with that
type, so `err.path` and `match err` work. For a union (`A | B`) or an inferred
set, `err` is the display text, a `str`: it prints and interpolates as the
interpreter does, and field access or `match` on it is reported by
native.subset.1.

## Rejected alternatives

- **Carry the type with the value.** Costs a word per error and an any-box
  for the one case OQ-029 has not specified.

## Lists of optionals

A list, field or parameter may hold optionals of numbers, strings and enums
(`List[str?]`, `List[Expr?]`): an element is one word, 0 for none. A literal
with a `none` among its items is a list of optionals.
