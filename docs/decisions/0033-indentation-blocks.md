# 0033. Blocks are delimited by indentation

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 2 — Core syntax; 3 — Grammar
- **Spec rules:** [lex.block.1–3](../spec/02-lexical.md)
- **Evidence:** [design/syntax-measurements-r1.md](../design/syntax-measurements-r1.md)

## Context

The brief's example used indentation. The architect raised edit robustness
as a concern ([ai-author-wishlist.md 3.2](../design/ai-author-wishlist.md)).
Measured with five tokenizers on the same program, indentation used about
9.5% fewer tokens than braces, and about 9% fewer than `end` keywords.

## Options considered

### Option A — Indentation (chosen)
- **Pros:** ~9.5% fewer tokens. Visually clean. The AST edit API lets
  AI tools avoid manual whitespace edits.
- **Cons:** A mis-indented line can silently change meaning (static types
  catch many cases, but not all). Whitespace can be lost when code is pasted
  through chat or web pages. Changing nesting requires re-indenting.

### Option B — Braces
- **Pros:** Whitespace-independent and robust for text tools.
- **Cons:** +9.5% tokens and visual noise.

### Option C — `end` keyword
- **Pros:** Whitespace-independent.
- **Cons:** +9% tokens, and `end end end` is hard to match by eye.

## Decision

Block structure is defined by indentation. A single fixed indentation unit is
used throughout (the unit is chosen in OQ-026). Inconsistent indentation is a
compile error.

## Consequences

- The lexer produces INDENT/DEDENT structure. The grammar is defined over it.
- **Mitigations for the known risks:**
  - The formatter is canonical (brief §5), so there is only one valid layout.
  - The compiler should warn when indentation and control flow look
    inconsistent, e.g. code after `return` at the same level, or a statement
    that probably belongs to the preceding block. Exact lints are to be
    designed.
  - The AST edit API (brief §14) performs nesting changes structurally.
  - `ail fmt` can't repair code whose indentation was destroyed. Tools that
    move code through chat or web pages must preserve whitespace.
- Measurements will be revisited with the full corpus (ADR 0029).

## Rejected alternatives

- **Braces:** more tokens, for a robustness benefit that the AST tooling and
  formatter mostly provide anyway.
- **`end`:** the same cost as braces and harder to read when nested.
