# 0098. Items are exported with `pub`; everything else is private to its module

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 12 — Modules
- **Spec rules:** [mod.vis.1–2](../spec/12-modules.md)
- **Background:** [design/concurrency-modules-options.md](../design/concurrency-modules-options.md) (representative code: `pub` 195.8, capitalisation 194.4, a 0.7% tie)

## Options considered

- **`pub` keyword (chosen):** changing visibility is a one-word edit with no
  renames, naming conventions stay free, and visibility is explicit.
- **Capitalisation (Go):** no tokens, but visibility changes become renames,
  and casing no longer tells types from values.

## Decision

1. Top-level declarations (types, functions, methods, traits, constants) and
   struct fields are private to their module unless marked `pub`.
2. Enum variants and trait methods take their enum's or trait's visibility.

## Consequences

- With ADR 0046, a function's `pub` marker also decides whether its error type
  must be spelled out.
- Package-level visibility (visible to sibling modules but not to other
  packages) may be added later if needed.

## Rejected alternatives

- Capitalisation for visibility (Go-style): no tokens, but changing visibility becomes a rename, and casing no longer tells types from values. On representative code `pub` costs 195.8 tokens against 194.4 for capitalisation, a 0.7% tie, so the choice rests on the other criteria.
