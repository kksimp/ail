# 0183. A bare variant pattern matches any fields

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Pattern matching; token efficiency
- **Spec rules:** [expr.match.4](../spec/05-expressions.md#match), expr.match.10

## Context

[ADR 0123](0123-variants-with-data.md) requires one binding or `_` per field
in a variant pattern, so a match that only asks *which* variant it has spells
out every field: `Null | Bool(_) | Int(_) | Float(_) | Str(_) => compact(v)`.
A catch-all `_` would be shorter, but [ADR 0056](0056-exhaustive-matching.md)
warns on it, rightly, because it hides variants added later. The corpus has 28
such patterns, most in `json` and `sheet`, the two programs above Python.

## Options considered

### Option A — a bare name matches any fields (chosen)
`Bool | Int => ...`, and `Shape.Rect => ...`.
- **Pros:** −0.6% tokens on the corpus (−84; `json` −31, `sheet` −41);
  adding a field to a variant no longer breaks every arm that ignores it, an
  edit agents make often; exhaustiveness checking is unchanged.
- **Cons:** `Rect` in a pattern no longer shows that `Rect` carries data.

### Option B — a rest pattern, `Rect(..)`
- **Pros:** shows that there are fields.
- **Cons:** costs about two tokens per use for no extra safety; a new token
  shape (`..` inside parentheses) to learn.

### Option C — keep one `_` per field
- **Pros:** the field count is visible at every use.
- **Cons:** the tokens above, and every such arm must change when a field is added.

## Decision

Adopt Option A. A bare variant with data binds no names; written with
parentheses, it still needs one binding or `_` per field (expr.match.6).

## Rationale

A tie on **tokens** under ADR 0076 (−0.6%), decided by **AI understanding**:
the pattern says exactly what the arm depends on (the variant), and the
"field count changed" edit no longer ripples through unrelated arms. The
variant's declaration, one `ail api` call away, still shows its fields.

## Consequences

- `variant_errors` keeps its negative case with a wrong count in parentheses.
- Examples now use bare names where every field was `_`.

## Rejected alternatives

- **`Rect(..)`:** more tokens and no added safety.
- **Keep `_` per field:** costs tokens and edits for no information the reader needs.
