# 0287. Inferred error sets in `ail api`, worked examples in `ail explain`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** AI-first tooling
- **Spec rules:** `err.api.1`

## Context

`fn sum(...) !int` hides which errors it returns, so an agent calling it had to read its body. `ail explain` for an `err.*` rule printed the nearest example in the file, which sometimes belonged to another rule.

## Options considered

### Option A — `fails_with` in `ail api`, and `error=RULE` blocks as worked examples (chosen)
- **Pros:** the spec's own conformance tests keep the examples true; the rejected program and its repair come from one place.
- **Cons:** a rule needs two blocks to get a worked example.

### Option B — keep the nearest example
- **Pros:** no spec edits.
- **Cons:** it can show another rule's code.

## Decision

`ail api` gives each fallible function `fails_with` (sorted error type names, after inference); `--text` prints `fails with A | B` under an inferred signature. `ail explain RULE` shows, when the spec has a block tagged `error=RULE`, that block as `rejected example` and the plain block after it as `repaired`, and omits the nearest example then. The JSON has `rejected` and `fixed`. Every `err.*` rule a program can trigger now has such a pair.

## Rationale

AI understanding: a rejected program with its repair is the shortest correct explanation. Tokens: `fails_with` is a few tokens per function and replaces reading the body.

## Rejected alternatives

- Keeping the nearest example (Option B), as above.
- A new tag for the repaired block: the plain block already is checked as a program.
