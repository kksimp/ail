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

## OS errors

`fs.IoError(path, kind)` and `proc.ProcError(cmd, kind)` are error structs of
the standard library that no source declares. The runtime builds the value
where an OS call fails, next to the display text, so `else err` on `fs.*` and
`proc.run` binds a typed value and `err.kind`, `err.path` and `err.cmd` read.

## Methods and `if v := f(i)`

`x.get(k) else e` calls the method when `x` is a struct or enum with a `get`
method, not the map built-in. `if v := f(i)` on a call that can fail binds the
success value and takes the `else` side on failure, clearing the error.
