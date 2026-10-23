# 0864. A recursive type's default text is a generated function

- **Status:** Accepted
- **Date:** 2026-10-23
- **Decided by:** AI (delegated, ADR 0076; number allocated by the integrator)
- **Topic:** 23 — native backend
- **Spec rules:** trait.auto.2

## Context

`print(v)` of a struct or enum with no `to_str` shows `Name(field: value, ...)`
(trait.auto.2). The native lowering inlined that text at each use. A type that
reaches itself, such as `Json` (`Arr(items: List[Json])`), expanded without
end and the compiler stopped with "call depth exceeded". It is found by
`print(j)` of any parsed JSON value.

## Decision

1. The first use of a type's default text inlines as before. A use of the
   same type met inside it calls `text.T`, a function generated once per type
   (`compiler/mir/lower_recur.ail`), whose body expands the type the same way.
2. The set of types being expanded is one field of the lowering state; the
   generator is one branch of the loop that already generates `eq.T`.
3. Related fixes in the same change: `f[T](...)?` on a generic function that
   returns `U ! E` counts as a failing call, and a user file may call the
   prelude's generic `load_json[T]` and `print_json[T]` with type arguments.

## Rejected alternatives

- **Always call `text.T`.** Changes every existing print's code for no gain.
- **Report a recursive type as not native.** Leaves `print(json_value)`, a
  common debugging line, out of native programs.
