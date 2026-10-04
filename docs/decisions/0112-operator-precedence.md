# 0112. Operator precedence; comparisons don't chain

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 5 — Expressions
- **Spec rules:** [expr.prec.1–2](../spec/05-expressions.md)

## Deciding criterion

Token usage **ties**. **AI understanding** decides: the familiar
arithmetic-over-comparison-over-logic order, `not` below comparisons (so
`not a == b` means `not (a == b)`, as in Python), and no chained comparisons
(`a < b < c` is an error rather than a silent surprise).

## Decision

From tightest to loosest binding:

| Level | Operators | Associativity |
|---|---|---|
| 1 | calls `f()`, fields `a.b`, methods `a.m()`, indexing `a[i]`, `?` | left |
| 2 | unary `-` | prefix |
| 3 | `*` `/` `%` | left |
| 4 | `+` `-` | left |
| 5 | `==` `!=` `<` `<=` `>` `>=` | none (no chaining) |
| 6 | `not` | prefix |
| 7 | `and` | left |
| 8 | `or` | left |
| 9 | `..` `..=` | none |

Writing two comparisons in a row without parentheses is a compile-time error.
