# 0909. `ail verify --diff`: what a change proved, what tests exercised, what stays unverified

- **Status:** Proposed
- **Date:** 2026-10-25
- **Decided by:** Designer (2026-10-25, chose the feature on a decision card); the details by AI (delegated, ADR 0076), for integrator review
- **Builds on:** [ADR 0396](0396-ail-diff-semantic-diff.md) (`ail diff`), [ADR 0882](0882-edit-impact.md) (impact and affected tests), [ADR 0211](0211-contract-runtime-checks.md) (contracts), [ADR 0394](0394-token-budgets-on-every-command.md) (`--budget`)
- **Topic:** 22 — Tooling
- **Planned rule keys:** `verify.run.*`, `verify.report.*`

## Context

After a fix, an agent says "done" because the last command printed no error. Nothing separates what the compiler proved from what a test happened to run, or says what nothing looked at. The result is regressions found one round later.

## Decision

`ail verify --diff [--rev REV] PROGRAM` compares PROGRAM with REV (default `HEAD`, as `ail ask --affected`) and prints one report in four parts, each a list of declarations by id (ADR 0396):

1. **proved:** changed declarations whose program checks with no errors, with their `requires`/`ensures` that the checker discharged statically.
2. **tested:** affected tests (`api.impact.2`) that ran, with pass or fail, and the changed declarations each test reaches.
3. **unverified:** changed or affected declarations that no test reaches, contracts checked only at run time and not hit by a test, and effects that changed (ADR 0396 body hashes) without a test reaching them.
4. **new failures:** diagnostics and failing tests present now and absent at REV. Failures present at both are listed apart as `old failures`.

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
