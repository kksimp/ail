# 0108. Calls pass one argument per parameter with exactly matching types; syntax errors cite one rule

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 2 — Core syntax (expressions); 20 — AI semantic API (diagnostics)
- **Spec rules:** [expr.call.1–2](../spec/05-expressions.md), [gram.syntax.1](../spec/03-grammar.md)

## Context

The bootstrap needs spec rule IDs to cite for wrong calls and for malformed
syntax (ADR 0002). Neither was specified yet.

## Deciding criterion

Token usage **ties**. **AI understanding** decides:
- No implicit conversions between argument and parameter types
  ([ai-author-wishlist.md 1.2](../design/ai-author-wishlist.md)). A call
  means exactly what it says.
- One stable rule for "this doesn't match the grammar", until the
  machine-checkable grammar (ADR 0002) assigns finer-grained rules.

## Decision

1. A call supplies exactly one argument per parameter, in order (named
   arguments, once designed, extend this rule).
2. Each argument's type must equal its parameter's type (or satisfy its
   constraint, for generic parameters). There are no implicit conversions,
   apart from the optional wrapping defined by ADR 0093.
3. Source that doesn't match the grammar is rejected with a diagnostic naming
   the expected construct (`gram.syntax.1`), unless a more specific rule applies.

## Consequences

- Implementation limits are reported with the code `bootstrap.unsupported`.
  That's not a spec rule; it names the spec rule of the unimplemented construct.
