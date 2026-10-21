# 0817. Token budgets for the agent-facing pages, and the quick-reference trim rule

- **Status:** Accepted (amends [ADR 0276](0276-language-card-budget-8000.md) in part: the card budget is now stated on the five-tokenizer mean)
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; number allocated by the integrator)
- **Topic:** Tooling (token measurement)
- **Spec rules:** none

## Context

Two pages are loaded into an agent's context to write or read AIL: the language
card and the quick reference. Each has a token budget, and nothing checked them
on every change. The card's budget was stated per tokenizer ("on every tokbench
tokenizer", [ADR 0276](0276-language-card-budget-8000.md)). The quick reference
had a budget of 12,518 but no decision recording it, and it grew past that budget
in the day-3 batch before #368 trimmed it.

## Decision

1. The budgets are measured on the five-tokenizer mean of `tools/tokbench`
   (the `mean` column, offline tokenizers only):
   - `docs/language-card.md`: **8,000** tokens.
   - `docs/quick-reference.md`: **12,518** tokens.
2. `tools/tokbench/run.sh --check-budgets` prints each mean with its budget and
   exits 1 if either is over. The budgets live in one table, `BUDGETS` in
   `tools/tokbench/tokbench.py`. The gate runs this mode.
3. Any addition to the quick reference is paid for by a trim in the same pull
   request. A change that takes the page over its budget does not merge.

## Rationale

The quick reference is the page an agent loads to write AIL, so its size is a
cost paid by every agent on every task. One number per page, measured the same
way for both, is easy to check and easy to explain. A rule that additions carry
their own trim keeps the page inside its budget without a separate cleanup pass.

The mean is looser than the per-tokenizer check that ADR 0276 described. A page
that passes the mean can still exceed one tokenizer's count. This decision keeps
the mean, the figure the gate reads, and leaves per-tokenizer limits as a
possible tightening (see Consequences).

## Consequences

- The budget check can fail on a branch whose quick reference is over budget.
  The check is a gate, so that branch cannot merge until it is trimmed or the
  budget is changed by a new decision.
- The card budget is unchanged at 8,000. The quick-reference budget is now
  recorded.
- Follow-up: whether per-tokenizer limits should replace the mean is open.

## Rejected alternatives

- **Per-tokenizer limits for both pages:** stricter than the mean, and a page
  close to the limit on one tokenizer would fail on that tokenizer alone; kept
  as a possible tightening.
- **No budget for the quick reference:** on the day-3 batch head before #368 the
  page measured 12,818.8, over its budget with nothing to stop it; an unchecked
  page drifts.
- **Budget only in the README:** the number would not be checked by the gate.
