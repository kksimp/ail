# 1065. Aim for speed equal to C and AI-first features ahead of the market; accept more tokens

- **Status:** Accepted
- **Date:** 2026-11-11
- **Decided by:** Designer (Kaleb), 2026-11-11. Written up by AI.
- **Topic:** Project goals ([vision.md](../vision.md))
- **Spec rules:** none (goals)
- **Amends:** [vision.md](../vision.md) goals 1, 3 and 4

## Context

The vision asked for "maximum semantic density per token" and made token
efficiency an explicit objective. The project goal in practice was to beat C
and Python on speed and on tokens. The 0.1.0 benchmarks show AIL at 0.5x to 1.8x
`gcc -O2` time on small programs, and at about 1.25x Python's tokens on the same
programs. The designer revised the goal on 2026-11-11. In his words: more tokens
are acceptable as long as speed and the AI-first features are better than
anything on the market, and for speed he would settle for equal to C, because an
AI should be able to use this language more efficiently than others anyway.

## Options considered

### Option A: speed equal to C, AI-first features ahead of the market, tokens secondary (chosen)
- **Pros:** the targets can be met and checked. Effort goes to the compiler's answers, checks
  and tools, where agents gain the most.
- **Cons:** AIL gives up the claim of being the most token-dense language.

### Option B: keep "beat C and Python on speed and tokens"
- **Pros:** a sharper headline.
- **Cons:** the measured numbers do not support it, and reaching it would cost readability or
  AI understanding.

## Decision

1. The speed target is **equal to optimised C** (`gcc -O2`), measured and never just
   claimed. Being faster is welcome but is not the goal.
2. The AI-first target is tooling and language features for agents **better than any other
   language's**.
3. **Using more tokens than other languages is acceptable** when it buys speed or a better
   AI-first feature. Tokens are still measured with `tools/tokbench`.
4. Docs and release notes state only measured numbers. They make no claim of beating C and no
   claim of token efficiency.

## Consequences

- vision.md goals 1, 3 and 4, the README tagline and the 0.1.0 release highlights are
  reworded.
- ADR 0076 still lists token usage first for delegated decisions. Whether that order should
  change after this goal is an open question for the designer
  ([OQ-142](../open-questions.md)).

## Rejected alternatives

- **Keep the old goal:** rejected by the designer; the measurements do not support it.
- **Drop token measurement:** rejected. Tokens remain a cost to report, just not a headline.
