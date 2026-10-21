# ADR 0812: `ail fix` writes `c ? a : b` as `if c then a else b`

- Status: Accepted
- Decided by: AI (delegated, ADR 0076)
- Date: 2026-10-21

## Context

gram.foreign.1 reports the C-style conditional operator and names the AIL
form, the one-line `if c then a else b`. The rewrite needs no choice, but the
diagnostic carried no fix.

## Decision

When the whole right-hand side of a binding, an assignment or a `return` is
`c ? a : b`, the fix inserts `if ` before `c`, and turns `?` into `then` and
the `:` into `else`. The edits touch three tokens, so the operands are never
re-typed. No fix when the conditional is an operand or an argument (it would
need parentheses and the extent of `c` is not known without the grammar), is
nested, or has no single top-level `:`.

## Rejected alternatives

- Parenthesising `(if c then a else b)` wherever it appears: finding the start
  of `c` inside a larger expression means guessing precedence.
- Rewriting to a multi-line `if`: longer, and the one-line form is in the card.
