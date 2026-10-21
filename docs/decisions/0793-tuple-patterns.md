# 0793. Tuple patterns in `match`; a bare name binds

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076), under the designer's ADR 0791
- **Topic:** 05 — Expressions (`match`)
- **Spec rules:** `type.tuple.6`, `expr.match.2` ([04-types](../spec/04-types.md))

## Context

ADR 0791 allows `(0, y)` and `(x, _)` as `match` patterns. Elsewhere in AIL
a bare name in a pattern is a variant (`expr.match.4`), and a variant binds
its fields by position. A tuple element needs a way to bind.

## Decision

- `(p, q, ...)` matches a tuple of as many elements. An element pattern is
  `_`, a literal, a range, a variant (bare or `Enum.Name`), a nested tuple
  pattern, or a plain name.
- **A plain name inside a tuple pattern binds the element.** If that name is a
  variant of the element's enum, it is an error (`type.tuple.6`) whose help
  says to write `Enum.Name`, so the two readings never silently differ.
- **Coverage:** a `match` on a tuple must have a `_` arm or an arm whose
  elements all match anything (names, `_`, nested such tuples); otherwise
  `expr.match.2`. The checker doesn't compute coverage across arms of
  literals (`(true, _)` and `(false, _)` still need a last catch-all), which
  keeps the rule local and the diagnostic simple.
- Alternatives `(0, _) | (_, 0)` bind no names (`expr.match.8`).

## Token measurement

Tokbench mean of five tokenizers, a three-arm `match` on `(int, str)`: 49.0
with names bound in the patterns, 49.6 with `_` patterns and `t.0`, `t.1`
reads in the bodies. A tie (ADR 0076); the binding form is chosen for AI
understanding, since each arm names what it uses, as variant fields already
do. The coverage rule costs nothing to write: the usual last arm is
`(a, b) =>` or `_ =>`.

## Rejected alternatives

- **Names always variants, binding only with a keyword (`(0, bind y)`):** a
  new keyword for the most common element pattern.
- **Full coverage analysis over literal elements:** exact, but a combinatorial
  check whose diagnostics would list missing combinations; the `_` arm is
  what agents write anyway.
- **Bare name binds even when it's a variant:** silently matches everything
  when the agent meant the variant.
