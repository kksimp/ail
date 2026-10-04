# 02. Lexical structure

Chapter key: `lex`

> **Status:** Drafted (partial). Specified: block structure, statement
> termination, comments, keywords, and string, character and numeric
> literals. Not yet designed: the character set and identifiers.

## Keywords

**lex.kw.1** — These words are reserved keywords and MUST NOT be used as
names: `and any as break continue defer elif else enum extern false fn for if
impl in match move mut none not or pub ref return self spawn struct trait true
unique unsafe use while`. *(ADR 0114)*

**lex.kw.2** — `c` is not reserved. It has its FFI meaning only directly
after `extern` or `use` (`ffi.decl.1–2`). *(ADR 0114)*

## Blocks

**lex.block.1** — Blocks are delimited by indentation. A line indented one
unit deeper than the line that opens a block begins that block. The block ends
at the first subsequent line indented less deeply. *(ADR 0033)*

**lex.block.2** — The indentation unit is exactly four space characters
(U+0020). Every line's indentation MUST be a whole number of units.
*(ADR 0033, ADR 0040)*

**lex.block.3** — Indentation that is not a whole multiple of the unit, or
that increases by more than one unit at a time, is a compile-time error.
*(ADR 0033)*

**lex.block.4** — A tab character (U+0009) MUST NOT appear in a line's
indentation. *(ADR 0040)*

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

## Comments

**lex.comment.1** — Outside string and character literals, `#` begins a
comment that extends to the end of the line. *(ADR 0043)*

**lex.comment.2** — A comment beginning with `##` is a documentation comment.
It documents the declaration that immediately follows it. *(ADR 0043)*

**lex.comment.3** — There are no block (multi-line delimited) comments.
*(ADR 0043)*

```ail
## Returns the area of a circle.
fn area(r f64) f64
    # pi is a library constant
    pi * r * r
```

## String and character literals

**lex.str.1** — A string literal is enclosed in double quotes. The escape
sequences are `\n`, `\t`, `\r`, `\0`, `\\`, `\"`, `\{` and `\u{hex}`. Any
other backslash sequence is a compile-time error. *(ADR 0089)*

**lex.str.2** — Within a string literal, `{expression}` interpolates the text
form of the expression. `\{` produces a literal `{`. *(ADR 0089)*

**lex.str.3** — A literal beginning with `"""` is a multi-line string. It ends
at the next `"""`. Its lines' common leading indentation is removed.
*(ADR 0089)*

**lex.str.4** — A character literal is a single Unicode scalar value, or one
escape sequence, in single quotes: `'a'`, `'\n'`. *(ADR 0089)*

**lex.str.5** — String literals are UTF-8 encoded. *(ADR 0089)*

```ail
msg := "Hello, {name}! You have {count} new messages."
```

## Numeric literals

**lex.num.1** — Integer literals are decimal, hexadecimal (`0x`), binary (`0b`)
or octal (`0o`). An underscore may separate digits. *(ADR 0094)*

**lex.num.2** — A floating-point literal contains a `.` with at least one digit
on each side, an exponent (`e` or `E`, optionally signed), or both.
*(ADR 0094)*

**lex.num.3** — A numeric literal whose value doesn't fit its type is a
compile-time error. *(ADR 0094)*
