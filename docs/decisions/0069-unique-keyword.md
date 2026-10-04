# 0069. Non-copyable types are declared with the `unique` keyword

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 7 — Ownership; 2 — Core syntax
- **Spec rules:** [own.uniq.1](../spec/07-ownership.md) (marker syntax)
- **Resolves:** OQ-035 (marker part)
- **Evidence:** `unique struct`, `resource struct` and `linear struct` cost the same. An `@unique` attribute costs +2.

## Options considered

- **`unique struct` / `unique enum` (chosen):** names the exact semantics
  (one owner, no copies), and also fits non-resource uses (capabilities,
  one-time tokens).
- **`resource struct`:** misleading for unique types that aren't OS resources.
- **`@unique` attribute:** +2 tokens, and understates a fundamental semantic change.

## Decision

A struct or enum declaration prefixed with `unique` declares a non-copyable type.

```
unique struct File
    fd int
```

## Rejected alternatives

- **`resource`:** too narrow.
- **Attribute:** costs more and is too subtle.
