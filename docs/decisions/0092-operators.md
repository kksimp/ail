# 0092. Logical operators are `and`, `or`, `not`; compound assignment exists; methods always use parentheses

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 2 — Core syntax (expressions)
- **Spec rules:** [expr.op.1–3](../spec/05-expressions.md)
- **Evidence:** [syntax-measurements-r1.md](../design/syntax-measurements-r1.md): `and`/`or`/`not` cost the same as `&&`/`||`/`!`

## Deciding criterion

- **Logical words:** tokens **tie**. **AI understanding** decides: `!` already
  forms error types (`T ! E`, `!T`, ADR 0046), so using it for "not" would
  give one symbol two meanings.
- **Compound assignment** (`+= -= *= /= %=`): **token usage**.
- **Parentheses on every method call:** omitting `()` on zero-argument
  methods (`xs.len`) would save about one token per call, under 2% on
  representative code, so that's a **tie**. **AI understanding** decides: `()`
  distinguishes computation from stored fields.

## Decision

1. `and`, `or` (short-circuiting) and `not` are the logical operators.
   There are no `&&`, `||` or `!`-as-not.
2. `x op= e` means `x = x op e` for `+ - * / %`. `x` must be a mutable place.
3. A method call always has parentheses, even with no arguments: `xs.len()`.
