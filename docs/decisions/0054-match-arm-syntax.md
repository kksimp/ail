# 0054. Match arms are written `pattern => result`

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 2 — Core syntax (expressions)
- **Spec rules:** [expr.match.1](../spec/05-expressions.md)
- **Evidence:** cl100k tokens for a 3-arm match: `:` 34, `=>` 36, `->` 36, `case … =>` 39

## Options considered

- **`=>` (chosen):** a strong, unambiguous separator that leaves `:` free for
  other uses (e.g. named arguments). Widely recognised. Con: ~6% more tokens than `:`.
- **`:`:** the fewest tokens, but uses up `:` for good, and a multi-line arm
  looks like a block header.
- **`->`:** looks like the return-type arrow AIL dropped (ADR 0037).
- **`case … =>`:** the most tokens.

## Decision

`match e` is followed by an indented block of arms, one per line, each written
`pattern => result`. A result that needs several statements is written as an
indented block after `=>`.

```
area := match s
    Circle(r) => pi * r * r
    Rect(w, h) => w * h
    Empty => 0.0
```

## Consequences

- Pattern syntax (bindings, wildcards, literals, guards, or-patterns),
  exhaustiveness, and `match` on error sets (OQ-029) are still to be designed.

## Rejected alternatives

- **`:`:** saves ~6% tokens, but costs `:` for every other future use.
- **`->`:** confusable with a return arrow.
- **`case`:** more tokens.
