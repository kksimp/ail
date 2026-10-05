# 0144. `else` is also an expression: `e else d` and `e else return ...`

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for it
- **Topic:** 8 — Error handling; 2 — Core syntax
- **Spec rules:** [err.else.6](../spec/08-errors.md), [expr.prec.1](../spec/05-expressions.md)
- **Extends:** [ADR 0049](0049-else-for-fallback-and-early-exit.md) (`else` on a binding)
- **Resolves:** [OQ-040](../open-questions.md)
- **Evidence:** [program-measurements-r2.md](../design/program-measurements-r2.md); `tools/tokbench/corpus/programs/`

## Context

`else` worked only on a binding (`x := e else d`) or a statement, so a
fallback inside a larger expression needed a throwaway name:
`lo := nums.min() else 0` then `lo`.

## Options considered

| Option | Example | Tokens (three programs) |
|---|---|---|
| Binding only (ADR 0049) | `lo := nums.min() else 0` / `lo` | 13,669.0 |
| **`else` as the loosest operator (chosen)** | `Min => nums.min() else 0` | **13,596.6** (−0.5%) |
| A method, `nums.min().or(0)` | | a second spelling of the same fallback |

Tokens alone are a tie (under 2%), though the affected lines shrink a lot
(the sheet's aggregate arms −38%). **AI understanding** decides: it's the
form Kotlin (`?:`) and Swift (`??`) programmers expect, and the drafting
agents wrote it unprompted.

## Decision

1. `e else d` is an expression: the value of `e` if it is present (`T?`) or
   succeeded (`T ! E`), otherwise `d`, which has `e`'s success type. `d` is
   evaluated only when needed.
2. `e else return x`, `else break` and `else continue` are expressions too;
   the fallback leaves, so the expression has `e`'s success type.
3. `else` binds more loosely than every other operator and associates to the
   right: `a + b else c` is `(a + b) else c`; write `(m.get(k) else 0) + 1`.
4. An `else` followed by an indented block (`else` or `else err` then a
   block) is still the binding or statement form of ADR 0049 and ADR 0139.
   At the start of a statement, `e else return ...` is the statement form;
   to use it as a function's value, write `(e else return ...)`.

```ail
match a
    Min => nums.min() else 0
count := (m.get(k) else 0) + 1
x := Float(text.to_float() else return fail("invalid number"))
```

## Consequences

- Single-use bindings for fallbacks disappear; the three examples lost 16 of
  them.
- A long `else return ...` inside an index or argument can hurt readability;
  keep a binding there.

## Rejected alternatives

- **Binding only:** a throwaway name for every fallback inside an expression.
- **A method (`.or(d)`):** two spellings of one fallback, and it can't leave
  the function.
- **Tighter precedence:** `a + b else c` meaning `a + (b else c)` surprises
  readers of Kotlin and Swift.
