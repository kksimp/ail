# 1127. `x = e else` accepts an indented block

- **Status:** Accepted
- **Date:** 2026-11-15
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); approved by Kaleb 2026-11-15
- **Topic:** Errors and optionals; AI understanding
- **Spec rules:** err.else.8
- **Extends:** [ADR 0049](0049-else-for-fallback-and-early-exit.md), [ADR 0139](0139-unit-results-statement-else-and-divergence.md)

## Context

An `else` followed by a block needed `:=`; after `=` it was a compile error
(lex.stmt.1) that told the agent to bind a new name and assign it. In the E13
agent benchmark, agents wrote `n = s.to_int() else` and a block, over an
existing `mut` name, in first drafts. Each one cost a fix iteration, and the
iteration count is the primary E14 metric.

## Options considered

### Option A: accept the block after `=` and `op=` (chosen)
The parser reads `target = e else BLOCK` as `tmp := e else BLOCK` followed by
`target = tmp`, with `tmp` a name no program can write. The block rules are
those of `:=` (err.else.2, err.else.3), and the checker needs no change.
- **Pros:** one rule for the block, no new checker path; works for fields and
  index targets; diagnostics are the existing ones.
- **Cons:** `e` is evaluated before the target's own parts (index or field
  expressions), which matters only when those have side effects.

### Option B: keep the error and improve the help
- **Pros:** one way to write it.
- **Cons:** the agent still pays an iteration for a form that has one obvious meaning.

## Consequences

- Spec err.else.8, cards updated, test `tests/cases/else_assign_block`, mistake
  `assign_else_block_falls` (a block that falls through is err.else.2).
- A block that completes without a value is still an error.
