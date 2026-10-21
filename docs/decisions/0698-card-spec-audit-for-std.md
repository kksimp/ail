# 0698. A std audit: every std function has a spec mention

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Standard library
- **Spec rules:** none added (the audit checks every std module)
- **Criterion that decided:** AI understanding: a function the spec never names is one an agent can only guess at, so the audit checks names, not wording.
- **Builds on:** [ADR 0002](0002-normative-spec-with-rule-ids.md), [ADR 0483](0483-rule-id-audit.md)

## Context

Lane 217 asked for an audit that matches the std card against the spec, in
both directions. The card (`docs/design/cards/std.md`, from `ail api --card`)
lists types only, not functions: its 114 lines are struct, enum and error
declarations. The functions live in the `MODULES` table of
`bootstrap/src/stdlib.rs`, and the language card (`docs/language-card.md`) and
the spec describe them. So the audit compares the table with the spec.

## Decision

`tools/card-spec-audit.py` reads `MODULES` from the Rust source, so the
audit and the checker cannot drift. It reports each std function whose name
appears nowhere in `docs/spec/` in call form (`name(`) or as a backticked
name. It is check 5 of `tools/rules-audit.sh`. The check fails the audit when
the list is not empty.

Results at this head: 95 std functions; 0 without a spec mention. The first
draft matched bare words, which made every name match; it was too weak and
was replaced.

## Rejected alternatives

- **Match `module.name` text.** The spec writes `find_all(re, text)` without
  the module, so this reported 9 false gaps.
- **Check the reverse (spec names not in `MODULES`).** Spec file names such as
  `regex.md` and prose words gave noise; no real stale name was found.
- **Parse the language card.** The card is hand-written prose with mixed
  notation; a parser would be fragile for a check with no current findings.

## Measurements

`docs/language-card.md`: 7,044 tokens (budget 8,000). `docs/design/cards/std.md`:
808 tokens (types only). No text changed for the card.

## Consequences

- A new std function now fails the audit until the spec names it. The rule
  asks for a runnable example in the spec, which this ADR does not add.
- The audit does not check the language card's wording; a function can be in
  the spec and still missing from the card.
