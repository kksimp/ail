# 1095. `e else return ...` as a block's last statement gives `e`'s value

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, ADR 0076), as review package S25 of the AI-first review that the designer approved for 0.1.0 on 2026-11-12
- **Topic:** Errors ([08-errors.md](../spec/08-errors.md))
- **Spec rules:** `err.else.7`; amends `err.else.6`

## Context

`e else return E(...)`, `else break` and `else continue` are expressions
(`err.else.6`), but at the start of a statement the parser makes them the
statement form (`err.else.5`), which has no value. So the natural tail

```
fn need(m Map[str, str], key str) str ! CfgError
    m.get(key) else return Missing(key: key)
```

was rejected (`err.else.4` / `expr.block.2`) and had to be written as
`v := m.get(key) else return ...` then `v`. Agents write the tail form
first; the AI-first review's trap `trap2_err.ail` and the batch 7 test
`discard_rebind.ail` both hit it.

## Options considered

### Option A: a block's last `else`-that-leaves statement is the expression when the block's value is used (chosen)
- **Pros:** the tail form just works in a function body, an `if` branch and
  a `match` arm; nothing that compiled changes meaning, since the statement
  form there had no value and was an error; the checker and lowering both
  reuse the `OrElse` expression.
- **Cons:** the same text is a statement or an expression by position.
  That is already true of every expression statement at a block's tail.

### Option B: make the parser produce the expression form at every statement start
- **Cons:** `save() else return` on a result with no value is the
  statement form (`err.else.5`), and the parser can't see types; a value
  given away in the middle of a block would be silently dropped.

### Option C: keep it rejected with a better message and a fix that binds
- **Cons:** keeps a two-line idiom where one line reads correctly; costs
  AI understanding for no gain.

## Decision

`err.else.7`: when the last statement of a block whose value is used is
`e else return ...`, `e else break` or `e else continue`, it is the
`err.else.6` expression and gives `e`'s success value. Elsewhere the
statement form and `err.else.4` stay. The checker types the tail
`Handle` statement as `OrElse` (`compiler/check/body.ail`); lowering's
`final_expr` returns the same expression (`ast.tail_else`), one dispatch
line in `lower.ail`.

## Consequences

- Function tails of the form `m.get(k) else return E(...)` compile.
- The `else` block form with a handler (`e else err` + block) as a block's
  value is unchanged.
