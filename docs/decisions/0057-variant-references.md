# 0057. Enum variants are written bare where the expected type is known, qualified otherwise

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 4 — Type system; 2 — Core syntax
- **Spec rules:** [type.enum.3–4](../spec/04-types.md)
- **Evidence:** cl100k tokens for two match arms: bare 22, `Shape.Circle` 24, `.Circle` 24

## Options considered

- **Bare when the type is known (chosen):** the fewest tokens where variants
  are most common, and never ambiguous. Con: two written forms depending on
  context.
- **Always qualified:** one form, but ~9% more tokens in matches.
- **`.Variant` when inferred:** same tokens as qualified, and the dot is easy to miss.

## Decision

1. Where the expected type of an expression or pattern is a known enum type
   (match patterns, arguments to typed parameters, typed bindings, return
   values), its variants may be written bare: `Circle(r)`.
2. Elsewhere, variants are qualified: `Shape.Circle(r)`.
3. The formatter applies this rule canonically: bare where allowed,
   qualified otherwise.

## Consequences

- Bare variant names take part in name resolution only in contexts with a
  known expected type. They don't pollute the ordinary scope, so there is no
  clash with ADR 0044 (no shadowing).
- Diagnostics show the qualified form when the context is ambiguous.

## Rejected alternatives

- **Always qualified:** more tokens for no extra clarity in typed contexts.
- **`.Variant`:** no saving, and a subtle marker.
