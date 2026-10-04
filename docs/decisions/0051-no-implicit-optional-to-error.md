# 0051. `?` on an optional is not allowed in an error-returning function; absence must be converted explicitly

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 8 — Error handling
- **Spec rules:** [err.prop.2](../spec/08-errors.md)
- **Partly resolves:** OQ-029

## Context

In a function returning `T ! E`, applying `?` to a `T?` value would need some
error to represent "absent".

## Options considered

### Option A — Not allowed; convert explicitly (chosen)
- **Pros:** Every error says exactly what went wrong, and no vague "absent"
  errors reach users or AI. Costs one line with `else` (ADR 0049).
- **Cons:** A few more tokens than a bare `?`.

### Option B — Allowed, producing a built-in "Absent" error
- **Pros:** The fewest tokens.
- **Cons:** Vague errors, and inferred error sets fill up with an
  uninformative type.

## Decision

Applying postfix `?` to an expression of type `T?` is a compile-time error
unless the enclosing function returns an optional type. In an error-returning
function, absence is converted with `else`, e.g.
`row := db.find(id) else return DbError.NotFound(id)`.

## Consequences

- The diagnostic offers the machine-applicable fix: rewrite as
  `else return <ErrorValue>`.

## Rejected alternatives

- **Built-in Absent error:** uninformative errors.
