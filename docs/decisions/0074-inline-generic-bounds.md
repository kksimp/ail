# 0074. Generic constraints are written inline: `[T Bound]`, combined with `+`

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 9 — Generics
- **Spec rules:** [gen.bound.1–2](../spec/09-generics.md)
- **Evidence:** [traits-generics-options.md §3](../design/traits-generics-options.md#3-generic-constraints) (inline 44.0, `where` 46.0, implicit 42.8)

## Options considered

- **Inline (chosen):** consistent with `name Type` (ADR 0037). Generic code is
  checked on its own, and it's compact. Con: long bound lists widen signatures.
- **`where` clause:** shorter parameter lists, but +2 tokens and the
  constraint sits far from its parameter.
- **Implicit (template-style):** fewest tokens, but no checking until use,
  and poor errors.

## Decision

1. A generic parameter's constraints follow its name inside the brackets:
   `[T Shape]`. Several constraints are joined with `+`: `[K Hash + Eq]`.
2. A generic function body is type-checked using only what its declared
   constraints provide. Using an operation the constraints don't provide is
   an error at the definition, not at the use site.

```
fn largest[T Shape](items List[T]) T
    ...
```

## Consequences

- The formatter wraps long generic parameter lists. No `where` form exists.
- How generics are compiled (specialisation vs. shared code) is a separate
  question.

## Rejected alternatives

- **`where`:** a second place to look for constraints.
- **Implicit:** unacceptable diagnostics for AI repair loops.
