# 0470. A one-line `if` is an operand in parentheses

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 5 — Expressions (`if`)
- **Spec rules:** `expr.if.8` (amended: an `if` operand is parenthesised)
- **Resolves:** OQ-117
- **Criterion:** AI understanding (ADR 0076, criterion 2); the bounds are plain

## Context

`col + if raw then 4 else 3` and `col + (if raw then 4 else 3)` were both
syntax errors (`gram.syntax.1`), so the one-line `if` only appeared as a whole
expression. The self-hosted lexer wrote two whole branches to avoid it.

## Options considered

### A (chosen): allow the one-line `if` in parentheses
- **Pros:** the bounds are plain: the `if` ends at `)`; the formatter keeps the
  parentheses (it works on tokens, `fmt.rs`); no new precedence rule.
- **Cons:** one more form that parentheses have to carry.

### B: allow it anywhere an operand is
- **Pros:** shortest.
- **Cons:** `a + if c then b else d + e` hides where the `else` branch ends.

### C: keep it
- **Pros:** no change.
- **Cons:** two whole branches where one expression reads better.

## Decision

Option A. The parser (`bootstrap/src/parser.rs`, `primary`) accepts `if` after
`(`, and `compiler/syntax/parse.ail` does the same. A bare `if` operand is
`expr.if.8` with the help "write `(if c then a else b)`", where it was
`gram.syntax.1`. The block form of `if` is not allowed inside parentheses,
since a block needs a line break.

## Measurements

The form adds no new tokens to existing code: it only makes a program that was
an error legal. No tokbench table is needed. The lexer's two-branch workaround
is noted in [self-hosting status](../design/self-hosting-status.md) as fixed.

## Consequences

- OQ-117 is closed.
- Tests: `bootstrap/tests/cases/if_operand.ail` (positive) and
  `if_operand_errors.ail` (`expr.if.8`).
- Formatter: no change needed.
