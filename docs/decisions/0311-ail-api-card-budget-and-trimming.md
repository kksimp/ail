# 0311. Card budget, token estimate and trimming order

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** [api.card.3](../design/semantic-api.md#program-card)
- **Extends:** [ADR 0150](0150-ail-api-outline.md)

## Context

A card must fit a reader's budget. The language card has 8,000 tokens
([ADR 0276](0276-language-card-budget-8000.md)); a program card is read in
addition to it, so the default is half of that. Deciding criteria:
**tokens**, then **AI understanding**.

## Options considered

### Budget: 4,000 tokens by default, `--budget N` (chosen)
`tools/edit` untrimmed is about 8,900 tokens (mean of the five tokenizers
in `tools/tokbench`); at 4,000 it keeps every top-level signature.

### Counting: a built-in estimate (chosen)
The command can't call a Python tokenizer. It counts a run of letters or
digits as one token per five characters (at least one) and every other
non-space character as one token. On cards of `tools/edit` it comes out
3% to 17% above the largest of the five tokenizers
(`tools/tokbench/run.sh`), so a card within budget by the estimate is
within budget for each. Tiny cards (under 300 tokens) can be a few percent
under; the budget is for big programs.

### Counting: exact, through tokbench
- **Rejected:** needs Python and a network download for a command that
  must work offline and fast.

## Decision: trimming order

The card is rendered at the first level that fits, from level 0 to 5:

1. drop private items (not `pub`, in a module that exports something or is
   not the root; a root module with no `pub` at all has no importers, so
   all of it counts as its surface);
2. drop doc comments;
3. drop parameters' names (types stay, `mut` and `move` stay);
4. drop fields, variants, contracts, error sets and the test names (a
   count stays);
5. keep only each item's kind and name, then only module names with counts.

A trimmed card ends with `trimmed to fit N tokens (~M); dropped: ...`
naming every level applied, and the JSON form has `dropped` and `budget`.
The footer counts toward the budget. If even level 5 is over, it is
printed as is.

## Rejected alternatives

- **Dropping whole modules, last first:** hides what exists; names only
  keeps the map of the program.
- **Dropping signatures before docs:** the signature is the contract; a
  doc is a hint.
