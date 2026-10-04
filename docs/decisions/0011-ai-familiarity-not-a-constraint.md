# 0011. Existing AI training-data familiarity is not a design constraint; open-source intent

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 2 — Core syntax; project strategy
- **Spec rules:** none

## Context

The [feasibility note](../design/feasibility.md) warned that current AI models
were trained on no AIL code, and suggested staying close to familiar syntax to
improve AI fluency. The designer intends to open-source AIL once it is
compelling enough, expecting AI systems to learn it through adoption.

## Decision

1. Resembling existing languages, purely so that today's models generate AIL
   more fluently, is **not** a design constraint. AIL's syntax should be what
   is best from first principles.
2. AIL is intended to become **open source** once it is compelling enough.
   The repository is already Apache-2.0 licensed.

## Rationale

Designing for today's training data would bake in the habits of existing
languages, which is exactly what the project is trying to move past. A
compelling open language will be learned.

## Consequences

- Syntax proposals are judged on clarity, regularity, token cost and
  readability. "It looks like Rust or Go" is not a reason to choose it.
- **Tokenizer cost remains a measured constraint.** Tokenizers are a separate
  issue from training familiarity: a symbol that splits into several tokens
  costs tokens no matter how familiar the language becomes. Syntax choices are
  still benchmarked against real tokenizers (OQ-007).
- Short-term mitigations stay useful and are not constraints: a compact
  language card, precise diagnostics, and the semantic API.

## Rejected alternatives

- **Mimic familiar syntax for AI fluency:** trades long-term design quality
  for a short-term advantage that disappears as AIL is adopted.
