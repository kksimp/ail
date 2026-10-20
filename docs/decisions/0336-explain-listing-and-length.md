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
cut to 6 lines (a rejected program and its repair stay whole), and a rule text longer than 420 characters cut after its last
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
