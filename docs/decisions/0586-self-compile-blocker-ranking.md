# 0586. Self-compile blockers are ranked by problem count and owned by lane

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; lane 181 of the native self-compile push)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md)
- **Builds on:** [ADR 0585](0585-self-compile-survey-tool.md) (the survey)

## Context

Several lanes work in parallel on the native backend (lanes 120 and 170–187).
Each needs to know which construct to take next, and which lane owns it.

## Decision

`docs/design/self-compile-status.md` lists the top blocking constructs,
ranked by the number of `native.subset.1` problems the survey reports for
them, with the lane that owns each one from the lane briefs. Ownership
follows the briefs: lane 120 owns structs, methods, enums, `match` and
modules; lanes 171–177 own heap, strings, maps, closures, optionals and
bounds. A construct no brief covers is listed as unowned, with a proposed
owner, rather than given to a lane by guesswork.

The ranking is re-run after each merge into the batch branch. A construct
moves up or down by count, and a construct whose count reaches zero leaves
the table.

Rejected alternatives:

- **Rank by estimated effort.** Effort estimates are guesses and aren't
  measurable with the survey.
- **Rank by how often agents write a construct.** This matters for the
  language's design, but it is not what blocks the self-compile. It is
  tracked by the corpus lanes (144, 83) instead.

## Consequences

- The table is a plan for the next merge, not a schedule. Lanes can
  reorder work when the survey changes.
- Cross-module counts are excluded from the ranking until module loading
  lands, because they are not blockers of a construct.
