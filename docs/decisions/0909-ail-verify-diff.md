# 0909. `ail verify --diff`: what a change proved, what tests exercised, what stays unverified

- **Status:** Proposed
- **Date:** 2026-10-25
- **Decided by:** Designer (2026-10-25, chose the feature on a decision card); the details by AI (delegated, ADR 0076), for integrator review
- **Builds on:** [ADR 0396](0396-ail-diff-semantic-diff.md) (`ail diff`), [ADR 0882](0882-edit-impact.md) (impact and affected tests), [ADR 0211](0211-contract-runtime-checks.md) (contracts), [ADR 0394](0394-token-budgets-on-every-command.md) (`--budget`)
- **Topic:** 22 — Tooling
- **Rule keys:** `verify.run.1-4`

## Context

After a fix, an agent says "done" because the last command printed no error. Nothing separates what the compiler proved from what a test happened to run, or says what nothing looked at. The result is regressions found one round later.

## Decision

`ail verify --diff [--rev REV] PROGRAM` compares PROGRAM with REV (default `HEAD`, as `ail ask --affected`) and prints one report in four parts, each a list of declarations by id (ADR 0396):

1. **proved:** whether the program checks with no errors, and the changed declarations.
2. **tested:** the affected tests (`api.impact.3`) that ran, with pass or fail.
3. **unverified:** that no test ran for the changed declarations, functions whose effects changed, and skipped tests.
4. **failures:** check errors and failed tests.

Version 1 is coarse on purpose: the impact graph names the tests a whole change reaches, not which test reaches which declaration, and REV's own check and test results are not re-run, so a failure is not yet split into new and old. Per-declaration reach, a `new`/`old` split and a list of contracts only checked at run time are the next slices.

Exit status is 0 when the check is clean and no new failure exists, 1 otherwise. "Unverified" never fails the command; it is the information. `--json` prints the same four parts; `--budget N` trims by rank: new failures first, then unverified, tested, proved. Everything is computed only when asked for (ADR 0868).

Version 1 uses only features that exist. `ail try` ([0893](0893-ail-try.md)), `--mutate` (0896) and `ail infer` (0895) are Proposed; when they land, `verify` reports mutation survivors under *tested* and suggests contracts under *unverified*.

## Rationale

Token usage ranks first (ADR 0076): one report replaces check, diff, affected and test runs read separately. AI understanding ranks second: an agent can no longer mistake silence for evidence.

## Consequences

The report is only as good as the reference graph: tests that read files outside the program are listed as reached but flagged `disk` (ADR 0882). A run costs one check and the affected tests.

## Rejected alternatives

- **A single pass/fail verdict:** hides exactly the unverified part agents need.
- **Coverage-based "tested":** needs a recorded run per test and goes stale (rejected in ADR 0882).
- **A flag on `ail test`:** the report spans check, diff and tests; a separate command keeps each tool small.
