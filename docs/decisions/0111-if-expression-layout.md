# 0111. In a multi-line `if` expression, `elif` and `else` align with the line that starts the `if`

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 2 — Core syntax
- **Spec rules:** [expr.if.3](../spec/05-expressions.md)
- **Corrects:** the example in [ADR 0091](0091-control-flow.md) and spec `expr.if.1`

## Context

The earlier example indented the first branch by two levels and `elif`/`else`
by one:

```
kind := if n < 0
        "negative"
    elif n == 0
```

That violates `lex.block.3` (a block is indented by exactly one level). The
layout rule for `if` used as a value was never stated.

## Deciding criterion

Token usage **ties**. **AI understanding** decides: one layout rule for every
`if`, whether it's a statement or a value. Blocks are one level deeper than
the line that opens them, and `elif`/`else` sit at that line's level.

## Decision

```
kind := if n < 0
    "negative"
elif n == 0
    "zero"
else
    "positive"
```

The same holds for `match` arms (one level deeper than the line containing
`match`) and for any other construct that opens a block from inside a statement.

## Rejected alternatives

- The earlier layout, with the first branch indented two levels and `elif`/`else` one level: rejected because it violates `lex.block.3`, which says a block is indented by exactly one level.
