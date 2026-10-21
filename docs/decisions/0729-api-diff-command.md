# 0729. `ail api --diff`: the public API change between two versions

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** api.diff.1, api.diff.2, api.diff.4
- **Builds on:** [ADR 0396](0396-ail-diff-semantic-diff.md) (`ail diff`), [ADR 0151](0151-stable-symbol-ids-and-references.md) (ids)

## Context

An agent upgrading a package, or checking a refactor, needs to know what
changed in the public API, not in every body and test. `ail diff` reports
everything and runs tests; the rest of the report buries the API lines.

## Decision

`ail api --diff OLD NEW` (two files or package directories) and
`ail api --diff --rev REV PATH` reuse the loading of `ail diff` (`at_revision`)
and the outline of `ail api`. Only public declarations count: `pub` ones, the
variants of a public enum and their fields, the methods of a public trait,
and the `impl` blocks of a public type. Compared keys are signature,
effects, `fails_with`, `requires`, `ensures` and `default`. Bodies, docs and
tests are not compared and no tests run. A single file needs no `main`.
A side with errors is not compared (exit 2).

## Rejected alternatives

- **A flag of `ail diff`.** It would mix two reports and run tests by default.
- **Compare private declarations too.** They are not an API; `ail diff` has them.
- **Compare doc comments.** Doc edits are frequent and never break a caller.
