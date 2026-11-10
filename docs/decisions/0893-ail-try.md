# 0893. `ail try`: check and test a patch in memory

- **Status:** Accepted (2026-11-10; first slice built, see Implementation)
- **Date:** 2026-10-24
- **Decided by:** Designer (2026-10-24, chose the feature on a decision card); the details by AI (delegated, ADR 0076), for integrator review
- **Builds on:** [ADR 0397](0397-ail-edit-structured-edits-by-id.md) (`ail edit`), [ADR 0399](0399-compact-edits-and-diff-hunks.md), [ADR 0868](0868-ai-first-tooling-after-rust-removal.md) item 3
- **Topic:** 20 — AI semantic API

## Context

An agent edits, runs check, runs tests, reads the failure and edits again, with the working tree changed at every step. A wrong guess must be undone by hand.

## Decision

- `ail try` takes the same hunks as `ail edit FILE --test` (ADR 0399), applies them to an in-memory copy of the program, runs `ail check` and the affected tests (`ail test --affected`), and prints a verdict. No file is written.
- Verdict, one line each: `check ok|N errors`, `tests N pass|M fail`, `effects +net -fs on f` (ADR 0894 form), and the patch's token cost. Failures print the first diagnostic in the `--budget` form.
- `ail try --apply` writes the files only if the verdict is clean.
- Several patch sets in one call (`--set A --set B`) give one verdict line each, so an agent compares options in a single command.
- Planned rule keys: `try.run.*`, `try.verdict.*`.

Cost when unused (ADR 0868): none. The feature is a separate command or pass, so a program that does not use it gets no extra code, no runtime check and no slower build.

## Consequences

Needs the compiler to check an overlay of file contents rather than the disk (the incremental front end already plans this). Tests that touch the disk run against the real disk and are reported as such.

## Implementation (2026-11-10)

The first slice is `tools/ailtry/ailtry.ail`, spec `try.*` in [22-tools](../spec/22-tools.md). It uses a scratch copy under `.ail/try/` instead of an in-memory overlay, which removes the dependency on the incremental front end; the overlay is a later speed-up with the same output. It runs the program's whole test set on the copy; `--affected` needs a git base, which the copy does not have, and comes with the overlay. The effects line is left to `ail edit --impact` for now. The token cost is the `ail api --card` estimate (ADR 0311), shown as `~N tokens`. Sets are `--set FILE` (or stdin for one) rather than the `--set A` names above, so a patch set is a file an agent already wrote. `ail_try` for `ail mcp` follows once the MCP completeness lane merges.

## Rejected alternatives

- **A git stash or branch per try:** slow, and leaves state when the process dies.
- **Dry-run flag on `ail edit`:** `ail edit` only edits; the verdict needs check and tests, which belong to a separate command.
- **Always run every test:** the affected set is the point.
