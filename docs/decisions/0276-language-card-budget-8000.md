# 0276. Raise the language card budget to 8,000 tokens

- **Status:** Accepted (amends [ADR 0174](0174-language-card.md)); amended in part by [ADR 1110](1110-language-card-budget-16000.md) (16,000) and [ADR 0817](0817-token-budgets-for-agent-pages.md) (the budget is checked on the five-tokenizer mean)
- **Date:** 2026-10-20
- **Decided by:** the designer
- **Topic:** AI-first tooling
- **Spec rules:** none

## Context

[ADR 0174](0174-language-card.md) set a 4,000-token budget for
`docs/language-card.md` on every tokbench tokenizer. During the 2026-10-20
buildout the language gained tasks, contracts, packages, unique types and
`ref`, comprehensions, `par for` and a larger standard library, and the card
had to drop examples (a second trait implementation, `Version`/`Ord`,
generic methods, `any Trait` collections, `int??`, `unreachable`) to stay
under budget. The designer judged that limit too tight.

## Options considered

### Option A — 8,000 tokens on every tokenizer (chosen)
- **Pros:** room for one example per construct, including the constructs
  added tonight, while still one small read for an agent.
- **Cons:** twice the context cost per load.

### Option B — keep 4,000 and split the card in two
- **Pros:** the core stays cheap.
- **Cons:** an agent must know which half it needs; two headers to version.

### Option C — no budget
- **Pros:** nothing to trim.
- **Cons:** the card would drift back into the quick reference.

## Decision

The language card's budget is 8,000 tokens on every tokbench tokenizer.
The card keeps one example per construct and stays non-normative; the
header records its measured count as before. Rule and test text that
mentions the budget cites this ADR.

## Rationale

The designer set the number on 2026-10-20 ("increase the lang card to 8k
tokens so we aren't limited too much"); the alternatives above were not
preferred.

## Rejected alternatives

- Splitting the card (Option B) and dropping the budget (Option C), as above.
