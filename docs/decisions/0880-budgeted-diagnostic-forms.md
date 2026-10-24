# 0880. Token-priced diagnostics: one-line and expanded forms under `--budget`

- **Status:** Accepted
- **Date:** 2026-10-24
- **Decided by:** Designer (2026-10-23, the item, ADR 0868 item 6); the forms and the choice by AI (delegated, ADR 0076)
- **Amends:** [ADR 0394](0394-token-budgets-on-every-command.md) (token budgets), [ADR 0868](0868-ai-first-tooling-after-rust-removal.md) item 6
- **Spec rules:** `budget.out.4`, `budget.out.7`
- **Topic:** 22 — Tools; 20 — AI semantic API

## Context

ADR 0868 item 6 gives every diagnostic a one-line and an expanded form, with `--budget`
(ADR 0394) choosing between them. Under ADR 0394 a budget only drops whole records, so
a budgeted `ail check` shows a few whole records and nothing at all of the rest.

## Decision

- **Forms.** A record's expanded form is the whole record (header, source line, caret,
  `help:`, `note:`, `fix:`); its one-line form is its first line, which already carries the
  file, line, column, severity, rule and message. A JSON line is its own one-line form.
- **Choice** (`budget.out.7`). Output that fits is unchanged (`budget.out.1`). Otherwise the
  first record shown is expanded, the rest are counted at their one-line cost until the next
  would pass the budget, and then records are expanded in rank order while the budget lasts.
  The expanded records are a prefix, so the most severe ones keep their help and fixes.
- **Note.** It says which records are one-line and gives `--from K`, which shows record
  `K + 1` expanded; the JSON note gains `"full"`.
- **Routing.** `ail check --budget` runs the native check under the native budget wrapper;
  the wrapper is told the command so a bare `--budget` takes its default (`budget.out.5`).

Measured (estimated tokens, ADR 0311): three `expr.bind.3` errors (285 tokens) under
`--budget 200` showed 1 record before and all 3 after (1 expanded). `ail check --budget
compiler` (286 warnings, ~50,800 tokens, default 1,500) showed 10 records before and 12
after (4 expanded): its `expr.match.3` headers list every variant, so their one-line form
is long. The gain grows with how much of a record lies below its first line.

## Consequences

The code is `compiler/budget/forms.ail` plus a few lines in `budget.ail`. Every command
under `--budget` gets the forms; commands whose records are one line are unchanged.

## Rejected alternatives

- **Each command printing its own short form:** about fifteen commands to change, against
  ADR 0394's one-wrapper design.
- **All one-line once over budget:** the most severe record loses the help and fix an
  agent usually needs to act at once.
- **Expanding the records that fit best (knapsack):** better packing, but the expanded set
  would no longer be a rank prefix, and `--from` could not say which comes next.
- **A shorter JSON form dropping `notes` and `fixes`:** the JSON consumer is a tool that
  can drop fields itself; a second JSON shape is one more schema.
