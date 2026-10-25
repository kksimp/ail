# 0903. `ail new NAME`: a project scaffold

- **Status:** Accepted
- **Date:** 2026-10-25
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 22 — Tools
- **Spec rules:** tool.new.1-2 ([22-tools](../spec/22-tools.md))

## Context

The adoption-gaps review (2026-10-24) listed a missing scaffold: Python has
`python -m venv` plus a file, Rust has `cargo new`, and a newcomer to AIL has
no command that makes a first program that runs and tests.

## Decision

`ail new NAME` (`compiler/scaffold/scaffold.ail`, one dispatch line in
`compiler/main.ail`) creates `NAME/ail.pkg` and `NAME/main.ail` with a `greet`
function, a `main` and a `@test`, and prints `ail run NAME` and
`ail test NAME/main.ail`. Names are lowercase letters, digits, `_` and `-`; an
existing path is never touched (tool.new.1-2). Native only, no Rust (ADR 0873).

## Rejected alternatives

- **Templates (`ail new --lib`, `--cli`).** More choices before the first
  program; add them when a second shape is wanted.
- **Interactive prompts.** Agents cannot answer them.
- **Git init and a README.** `ail new` makes a package, not a repository.
- **A launcher-script verb.** The native CLI is the product (integrator note).
