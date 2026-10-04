# 0107. Every name must resolve; unknown names get "did you mean" suggestions

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 5 — Expressions; 20 — AI semantic API (diagnostics)
- **Spec rules:** [expr.name.1](../spec/05-expressions.md)

## Deciding criterion

Token usage **ties**. **AI understanding** decides: a typo'd name should
produce one precise, fixable diagnostic. The nearest declared names are
offered as machine-applicable fixes ([ai-author-wishlist.md 2.4](../design/ai-author-wishlist.md)).

## Decision

Every name used in an expression must refer to a declaration visible at that
point. Otherwise it's a compile-time error, and the diagnostic suggests the
closest visible names.
