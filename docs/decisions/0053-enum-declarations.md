# 0053. Enums (sum types) are declared `enum Name` with one indented variant per line

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 4 — Type system; 2 — Core syntax
- **Spec rules:** [type.enum.1–2](../spec/04-types.md)
- **Evidence:** cl100k tokens for a 3-variant enum: `enum Shape` 22, `type Shape enum` 23, ML-style `|` variants 25

## Options considered

- **`enum Shape` (chosen):** fewest tokens, consistent with `struct`, and one
  variant per line keeps edits to a single line.
- **`type Shape enum`:** +1 token.
- **ML-style `|`:** +14% tokens, and redundant with indentation.

## Decision

An enum is declared with `enum`, its name, and an indented block with one
variant per line. A variant may carry a parenthesised field list written
`name Type` (ADR 0037), or no data.

```
enum Shape
    Circle(r f64)
    Rect(w f64, h f64)
    Empty
```

## Consequences

- Enums are AIL's sum types. Exhaustive matching on them is proposed
  ([ai-author-wishlist.md 2.1](../design/ai-author-wishlist.md)) and will be
  asked with the `match` semantics.
- How variants are referred to (`Shape.Circle` vs bare `Circle`) and their
  representation (niche optimisation, tags) are still open.

## Rejected alternatives

- **`type X enum`:** +1 token.
- **`|` variants:** more tokens, and redundant.
