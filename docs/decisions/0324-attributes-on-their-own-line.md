# 0324. Attributes may sit on their own lines above a function, in name order

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 2 — Core syntax
- **Spec rules:** `gram.attr.2`

## Context

[ADR 0083](0083-attribute-syntax.md) and gram.attr.2 say the formatter moves attributes to their own preceding line when the line is too long, but the parser accepted only same-line attributes (`@test` then a newline was `gram.attr.2`), so the formatter could not do it, and attribute order was whatever the author wrote.

## Decision

1. The parser accepts a newline between an attribute and the next attribute or the function (gram.attr.2).
2. fmt.attr: attributes are sorted by name. They stay on the function's line when the whole header fits in 100 columns (ADR 0083), otherwise one per line directly above it. Source layout and order don't matter; a blank line between an attribute and its function is removed. A comment between them leaves the layout as written.
3. `fmt.safe` treats attribute order and the newline after an attribute as not meaningful.

## Rationale

**Tokens decide.** Same-line placement stays the default (one token cheaper, ADR 0083), and own-line is used only past the limit, so the repository formats to itself (0 tokens changed). Name order is arbitrary but fixed, which is what makes reformatting stable.

## Rejected alternatives

- *Always one per line above the item.* Rejected: about one extra token per attribute on every `@test`, with 73 of them in the repository.
- *Keep the author's order.* Rejected: two orders of the same attributes would diff for no reason.
- *Order by importance (`@test` first).* Rejected: needs a list of every attribute; alphabetical needs none.
