# 0731. Output of `ail api --diff` and its token budget

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** api.diff.5
- **Builds on:** [ADR 0729](0729-api-diff-command.md), [ADR 0394](0394-token-budgets-on-every-command.md)

## Context

Commands that print to an agent have a token budget (ADR 0394). An API diff
of a large package can be long; the breaking lines matter most.

## Decision

Text: one line per added (`+`) or removed (`-`) declaration, a `~ ID` line
per changed one with indented `KEY: OLD -> NEW` lines, `  breaking: WHY`, and a
summary line `N added, N removed, N changed; N breaking`. `--json` gives one
JSON line each plus `{"summary":true,...}`. Exit 0/1/2 as `ail diff`.
Under `ail budget`, the default is the `diff` one (2,000 tokens) and the
output ranks error, fail, breaking, warning, other lines, then the summary,
so breaking changes survive trimming.

## Rejected alternatives

- **A new exit status for breaking.** Scripts can read the summary; one
  status scheme for all diff tools is simpler.
- **Sorting by id.** Output follows declaration order of the new version, then
  removals, which reads as the file does.
