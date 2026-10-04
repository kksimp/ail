# 02. Lexical structure

Chapter key: `lex`

> **Status:** Drafted (partial). Only block structure and statement
> termination are specified. The character set, identifiers, keywords,
> literals, comments and operators are not yet designed.

## Blocks

**lex.block.1** — Blocks are delimited by indentation. A line indented one
unit deeper than the line that opens a block begins that block. The block ends
at the first subsequent line indented less deeply. *(ADR 0033)*

**lex.block.2** — A single fixed indentation unit is used for all source text.
*(ADR 0033; the unit is chosen in OQ-026)*

**lex.block.3** — Indentation that is not a whole multiple of the unit, or
that increases by more than one unit at a time, is a compile-time error.
*(ADR 0033)*

> **Note:** blank lines and comment-only lines do not affect block structure.
> The exact rule will be given with the comment syntax.

## Statements

**lex.stmt.1** — A newline ends a statement, except as provided by
`lex.stmt.2`. *(ADR 0034)*

**lex.stmt.2** — A newline does not end a statement when it occurs:
1. inside an unclosed `(`, `[` or `{`, or
2. immediately after a binary operator or a comma.

The next line is then a continuation of the same statement. Continuation lines
do not open or close blocks. *(ADR 0034)*

**lex.stmt.3** — AIL has no statement-terminator token. *(ADR 0034)*

> **Rationale:** semicolons cost up to ~5% more tokens with no semantic
> benefit once lines and indentation carry structure. See
> [syntax-measurements-r1.md](../design/syntax-measurements-r1.md).
