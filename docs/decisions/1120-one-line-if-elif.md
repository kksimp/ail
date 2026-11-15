# 1120. The one-line `if` takes `elif` arms

- **Status:** Accepted
- **Date:** 2026-11-15
- **Decided by:** Designer (Kaleb approved the direction on 2026-11-15, E13 follow-up slice S9); details by AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** Core syntax
- **Spec rules:** expr.if.8
- **Amends:** [ADR 0280](0280-one-line-if-and-round-5-measurements.md) ("It has no `elif`")

## Context

ADR 0280 added `if c then a else b` with exactly one condition. In the E13 agent
benchmark, 19 AIL trials wrote a three-way choice on one line as
`if c then a elif d then b else e`, the shape the block form (`expr.if.1`) has.
The parser stopped at `elif` with "`if c then a` needs `else b` on the same line",
which named the wrong problem: the `else` was there, after an arm the form did
not accept. Each such trial spent a compile round on it.

## Options considered

- **Accept `elif d then e` arms before the final `else` (chosen).** The one-line
  form then has the arms of the block form, in the same order and with the same
  keywords, so knowing one form is knowing the other. No new keyword or AST node:
  the arms become the branches the block form builds.
- **Keep one condition, improve the error** to say "nest with `else if` or use
  the block form". Rejected: the message would teach a restriction that exists in
  no other `if`, and agents would still write the shape first.
- **Make one-line `else if` an error with a fix to `elif`** (as in the block form).
  Rejected for now: `else if c then a else b` is a valid one-line `if` used as the
  `else` value, means the same thing, and appears in the compiler's own source,
  which the published seed must keep building. It stays legal. A lint that
  rewrites it to `elif` can come once a seed accepts one-line `elif`.

## Decision

- `if c then a elif d then e ... else b` is one expression on one line. Any
  number of `elif` arms may come before the `else`; each arm is
  `elif COND then EXPR`, and each condition may be a binding (`expr.if.5`).
- A one-line `if` without its final `else` is an error that shows the shape
  written so far and the missing `else`; an `elif` without `then` is an error that
  says so.
- `then` stays a contextual keyword, only right after a condition.
- Grammar: `IfExpr = 'if' Cond ( 'then' ExprPlain { 'elif' Cond 'then' ExprPlain } 'else' ExprPlain | ... )`.
- The formatter works on tokens and keeps the one-line form as written while
  it fits in 100 columns; nothing changes there.

## Consequences

- The compiler's own source must not use one-line `elif` until a seed carries it.
- Tokens: a three-way choice costs one line instead of six (block form) or a
  nested `else if`, which costs the same tokens. Within the ADR 0076 tie band.
