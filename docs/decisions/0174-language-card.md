# 0174. A language card for AI agents, under 4,000 tokens

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** AI-first tooling
- **Spec rules:** none

## Context

The [quick reference](../quick-reference.md) is about 4,300 tokens (mean of the
tokbench tokenizers) and has grown with each feature. An agent writing AIL
should be able to load the whole language in one small read.

## Options considered

### Option A — a separate card cut from the quick reference (chosen)
- **Pros:** **tokens** decide: 3,477 mean and 3,915 on the costliest
  tokenizer, under the 4,000 budget on all five; every example still checks
  and runs; derived by cutting, so the two don't contradict.
- **Cons:** two documents to keep in step.

### Option B — shrink the quick reference itself
- **Pros:** one document.
- **Cons:** loses the long form that explains edge rules and diagnostics.

### Option C — generate the card from the spec
- **Pros:** can't drift.
- **Cons:** the spec is normative prose, many times the budget; selecting
  what matters is a judgement, not a transform.

## Decision

`docs/language-card.md`, versioned in its header (`Language card v1,
2026-10-20, bootstrap milestone 6`) with its token count. It is cut from the
quick reference: the long positional-argument, bare-variant, slice and
`for mut` notes and some examples are dropped; every kept example checks.
The budget is checked with `tools/tokbench/run.sh docs/language-card.md`;
raise the version and the count whenever it changes.

## Rejected alternatives

- Generating it from the spec (Option C), and merging it into the quick
  reference (Option B), as above.
