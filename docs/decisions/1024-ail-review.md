# 1024. `ail review`: a semantic summary of a change

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076), from the lane plan of 2026-11-09
- **Topic:** 22 — Tooling
- **Spec rules:** api.rev.1-11
- **Builds on:** [ADR 0396](0396-ail-diff-semantic-diff.md), [ADR 0729](0729-api-diff-command.md), [ADR 0882](0882-edit-impact.md), [ADR 0394](0394-token-budgets-on-every-command.md)
- **Amends:** [ADR 0894](0894-effect-changes-in-ail-diff.md) (the effect changes are built here, as a section of `ail review`)

## Context

Judging a pull request means reading its diff, and the diff costs tokens in proportion
to the lines that changed, not to what the change means. A reviewer needs five answers:
does the program do more than before, did the public API break, did a contract move,
did anything get slower, and is the new code tested. The checker already holds all five
for two program states (`ail diff`, `ail api --diff`, edit impact).

## Options considered

### Option A: `ail review BASE..HEAD`, one budgeted report over the existing passes (chosen)
- **Pros:** one call and one cheap answer; no new analysis, only a view over passes that exist;
  the budget is the reviewer's own choice; `--deny-new` gives a gate without hosted CI.
- **Cons:** the program must load at both revisions; a version with check errors is read only as
  far as it resolves.

### Option B: an `effects:` section in `ail diff` (ADR 0894 as written)
- **Pros:** reviewers already run `ail diff`.
- **Cons:** `ail diff` lists every change and has no budget; the effect lines would be buried, and
  API, contracts and test reach would still be separate calls.

### Option C: read the textual diff
- **Pros:** exists.
- **Cons:** the cost is the thing to remove, and it cannot show an inherited effect or an unreached function.

## Decision

`ail review BASE..HEAD [PATH] [--budget N] [--json] [--deny-new]` (api.rev.1-11) prints a first line
with the count of each section, then the sections in the order kept: `effects` (a function
that is `main` or `pub` and gains an effect is `NEW CAPABILITY`, listed first), `api` (added,
removed and changed public declarations, breaking ones first), `contracts` (changed `requires` and
`ensures`), `perf` and `untested` (changed or new functions no `@test` function reaches). The longest
prefix that fits the budget is shown, never a line cut in the middle, and the last line says what
was cut and the budget that shows all. The default budget is 500.

`--files OLD NEW` compares two paths without git, which is how the spec examples run.
`--deny-new` makes a `NEW CAPABILITY` line exit 1. This is the `--deny-new` of ADR 0894, on the
command that now holds the section.

**Amending ADR 0894.** Its `effects:` section moves from `ail diff` to `ail review`. `ail diff`
already prints a changed function's `effects` key, so it loses nothing.

**Perf hook.** The `perf` section is empty until `ail perf --explain` (ADR 1021) is on main. Its
remarks for the two versions are then compared in `rv_perf_items` and each loop whose verdict
differs is one line.

**Untested.** Edit impact (ADR 0882) gains the list of changed functions no test reaches; it is
read by `ail review` and not printed by `ail ask --affected`.

## Measurement

The slice's tokbench task is "judge this pull request". The before-and-after token counts (the
diff against the report) are added here once a Mac run has measured them.

## Rationale

Token usage ranks first (ADR 0076): the report costs what the reviewer chooses, and the lines
that matter most for a reviewer, a widened capability and a breaking change, are kept longest.

## Consequences

The command is native only. `ail mcp` can serve it later as `ail_review`.

## Rejected alternatives

- **Cutting inside a line or cutting the longest section:** a report's shape would change with the
  change, not the budget.
- **Failing on any effect change:** most effect changes are internal; only a widened `main` or
  `pub` function widens what the program can do.
