# 02. Lexical structure

Chapter key: `lex`

> **Status:** Drafted (partial). Specified: block structure, statement
> termination, comments, keywords, and string (including multi-line and raw),
> character and numeric literals. Not yet designed: the character set and
> identifiers.

## Keywords

**lex.kw.1** — These words are reserved keywords and MUST NOT be used as
names: `and any as break continue defer elif else ensures enum error extern
false fn for if impl in invariant match move mut none not or par pub ref requires
return self spawn struct todo trait true unique unsafe use while`.
*(ADR 0114, ADR 0120, ADR 0147, ADR 0210, ADR 0237, ADR 0305)*

**lex.kw.2** — `c` is not reserved. It has its FFI meaning only directly
after `extern` or `use` (`ffi.decl.1–2`). *(ADR 0114)*

## Blocks

**lex.block.1** — Blocks are delimited by indentation. A line indented one
unit deeper than the line that opens a block begins that block. The block ends
at the first subsequent line indented less deeply. A line that lies inside a
string literal (`lex.str.3`, `lex.str.6`) is part of that literal, not block
structure: it doesn't open or close a block, and `lex.block.2–4` don't apply
to its indentation (`lex.str.3` rule 4 governs it instead). *(ADR 0033,
ADR 0135)*

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
1. inside an unclosed `(`, `[` or `{`,
2. immediately after a binary operator or a comma, or
3. inside a multi-line string literal that is not yet closed (`lex.str.3`), or
4. when the next line's first token is `.` followed by a name (a chained call,
   ADR 0323).

The next line is then a continuation of the same statement. Continuation lines
do not open or close blocks. *(ADR 0034, ADR 0135)*

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

```ail skip=context
## Returns the area of a circle.
fn area(r f64) f64
    # pi is a library constant
    pi * r * r
```

## String and character literals

**lex.str.1** — A string literal is enclosed in double quotes. The escape
sequences are `\n`, `\t`, `\r`, `\0`, `\\`, `\"`, `\'`, `\{` and `\u{hex}`. Any
other backslash sequence is a compile-time error. The diagnostic MUST list the
valid escapes and, for `\x41`, `\b`, `\f`, `\a` and `\e`, point to `\u{..}`.
*(ADR 0089, ADR 0135)*

**lex.str.2** — Within a string literal, `{expression}` interpolates the text
form of the expression. `\{` produces a literal `{`. An interpolation MUST NOT
begin with `{`: `{{` is a compile-time error whose help says a literal brace
is `\{`. *(ADR 0089, ADR 0135)*

```ail error=lex.str.2
msg := "Hello, {name}! You have {count} new messages."
a := "\{x}"          # the text {x}
b := "{{x}}"         # compile-time error: a literal brace is \{
```

**lex.str.3** — A literal beginning with `"""` is a multi-line string. It
follows these rules, in this order. *(ADR 0089, ADR 0135)*
1. **One-line form.** If a closing `"""` follows the opening on the same line,
   the literal is a one-line string, with no dedent. It is for text that
   contains `"`. The closing delimiter is the last three quotes of a run of
   quotes, so `"""say "hi""""` is `say "hi"`. Quotes right after the opening
   `"""` belong to the text.
2. **Opening line.** Otherwise the opening `"""` MUST end its line (only
   spaces may follow it). That newline isn't part of the text.
3. **Closing line.** The closing line is the line that holds the closing
   `"""`. If only spaces precede the `"""` on it, those spaces count toward the
   common indentation, and the text ends at the end of the previous line: the
   newline before the closing line is not part of the text. Code may follow
   the closing delimiter (`"""),`). If text precedes the `"""`, that line is
   the last text line.
4. **Dedent.** The common indentation is the fewest leading spaces among the
   non-blank text lines and the closing line. It is removed from every line.
   A line of only spaces becomes empty. Only spaces indent: a tab in a text
   line's leading whitespace is a compile-time error.
5. **Escapes and interpolation.** Dedent applies to the source text, before
   escapes and interpolation. Those then work as in `lex.str.1–2`, except
   that an interpolation MUST NOT span lines. Source line endings (CRLF)
   become `\n`.
6. **Statements and blocks.** A newline inside the literal doesn't end the
   statement (`lex.stmt.2`), and the literal's lines aren't block structure
   (`lex.block.1`).

