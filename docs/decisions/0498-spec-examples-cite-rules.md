# 0498. Spec examples cite the rules they demonstrate, and uncovered rules fail the gate

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** Conformance (lane 151, batch 2)
- **Spec rules:** none new; extends [ADR 0002](0002-normative-spec-with-rule-ids.md) in `docs/spec/README.md`
- **Confirms:** [ADR 0002](0002-normative-spec-with-rule-ids.md) (spec examples as conformance tests),
  [ADR 0172](0172-spec-conformance-harness.md)

## Context

Every fenced `ail` block in `docs/spec/` is a conformance test, but nothing
said which rule a block demonstrates. A rule with no example could ship
untested, and nothing could show it. The batch changes the types, expressions,
standard modules, effects and tools chapters, so every rule in those chapters
needs a runnable example.

## Decision

- A fence may carry `rules=ID,ID`. A checked block counts as an example of each
  listed rule. `error=RULE` counts the same way for `RULE`.
- A rule ID cited by an example must be defined in `docs/spec`. Otherwise the
  test fails.
- `bootstrap/tests/spec.rs` fails when a rule of the batch chapters (types,
  expressions, std, effects, tools) has no example. `AIL_LIST_UNCOVERED=1` makes
  it print every uncovered rule of every chapter, for example with
  `./ail selftest --test spec -- --nocapture`.

## Rejected alternatives

- **A separate coverage table in `docs/status.md`.** It drifts from the spec and
  nothing checks it. Rejected: the fence is next to the rule and the test reads it.
- **A comment inside the code (`# covers type.num.1`).** It costs tokens in every
  example that is loaded into context, and a comment can name a rule the code
  doesn't demonstrate. Rejected: the fence tag is one line and the harness checks
  the ID.
- **Listing uncovered rules in a generated file.** Rejected: the test prints them
  on demand, and a generated file is one more thing to keep current.

## Measurements

Not applicable: this changes the test harness and the spec's examples, not the
language syntax, so `tools/tokbench` does not apply. Example count after this
change: 313 checked, 51 skipped (`skip=bootstrap`); zero uncovered rules in the
five batch chapters.
