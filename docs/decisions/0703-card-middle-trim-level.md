# 0703. A middle trimming level for `ail api --card` (amends 0311)

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; lane 222, integrator decision)
- **Topic:** Tools (lane 222)
- **Spec rules:** api.card.3 (the trimming order) is amended; no new rule IDs
- **Amends:** [ADR 0311](0311-ail-api-card-budget-and-trimming.md)

## Context

ADR 0311 trims a card in fixed levels. Before this decision the levels went from
full docs straight to signatures without docs, and then to names. At a 2,000-token
budget the trimmer dropped every signature and kept only names, so the tool cards
written in AIL (lane 212, PR #252) lost almost all of their content at that size.
The levels are also not nested in size: the first level that fits is not always
the fullest one.

## Decision

1. A new level 4 keeps each public item's signature and the first sentence of its
   doc comment. It drops members, contracts, examples and any doc text past the
   first sentence. Level 5 is the old level 4 (signatures, no docs). Level 6 is
   names only.
2. `ail api --card` picks the richest level whose estimate fits the budget, not
   the first level that fits. The first-sentence level can be larger than the
   signatures-only level, so first-fit would never choose it.
3. The card's last line lists what the chosen level dropped, as before.

## Token table

tokbench, cl100k column only (`tools/tokbench/run.sh`), for `ail api --card --budget B`
on each package. "Before" is lane 212's binary (first-fit trimming); "after" is this
decision. Figures are measured, not estimated.

| Card | 8,000 before → after | 4,000 before → after | 2,000 before → after |
|---|---:|---:|---:|
| garlic | 3,026 → 5,750 | 3,026 → 3,036 | 120 → 131 |
| garlic-js | 1,793 → 3,467 | 1,793 → 1,803 | 85 → 96 |
| pdf | 99 → 109 | 99 → 109 | 99 → 109 |
| compiler | 2,639 → 4,756 | 2,639 → 2,661 | 106 → 116 |
| edit | 4,714 → 4,714 | 2,262 → 2,272 | 80 → 90 |
| std | 770 → 786 | 770 → 786 | 770 → 786 |

The 4,000 budget changes by 10 to 22 tokens per card: the trimmed line names the
new level. `pdf` is names-only at every budget in this range, so its card does not change.

## Rejected alternatives

- **Keep the first-fit rule and reorder the levels.** Rejected: the first-sentence
  level can be larger than the signatures-only level for a package with few members,
  so no fixed order makes both reachable.
- **Signatures without any doc sentence at a middle size.** Rejected: the existing
  signatures-only level already does that.
- **Split packages per module to reach 2,000 tokens.** Not rejected; it is the next
  step, not this decision. At 2,000 tokens nothing here keeps signatures for garlic
  or compiler, because their signatures alone are larger than the budget. A per-module
  card needs a change to how `ail api --card` finds packages, which this ADR does not make.

## Consequences

- At 8,000 tokens, the garlic and compiler cards carry public signatures and their
  first doc sentences.
- At 2,000 tokens these packages still get names only, so the cards there do not
  show signatures. Meeting that target needs per-module cards (see
  [the cards README](../design/cards/README.md)).
- `bootstrap/tests/api_card.rs` has a fixture test that the middle level is chosen
  for a 60-function package.
