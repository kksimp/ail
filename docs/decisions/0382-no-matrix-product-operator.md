# 0382. The matrix product stays a method: `a.matmul(b)`, no `@` operator

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** Designer
- **Topic:** 11 — Operators; standard library (`std.tensor`)
- **Spec rules:** none changed (`trait.op.2` and the operator set of
  [ADR 0077](0077-operator-traits.md) stay as they are)
- **Resolves:** OQ-113

## Context

[ADR 0344](0344-tensor-operators-and-scalar-broadcast.md) gave tensors
`+ - *` elementwise and `a.matmul(b)` for the matrix product. Python spells
the product `a @ b`.

## Options considered

### Option A — keep `a.matmul(b)` (chosen)
- **Pros:** no new operator; `matmul` is rare in agent code, so the token
  gain of an operator is under ADR 0076's 2% tie; nothing to teach.
- **Cons:** differs from Python and NumPy.

### Option B — `a @ b`
- **Pros:** familiar to Python users.
- **Cons:** amends ADR 0077's fixed operator set; `@` already starts
  attributes (`@inline`), so a line starting with a bare `@` reads two ways.

### Option C — a named operator `a matmul b`
- **Cons:** a form no other part of AIL has.

## Decision

Option A. `std.tensor` keeps `matmul` as a method.

## Consequences

- OQ-113 is closed. Rejected alternatives are recorded above.

## Rejected alternatives

- The `a @ b` operator (Python's spelling): rejected because it amends the fixed operator set of ADR 0077, and `@` already starts attributes (ADR 0083), so a line beginning with a bare `@` would read two ways.
- A named infix operator `a matmul b`: rejected because no other part of AIL has that form.
