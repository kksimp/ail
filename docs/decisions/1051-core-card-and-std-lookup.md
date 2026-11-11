# 1051. Core card, `ail card` and `ail ask std`

- **Status:** Accepted
- **Date:** 2026-11-10
- **Decided by:** AI (delegated, ADR 0076), within the AI-first edge plan the designer approved for 0.1.0
- **Topic:** 22 — Tooling
- **Spec rules:** api.find.1-4, tool.card.1-3

## Context

The full language card is about 6,800 tokens and dominates an agent's fixed cost per task. An agent
also recalls standard-library names from memory, and a wrong guess costs a diagnostic round.

## Options considered

### Option A: a core card of at most 2,000 tokens, `ail card [TOPIC]` and `ail ask std` (chosen)
- **Pros:** the fixed cost drops to a third; the rest is one call away; no new data to keep in step,
  because the lookup indexes the std rules the spec already holds.
- **Cons:** a smaller card may cost more rounds than it saves tokens. So it ships as a benchmark arm
  (`ail-core`, E3) first, and becomes the default only if the correct count does not drop.

### Option B: shrink the full card in place
- **Pros:** one card.
- **Cons:** every agent loses the long form; no way to measure the trade.

### Option C: a hosted lookup
- **Cons:** needs a network and a service (ADR 0003: GitHub and wiki only).

## Decision

- `docs/language-card-core.md`: syntax, the ten rules the mistakes corpus hits most, and how to look up the rest.
- `ail card [TOPIC]` prints the core card, a section of the full card, or the signatures of `std.MODULE` (tool.card.1-3).
- `ail ask std WORDS...` and `ail ask std --sig "(PARAMS) RET"` rank std functions from the spec's `std.*` rules,
  at most ten lines (api.find.1-4). Implemented in `compiler/api/find.ail` and `compiler/api/cardcmd.ail`.
- The cards and the spec are read from the repository, `AIL_ROOT` or the parent of `AIL_COMPILER`, as `ail explain`
  does. The plan asked for text built into the binary; the CLI is always built from a checkout and has no embedding
  mechanism, so that is deferred until `ail explain` gets one (the same change would serve both).

## Rationale

Tokens rank first (ADR 0076): 2,000 against 6,800 per trial, measured by tokbench. Indexing the spec means the lookup
cannot drift from the documented signatures.

## Consequences

A std function whose rule does not open with a backticked signature is not found by `ail ask std`. The audit that every std
function is reachable by a word of its doc is the `spec` suite's job once the rules are all in that form.

## Rejected alternatives

- **Shrink the full card in place:** no measurement possible, and it removes the long form.
- **A hosted lookup:** needs a service (ADR 0003).
- **A separate std index file:** one more thing to keep in step with the spec.
