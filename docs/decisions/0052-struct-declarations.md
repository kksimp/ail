# 0052. Struct types are declared `struct Name` with indented `field Type` lines

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 4 — Type system; 2 — Core syntax
- **Spec rules:** [type.struct.1–2](../spec/04-types.md)
- **Evidence:** cl100k tokens for a 3-field struct: `struct User` 16, `type User struct` 17, `type User =` 16

## Options considered

- **`struct User` (chosen):** fewest tokens, the kind of type is immediately
  visible, and easy to scan and grep. Con: one keyword per kind of type.
- **`type User struct`:** one keyword for all types, but +1 token and the
  kind comes after the name.
- **`type User =`:** the kind is implicit, so readers must infer it from the body.

## Decision

A struct is declared with `struct`, its name, and an indented block of field
declarations. Each field is written `name Type`, matching parameter syntax
(ADR 0037).

```
struct User
    id i64
    name str
    email str?
```

## Consequences

- Struct construction (literal) syntax, default field values, field
  visibility, and methods are still to be designed.
- Layout remains compiler-chosen unless a fixed layout is requested (ADR 0013).

## Rejected alternatives

- **`type X struct`:** +1 token, and the kind comes late.
- **`type X =`:** the kind is implicit.
