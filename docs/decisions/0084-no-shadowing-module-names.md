# 0084. Local names can't reuse module-level, imported or prelude names

- **Status:** Accepted; amended by [0469](0469-locals-may-shadow-functions.md) and [1121](1121-locals-may-shadow-functions-and-modules.md) (a local may hide a function or an imported module)
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 2 — Core syntax; 12 — Modules
- **Spec rules:** [expr.bind.5](../spec/05-expressions.md)
- **Resolves:** OQ-028
- **Extends:** [ADR 0044](0044-no-shadowing.md)

## Deciding criterion

Token usage **ties**. **AI understanding decides**: "one name, one meaning"
([ai-author-wishlist.md 1.2](../design/ai-author-wishlist.md)) extends naturally
from locals to every name visible in a file.

## Decision

Declaring a local name that is the same as a visible module-level item, an
imported name, or a prelude name is a compile-time error. The diagnostic
offers renames.

## Consequences

- The prelude (names visible without import) must stay **small**, so that
  common local names (`count`, `max`, `total`) stay available. Prefer
  methods (`xs.len()`, `a.max(b)`) over free functions in the prelude.
- Relaxing this later wouldn't break existing code.

## Rejected alternatives

- Allow locals to shadow module-level, imported and prelude names: rejected because "one name, one meaning" (ai-author-wishlist 1.2) extends from locals to every name visible in a file. (reconstructed 2026-10-21: the record gives the principle, not a named alternative.)
