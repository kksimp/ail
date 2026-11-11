# 1052. Change certificate: `ail verify --diff --attest` and `ail verify --check`

- **Status:** Accepted
- **Date:** 2026-11-10
- **Decided by:** AI (delegated, ADR 0076). Slice E8 of the AI-first edge plan of 2026-11-10.
- **Builds on:** [0909](0909-ail-verify-diff.md) (`ail verify --diff`), [1022](1022-mutation-testing.md) (`--mutate`)
- **Topic:** 22 — Tooling
- **Spec rules:** verify.cert.1-4

## Context

"Verified AI-generated software" is a sentence until a gate can re-check it. `ail verify --diff` prints what a change
proved, what tests ran and what nothing looked at, but the report is not a file, and nothing ties it to the revision,
the files or the compiler that produced it.

## Options considered

### Option A: an unsigned JSON certificate that a re-run must reproduce (chosen)
- **Pros:** needs no keys, so no accounts matter; it is evidence a gate can reproduce, which is the claim; small (one object); reuses `verify.run.*` and `test.mutate.*` unchanged.
- **Cons:** proves reproducibility, not identity: anyone can write a certificate that reproduces.

### Option B: a signed certificate
- **Pros:** names the author.
- **Cons:** needs key management and trust roots, an accounts matter for the designer. Left for a later ADR.

### Option C: a flag on `ail test`
- **Cons:** the report spans check, diff and tests (the reason ADR 0909 rejected it).

## Decision

`ail verify --diff [--rev REV] [--mutate [--max N]] --attest OUT FILE` writes the `verify.run.1-5` report as one JSON
object `OUT`, with the commit REV resolves to (`base`), the git hash of each changed file, the compiler version, the
effect changes and, with `--mutate`, the mutation score and the surviving mutants in the changed functions. The exit
status is that of `--diff`. `ail verify --check CERT` re-runs the certificate against its `base` and exits 1, naming
each member, when any differs. No signing in this slice.

`ail task finish` will attach the certificate after the task ledger (lane D of the plan) merges.

## Rationale

Token usage: one file replaces a transcript of check, test and diff runs. AI understanding: a gate and an agent read
the same object. The reliability figure is the share of wrong final answers whose certificate lists the faulty
function as unverified or a surviving mutant.

## Consequences

`--check` costs as much as the original run. A certificate made with `--mutate` needs the tests green. A change in the compiler version differs by design.

## Rejected alternatives

- **Signing now:** see Option B.
- **A separate `ail certify` command:** a second entry point for what `verify` already computes.
