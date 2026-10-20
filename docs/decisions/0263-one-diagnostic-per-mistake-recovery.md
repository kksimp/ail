# 0263. One diagnostic per mistake: line-level recovery and whole-file re-indentation

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** AI semantic API (diagnostics)
- **Spec rules:** `gram.rec.1`, `gram.rec.2`, `gram.rec.3` ([03-grammar.md](../spec/03-grammar.md))

## Context

One slip often produced several reports: a lexer error followed by a parser error on the same line; a 2-space file reported once per line and then parsed as if every block were at the top level; an unclosed `(` followed by an error at end of file; an undeclared name reported again at each use. An agent that reads the first and fixes it wastes a round when the rest was noise, and one that reads all of them acts on false leads. Criteria: AI understanding (#2).

## Options considered

### Option A — recover at the cause (chosen)
One syntax diagnostic per line, absorbing the fixes of the others; the file's indentation unit detected once; the parser sees closed brackets; a failed declaration still declares the name.
- **Pros:** the first report is the cause; `ail fix` settles a line in one round.
- **Cons:** a second, independent slip on the same line shows only after the first is fixed (one more round).

### Option B — keep every report and let the agent sort them out
- **Pros:** nothing hidden.
- **Cons:** the corpus showed 9 of 109 programs with cascades; the second report was never the mistake.

## Decision

- **gram.rec.1:** among the lexer and parser errors, only the leftmost on a line is reported (string errors excepted). The edits of the fixes it absorbs join its fix when they don't overlap. A file whose statements are all indented by 2 or 3 spaces, or by tabs, gets one diagnostic whose fix re-indents every statement (the lexer re-lexes at the detected unit, so the parser sees real blocks). The unit is detected only when the first indented level equals the common divisor, so one stray 6-space line in a 4-space file is still reported alone.
- **Unclosed brackets:** the lexer closes them for the parser at the end of the file and offers the closing characters at the end of the line they opened on, when they all opened on one line.
- **gram.rec.2:** `x = 1` for an undeclared `x` declares `x` (with an error type) for the rest of the function.
- **gram.rec.3:** no suggestion, hence no fix, for names under 3 characters: `p.y` is one edit from `p.x`.

## Rationale

All four are about reporting the cause once. The short-name floor is a correctness fix: a "did you mean" fix that changes a program's meaning is worse than none.

## Consequences

- Syntax diagnostics in the corpus: 120 before, 111 after, with 109 programs; the two left with two reports each hold two separate mistakes.
- Existing cases changed: `rust_style_signature` (a colon and an arrow on one line) now reports one diagnostic.

## Rejected alternatives

- **Round each odd indentation down:** turns a 2-space file into a flat one.
- **Stop at the first error:** hides independent errors in other statements.
- **Add token end positions to the syntax tree** to make every edit exact: touches every shared parser and checker file that ten lanes are editing tonight (see [0264](0264-anchored-call-rewrite-fixes.md)).
