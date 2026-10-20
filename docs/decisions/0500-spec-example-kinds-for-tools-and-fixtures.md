# 0500. Spec examples for tools, fixtures and flags: `tool=`, `fixture=` and `flags=`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** Conformance (lane 151, batch 2)
- **Spec rules:** none new
- **Confirms:** [ADR 0002](0002-normative-spec-with-rule-ids.md), [ADR 0335](0335-explain-and-fix-coverage.md)

## Context

Some rules are not about a program that runs. Effects denied by `--deny` and
`--deterministic` are rejected by `ail check`, but only with flags. The semantic
API and `ail explain`/`ail fix` are commands, and their rules describe their
output. Package rules need a directory, and a fence holds one file. The
case fixtures in `bootstrap/tests/cases` already exercise the package rules.

## Decision

- `flags=A,B` checks the block with those words added to `ail check`.
  `error=RULE` with it covers the flags' rejection.
- `tool=A,B,FILE expect=TEXT` runs `ail A B FILE` on the block and requires exit
  status 0 and that stdout contains `TEXT`. `DIR` is the block's directory, with
  the block as `main.ail`. Output formats are non-normative
  ([semantic-api.md](../design/semantic-api.md)), so `expect` is a short fragment
  the rule describes, not a full output.
- `fixture=DIR` checks `ail check` on the case fixture `bootstrap/tests/cases/DIR`
  and requires each rule the block cites in `rules=` to be reported there.
- `explain.cover.*` is covered by a `tool=` example and is also enforced by
  `bootstrap/tests/explain_coverage.rs`, which runs in `./ail selftest`.

## Rejected alternatives

- **Put the package manifest in the fence.** Rejected: a manifest needs sibling
  modules, and the case fixtures already hold them.
- **Make the tool examples match the full output.** Rejected: the output format
  is non-normative and would break whenever a column moves.
- **Leave the package and CLI rules uncovered.** Rejected: the batch chapters
  must have zero uncovered rules, and these examples check the real behaviour.

## Limits

`tool=` examples check that the tool exits 0 and prints the fragment. A rule
about the exact text, such as `api.tok.2`'s token layout, is only partly checked
by the fragment. The `usage error` of `--deny` with an unknown capability (exit
status 2, `eff.cap.2`) is not checked by the harness and remains described only.
