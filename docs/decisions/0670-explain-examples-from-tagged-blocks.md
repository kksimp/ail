# 0670. Explain examples: worked examples count, and tagged blocks fill the gaps

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 22 — Tools (`ail explain`)
- **Spec rules:** explain.show.1, explain.list.1
- **Builds on:** [ADR 0146](0146-ail-explain.md) (explain reads the spec), [ADR 0335](0335-explain-and-fix-coverage.md) (explain examples)

## Context

`tools/rules-audit.sh` check 4 reported 386 `ail explain` entries with no
example. Two causes, neither a missing example in the text:

1. A rule with a worked example (`error=RULE`, printed as `rejected` and
   `repaired`) has no plain `example` in the JSON, because the worked example
   replaces the nearest one (`explain.rs`). The audit counted it as missing.
2. Many tool rules (`diff.out.*`, `edit.*`, `replay.*`) are covered by a block
   tagged `rules=RULE` that sits in a different section, so the nearest block
   after the rule is not theirs.

## Decision

- `tools/rules-audit.sh` check 4 counts a worked example (`rejected`) as an
  example. That is 69 of the 386.
- `ail explain` falls back, when a rule's section has no example, to the first
  block whose `rules=` tag names the rule. A `tool=` block prints its command
  as `$ ail ...` (with `< FILE` for `stdin=`) and `# package NAME` for `pkg=`,
  then at most four lines of code. The spec harness already checks every such
  block, so a fallback example cannot drift from the behaviour it shows.
- New examples are written where the rule is explained: an `error=RULE` block
  followed by its repaired block, a `rules=RULE` block for a rule that is
  not an error, or a `norun` block for a trap. Each is checked by the spec
  harness (`./ail selftest --test spec`), which requires an `error=` block to
  report its rule and an untagged block to check clean.
- The mean `ail explain` entry is kept under 400 characters (the test in
  `explain_coverage.rs`). Examples are trimmed to the shortest program that
  shows the rule.

## Rejected alternatives

- **A separate examples file keyed by rule ID.** Each example would live
  outside the spec, so it could drift from the rule text, and the harness
  would not check it without a new loader. Rejected on AI understanding
  (criterion 2).
- **Duplicating each tagged block after its rule.** Costs spec tokens for
  every copy and needs a second source of truth. Rejected on token usage
  (criterion 1).
- **Raising the 400-character test limit.** Would let every explanation grow
  without a budget. Rejected on token usage (criterion 1).
- **Only plain examples, no worked-example credit.** Would force a second,
  duplicate example for each `error=` rule. Rejected on token usage.

## Consequences

- `ail explain` shows an example for every rule whose text is covered by a
  tagged block. Before this ADR, 386 rules had none; after it, 102 remain
  (the list is in the PR body). Those are mostly the older chapters (09, 10,
  12, 14, 19, 20) and rules the bootstrap cannot yet show (`trace.val.2`).
- Rules whose only example is a `pkg=` package show the command, not the
  package text. An agent must open the package in the spec to see it.
- Some examples are not verifiable by a single program (for example
  `std.date` until the module lands). Those stay open.
