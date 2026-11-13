# 1110. Raise the language card budget to 16,000 tokens and check coverage instead of size

- **Status:** Accepted (amends [ADR 0276](0276-language-card-budget-8000.md) and [ADR 0817](0817-token-budgets-for-agent-pages.md) in the card's numbers)
- **Date:** 2026-11-13
- **Decided by:** the designer
- **Topic:** AI-first tooling
- **Spec rules:** tool.card.1, tool.card.2

## Context

For a language no model was trained on, the card is the language: an agent
uses what the card shows. The card has been at its 8,000-token budget since
ADR 0276, and each new feature of 0.1.0 (closures, `select`, distinct types,
default parameters, struct patterns, scoped capabilities, assertions) had to be
paid for with a trim, so features reached the compiler before they reached the
card. The designer judged that the card may grow: once models have seen the
language the size no longer matters, and the public corpus makes it matter less.

## Options considered

### Option A — 16,000 tokens, two layers, coverage checked (chosen)
- **Pros:** room for one worked, gate-checked example per feature; the core
  card stays small for a first read; a script, not a size limit, keeps the card
  complete.
- **Cons:** up to twice the context cost per full load; agents that load only
  the core card (`ail card`) must still follow its pointers.

### Option B — keep 8,000 and trim
- **Pros:** the cheapest load.
- **Cons:** features go missing from the card, which is how agents learn them.

### Option C — no budget
- **Pros:** nothing to trim.
- **Cons:** the card would drift into the quick reference.

## Decision

1. The full card (`docs/language-card.md`) budget is 16,000 tokens and the core
   card (`docs/language-card-core.md`) about 3,000, both on the five-tokenizer
   mean of ADR 0817 (`tools/tokbench`).
2. The two layers stay. The full card keeps one `##` section per topic, so
   `ail card TOPIC` (tool.card.2) works.
3. `tools/check-card-coverage.py`, in the `docs` gate suite, fails when a
   built-in method of `docs/design/builtin-methods.txt` or a rule family of the
   language chapters is on no card line, in no how-to and in no method-table
   row, unless `docs/design/card-coverage-exceptions.txt` names it with a
   reason. The exceptions list may shrink, and a new entry needs a reason.
4. Prefer one worked example per feature to prose; the gate already runs the
   card's `ail` blocks (`tools/gate/spec.ail`).
5. `llms-full.txt` at the repository root is the full card followed by
   `docs/how-to.md`, written by `tools/gen-llms-full.py` and checked in the
   `docs` suite; `llms.txt` links it.
