# 0042. Functions are introduced by the keyword `fn`

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 2 — Core syntax
- **Spec rules:** [gram.fn.1](../spec/03-grammar.md)
- **Evidence:** [syntax-measurements-r1.md](../design/syntax-measurements-r1.md) (`fn`, `func`, `function` and `def` each cost 1 token)

## Context

Every candidate keyword costs exactly one token, so the choice is about
readability only.

## Options considered

- **`fn` (chosen):** visually short, so signatures stay compact. Used in the
  brief, and recognisable. Con: an abbreviation.
- **`func`:** clear, but visually longer at no token gain.
- **`function`:** the most self-explanatory, but pushes signatures wide.

## Decision

Function declarations begin with `fn`.

## Consequences

- This sets a precedent for other keywords: since token cost is equal, choose
  the clearest *short* word. Each keyword is still decided individually.

## Rejected alternatives

- **`func`, `function`:** longer on screen with no benefit.
