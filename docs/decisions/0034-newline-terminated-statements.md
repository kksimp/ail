# 0034. Newlines end statements; there are no semicolons

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 2 — Core syntax; 3 — Grammar
- **Spec rules:** [lex.stmt.1–3](../spec/02-lexical.md)
- **Evidence:** [design/syntax-measurements-r1.md](../design/syntax-measurements-r1.md)

## Context

Semicolons cost between ~0% (OpenAI tokenizers, where `;` merges with the
newline) and ~5% (other tokenizers) on the measured program. With
indentation-delimited blocks (ADR 0033), line structure already carries meaning.

## Options considered

### Option A — Newline ends a statement (chosen)
- **Pros:** The least clutter, fewer tokens, and one canonical form.
- **Cons:** Line-continuation rules must be precise.

### Option B — Semicolons required
- **Pros:** Explicit, and line breaks never matter.
- **Cons:** Noise, up to ~5% more tokens, and trivial "missing `;`" errors.

### Option C — Optional semicolons
- **Pros:** Flexible.
- **Cons:** Two ways to write the same thing, and ASI-style edge cases (as in
  JavaScript).

## Decision

A newline ends a statement. A statement continues onto the next line only when
the line clearly cannot end there: inside unclosed `()`, `[]` or `{}`, or when
the line ends with a binary operator or a comma. AIL has no statement
terminator token.

## Consequences

- The exact continuation rules belong in the grammar chapter and must be
  unambiguous. The formatter decides where long expressions break.
- There's no "missing semicolon" error class.

## Rejected alternatives

- **Required semicolons:** noise and errors for no semantic gain.
- **Optional semicolons:** violates canonical syntax.
