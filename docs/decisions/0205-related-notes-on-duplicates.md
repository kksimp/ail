# 0205. Duplicate declarations point at the first one

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** AI semantic API
- **Spec rules:** `fix.note.1` (non-normative)

## Context

`notes` ([ADR 0145](0145-diagnostics-schema-v1.md)) carry related positions, but only a duplicate type had one. An agent told `helper is declared more than once` has to search for the other.

## Decision

These diagnostics carry a note at the earlier declaration: `mod.unit.1` for a duplicate function, method or constant (types already did), `expr.bind.4` for a name already declared in the function, and `expr.bind.5` for a local that takes the name of a function, type or constant (not for the prelude).

## Rationale

AI understanding (#2): the position removes a search; the cost is a few tokens only in the diagnostics that need them.

## Rejected alternatives

- **Put the position in the message.** Not machine-readable; `notes` already is.
- **Also note uses before declaration.** AIL has no use-before-declaration error for functions (declaration order is free) and locals are unknown names (`expr.name.1`), whose fix is a suggestion, not a position.
