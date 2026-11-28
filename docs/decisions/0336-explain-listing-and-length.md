# 0336. `ail explain` entries average under 120 tokens

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20). Criterion: tokens, measured with `tools/tokbench/run.sh`.
- **Topic:** 20 AI semantic API (`ail explain`)
- **Spec rules:** `explain.cover.4` ([22-tools](../spec/22-tools.md))

## Context

An agent reads an entry whole. Measured over the 556 rules that existed when
this lane started, the text form averaged 178 tokens (mean of the five
tokenizers; 166 for `cl100k`, 203 for `mistral-spm-v1`). Where the tokens went,
per entry: header 40 characters, rule text 288, the ADR lines 111 (titles),
examples 132.

## Decision

The text form of `ail explain RULE` prints the file name without its
directory, the rule text with markdown link targets removed, no separate ADR
list (the text already cites them; `--json` keeps ids and titles), each plain example
cut to 6 lines (a rejected program and its repair stay whole), and a rule text longer than 420 characters (400 since the 2026-11-20 amendment, 380 since [ADR 1314](1314-explain-text-cut-380.md)) cut after its last
whole sentence with a pointer to the line above. `--json` is unchanged except
for a `fix` key. The test suite fails when the mean entry passes 400
characters (`explain.cover.4`), which tokbench puts at about 120 tokens.

Result: mean 116 tokens (cl100k 108, o200k 108, claude-legacy 115,
mistral-tekken 117, mistral-spm-v1 132), down from 178.

## Rationale

Tokens decide. Link targets and ADR titles are the largest parts an agent
can't act on in a terminal; the cut text is still one `ail explain --json` or
one `Read` away.

## Rejected alternatives

- **Shortening the specification text itself.** The specification is also read
  by people and in full; the cap belongs to the lookup.
- **No example.** Examples are what turn a rule into a repair; they stay,
  shortened.

## Amendment (2026-11-20): rule text cut at 400 characters

- **Decided by:** AI (delegated, ADR 0076), during the 0.2 release candidate.

The 0.2 chain added ten `std.attach`, `std.screen` and `std.ui` rules and several examples, and the mean entry
reached 400.6 characters, failing `explain.cover.4`. A rule text longer than 400 characters (was 420) is now cut
after its last whole sentence. Measured over the 1,623 rules: mean 393.1, longest unchanged (`native.subset.1`, whose
constructs table is not cut). The cut keeps the pointer to the spec line, so nothing becomes unreachable; the limit of
400 on the mean is unchanged.

Rejected: shortening the longest entries' spec text by hand (each new rule's text is normative and was over 420 already,
so trimming it saves nothing until it is under the cut; reaching a margin would mean rewording dozens of rules during a
release freeze); raising the 400 limit (it is the token budget of ADR 0336); cutting worked examples (they are whole
programs whose repair must stay readable).
