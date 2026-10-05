# 0091. Control flow: expression blocks, `if`/`elif`/`else`, `for in`, `while`, ranges `a..b`

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 2 — Core syntax (expressions)
- **Spec rules:** [expr.block.1–2, expr.if.1, expr.loop.1–3](../spec/05-expressions.md)
- **Evidence:** `tools/tokbench/corpus/r4/` `elif` 30.0 vs. `else if` 31.0; `0..n` 14.6 vs. `range(0, n)` 16.2

## Deciding criterion

**Token usage:** `elif` saves one token per chain link, `a..b` beats a range
call by 10%, and implicit block values avoid a `return` per function (already
used since ADR 0037's example). An infinite `loop` keyword was **not** added:
`while true` is rare, so the token cost ties, and **AI understanding** prefers
fewer keywords.

## Decision

1. A block is an expression. Its value is its last expression. A function
   returns the value of its body, and `return` exits early.
2. `if c` / `elif c` / `else`, each followed by an indented block. `if` is an
   expression when every branch produces a value and `else` is present.
   (Layout of a multi-line `if` used as a value: [ADR 0111](0111-if-expression-layout.md).)
3. `for x in iterable` and `while condition` loops, with `break` and `continue`.
4. `a..b` is the half-open range from `a` up to but excluding `b`. `a..=b`
   includes `b`.

## Consequences

- Loop labels, `for` with an index, and pattern destructuring in `for` are to
  be designed with iteration (OQ-037).