```ail skip=context
one := """say "hi" to {name}"""     # one line: say "hi" to Ada
two := """
    a
    b
    """                             # "a\nb": no newline before the closing line
three := """
    a

    """                             # "a\n": an empty line ends the text with \n
four := """
    a
      b"""                          # "a\n  b": text before the closing delimiter
print("""
    usage: ail run FILE
    """)                            # prints "usage: ail run FILE"
```

> **Rationale:** the newline before the closing line is dropped (Swift's
> rule), so `"""` text split on `\n` doesn't end with a stray empty piece.
> Keeping the final newline (Java, Python) was rejected for that reason; to
> end with a newline, leave an empty line before the closing line.
> Making `"""` strings multi-line only was rejected: text that contains `"`
> needs the one-line form (rule 1).

**lex.str.4** — A character literal is a single Unicode scalar value, or one
escape sequence, in single quotes: `'a'`, `'\n'`. Its escape sequences are
those of `lex.str.1`, including `\'`. `'''` (three single quotes) is a
compile-time error whose help is `'\''`. A character literal never
interpolates: `'{'` is the character `{`. *(ADR 0089, ADR 0134, ADR 0135)*

**lex.str.5** — String literals are UTF-8 encoded. *(ADR 0089)*

**lex.str.6** — A raw string literal is `r"..."` or `r"""..."""`. It has no
escapes and no interpolation: `\` and `{` are literal characters.
*(ADR 0135)*
1. `r"..."` ends at the next `"`, so it can't contain `"` or a newline.
2. `r"""..."""` follows `lex.str.3`: multi-line, or one line by its rule 1.
3. `r` MUST be immediately followed by the opening `"`. `r "x"` is the name
   `r` followed by a string, since `r` isn't a keyword (`lex.kw.1`).

```ail fragment
pat := r"\d+\.\d+"                  # the text \d+\.\d+
src := r"""
    fn main()
        print("{x}")
    """                              # the text fn main() ... print("{x}")
```

> **Rationale:** raw strings suit embedded source text and regex-like text.
> A plain string is cheaper for a short escape (a raw form costs about 7%
> more tokens there, measured), so raw strings don't replace plain ones.

**lex.str.7** — An interpolation may carry a format specification:
`{expression:spec}`. The `:` is the first one outside brackets, strings and
character literals. The specification is `[[fill]align][width][.decimals][x]`
and nothing else:

- `align` is `<` (left), `>` (right) or `^` (centre); `fill` is any one
  character before it. The default fill is a space. Without `align`, numbers
  are right-aligned and everything else is left-aligned.
- `width` is a decimal number with no leading `0` (write `0>4`); text longer
  than `width` isn't cut. Width counts characters.
- `.decimals` and `x` are typed by `type.str.11`.
- `width` and `decimals` are at most 1000. `x` and `.decimals` don't combine.

Anything else is a compile-time error whose help shows the grammar. There is
no `{x!r}`, sign, `#`, `,` or `e` form. *(ADR 0173)*

```ail fragment
n := 255
r := 3.14159
print("[{n:>6}][{r:.2}][{n:0>4x}][{"ab":^6}][{"ab":*<5}]")
```

```ail error=lex.str.7
n := 1
print("{n:03}")      # compile-time error: write {n:0>3}
```

## Numeric literals

**lex.num.1** — Integer literals are decimal, hexadecimal (`0x`), binary (`0b`)
or octal (`0o`). An underscore may separate digits. *(ADR 0094)*

**lex.num.2** — A floating-point literal contains a `.` with at least one digit
on each side, an exponent (`e` or `E`, optionally signed), or both.
*(ADR 0094)*

**lex.num.3** — A numeric literal whose value doesn't fit its type is a
compile-time error. A unary `-` directly before an integer literal is part of
the literal for this check, so `-9223372036854775808` is a valid `int` (in
expressions, patterns and constants), unless a postfix operator binds the
literal first (`-9223372036854775808.abs()` is an error). A floating-point literal is correctly
rounded (`type.num.8`); one whose value overflows to infinity is an error.
*(ADR 0094, ADR 0130, ADR 0138)*

```ail error=lex.num.3
lo := -9223372036854775808     # valid: the minimum int
hi := 9223372036854775808      # compile-time error: doesn't fit int
big := 1e999                   # compile-time error: overflows float
```
