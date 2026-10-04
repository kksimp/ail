# 0024. The AI semantic API starts in Stage 2 and grows with each stage

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 20 — AI semantic API; 19 — AST/IR architecture; roadmap
- **Spec rules:** none
- **Resolves:** roadmap refinement #1

## Context

The founding brief placed the full AI semantic interface at Stage 8. The
brief also makes the AST a first-class interface. In addition, ADR 0015
requires a dependency-tracked, query-based compiler, which is the same
infrastructure a semantic API needs.

## Options considered

### Option A — Start in Stage 2 (chosen)
- **Pros:** The compiler's internals are designed around queries from day
  one. AI agents can use AIL's own API while building AIL. The API is
  battle-tested by 1.0.
- **Cons:** The API evolves while the language changes, so it needs
  versioning from the start.

### Option B — Keep at Stage 8
- **Pros:** Designed once, against a stable compiler.
- **Cons:** Risks a costly retrofit, and gives AI agents no structured help
  for most of the project.

## Decision

The semantic API ships with the Stage 2 front end (symbols, types, references,
structured diagnostics) and gains capabilities at each later stage
(allocation facts, optimisation decisions, performance data, debug data).

## Consequences

- The API is **versioned** from its first release, and its schema is
  documented in `docs/`.
- Stage 8 becomes "complete the semantic API" rather than "create it".
- The compiler architecture topic must design the internal query system and
  the external API together.

## Rejected alternatives

- **Stage 8:** retrofit risk, and AI help comes too late.
