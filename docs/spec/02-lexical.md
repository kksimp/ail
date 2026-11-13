# 02. Lexical structure

Chapter key: `lex`

> **Status:** Drafted (partial). Specified: block structure, statement
> termination, comments, keywords, and string (including multi-line and raw),
> character and numeric literals, and the characters of a name. Not yet
> designed: the character set and name normalisation.

## Keywords

**lex.kw.1** — These words are reserved keywords and MUST NOT be used as
names: `and any as break continue defer elif else ensures enum error extern
false fn for if impl in invariant match move mut none not or par pub ref requires
return self spawn struct todo trait true uhwhatnow unique unsafe use while with
yield`.
`uhwhatnow?`, with no space before the `?`, is one keyword token, the hole
that asks (`expr.hole.3`); where `uhwhatnow` alone stands as an expression,
the error carries a fix that adds the `?`. In the positions of `lex.kw.3` a
keyword is a name. The error's help names a replacement; for `error`, `todo`,
`any`, `ref`, `enum`, `struct`, `use` and `pub` its fix renames the binding and
each later use of the word in the function. *(ADR 1085, ADR 1067, ADR 0114, ADR 0120, ADR 0147, ADR 0210, ADR 0237, ADR 0301, ADR 0305, ADR 0872, ADR 1073)*

```ail error=lex.kw.1
fn main()
    fn := 1
    print(fn)
```

```ail
fn main()
    count := 1
    print(count)
```

**lex.kw.2** — `c` is not reserved. It has its FFI meaning only directly
after `extern` or `use` (`ffi.decl.1–2`). *(ADR 0114)*

```ail rules=lex.kw.2
fn main()
    c := 1
    print(c)
```

**lex.kw.3** — A keyword is a plain name where nothing else can stand: as a
field of a struct or of an enum variant (except `pub`, `fn` and `invariant`,
which begin a member line), after `.` (`o.error`, `xs.any(f)`), as a label
before `:` in a call or literal (`Outcome(error: "")`), as a method name
(`fn T.any(self)`, and `fn error(self)` inside a `trait` or `impl` block), and
after `uses` (`trait.hash.1`). Inside a method a keyword-named field is
reached as `self.error`; the bare word is the keyword. *(ADR 1085)*

```ail rules=lex.kw.3
struct Outcome
    job str
    error str

fn Outcome.any(self) bool
    self.error != ""

fn main()
    o := Outcome(job: "sync", error: "disk full")
    print("{o.job}: {o.error} {o.any()}")
```

## Names

**lex.name.1** — A name starts with a letter (the Unicode `Alphabetic`
property) or `_`, and continues with letters, ASCII digits `0`–`9` and `_`:
the characters for which `char.is_alpha()` and `char.is_alnum()` are true
(`type.char.2`). A name that is a keyword is the keyword (`lex.kw.1`), except
in the positions of `lex.kw.3`.
*(ADR 0372)*

```ail rules=lex.name.1
fn main()
    ça := 1
    print(ça)
```

## Blocks

**lex.block.1** — Blocks are delimited by indentation. A line indented one
unit deeper than the line that opens a block begins that block. The block ends
at the first subsequent line indented less deeply. A line that lies inside a
string literal (`lex.str.3`, `lex.str.6`) is part of that literal, not block
structure: it doesn't open or close a block, and `lex.block.2–4` don't apply
to its indentation (`lex.str.3` rule 4 governs it instead). *(ADR 0033,
ADR 0135)*

```ail rules=lex.block.1
fn main()
    if true
        print(1)
```

**lex.block.2** — The indentation unit is exactly four space characters
(U+0020). Every line's indentation MUST be a whole number of units.
*(ADR 0033, ADR 0040)*

```ail error=lex.block.2
fn main()
  print(1)
```

```ail
fn main()
    print(1)
```

**lex.block.3** — Indentation that is not a whole multiple of the unit, or
that increases by more than one unit at a time, is a compile-time error.
*(ADR 0033)*

```ail error=lex.block.3
fn main()
    if true
            print(1)
```

```ail
fn main()
    if true
        print(1)
```

**lex.block.4** — A tab character (U+0009) MUST NOT appear in a line's
indentation. *(ADR 0040)*

> **Note:** blank lines and comment-only lines do not affect block structure.
> The exact rule will be given with the comment syntax.

```ail error=lex.block.4
fn main()
	print(1)
```

```ail
fn main()
    print(1)
```

## Statements

**lex.stmt.1** — A newline ends a statement, except as provided by
`lex.stmt.2`. *(ADR 0034)*

```ail rules=lex.stmt.1
fn main()
    x := 1
    print(x)
```

**lex.stmt.2** — A newline does not end a statement when it occurs:
1. inside an unclosed `(`, `[` or `{`,
2. immediately after a binary operator or a comma, or
3. inside a multi-line string literal that is not yet closed (`lex.str.3`), or
4. when the next line's first token is `.` followed by a name (a chained call,
   ADR 0323), or
5. when the next line's first token is `and`, `or`, `+`, `*`, `/`, `%`, `==`,
   `!=`, `<`, `<=`, `>` or `>=` (a long expression split before an operator,
   ADR 0346). `-` is left out because it can begin a statement.

The next line is then a continuation of the same statement. Continuation lines
do not open or close blocks. *(ADR 0034, ADR 0135)*

```ail error=lex.stmt.2
fn main()
    print(f(1) 
```

```ail
fn main()
    x := 1 +
        2
    print(x)
```

**lex.stmt.3** — AIL has no statement-terminator token. *(ADR 0034)*

> **Rationale:** semicolons cost up to ~5% more tokens with no semantic
> benefit once lines and indentation carry structure. See
> [syntax-measurements-r1.md](../design/syntax-measurements-r1.md).

```ail error=lex.stmt.3
fn main()
    x := 1;
    print(x)
```

```ail
fn main()
    x := 1
    print(x)
```

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

**lex.bytes.1** — A `bytes` literal is `b"..."`. Its text is ASCII, with the
escapes `\n \t \r \0 \\ \"` and `\xHH` (two hex digits, either case) for any
byte. It has no interpolation, so `{` is a literal character, and it ends at
the next unescaped `"`, on the same line. A non-ASCII character, an unknown
escape or a missing closing `"` is an error. `b` MUST be immediately followed
by the opening `"`; `b "x"` is the name `b` followed by a string. The literal's
type is `bytes` (`type.bytes.1`). *(ADR 0413)*

```ail fragment
magic := b"\x89PNG\r\n\x1a\n"        # eight bytes
req := b"GET / HTTP/1.1\r\n\r\n"
```

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
character literals. The specification is
`[[fill]align][+][0][width][,][.decimals][x|b|e]` and nothing else:

- `align` is `<` (left), `>` (right) or `^` (centre); `fill` is any one
  character before it. The default fill is a space. Without `align`, numbers
  are right-aligned and everything else is left-aligned.
- `width` is a decimal number. A `0` before it (`{n:08}`) pads a number with
  zeros after its sign and can't combine with `align`; to fill text or to
  align, write the fill (`0>8`). Text longer than `width` isn't cut. Width
  counts characters.
- `+` writes a `+` before a number that isn't negative. `,` puts a `,` between
  each three digits of a number's integer part.
- `.decimals`, `x`, `b` and `e` are typed by `type.str.11`.
- `width` and `decimals` are at most 1000. `x` and `b` don't combine with
  `.decimals`, and `,` doesn't combine with `x`, `b` or `e`.

Anything else is a compile-time error whose help shows the grammar. There is
no `{x!r}` or `#` form. *(ADR 0173, ADR 1087)*

```ail fragment
n := 255
r := 3.14159
print("[{n:>6}][{r:.2}][{n:0>4x}][{"ab":^6}][{"ab":*<5}][{n:08}][{n:b}][{n:+}]")
```

```ail error=lex.str.7
n := 1
print("{n:#x}")      # compile-time error: there is no `#` form
```

**lex.str.8** — A `{` that directly follows a `$` in a string literal is a
compile-time error: `"${n}"` is another language's interpolation and AIL
would print `$` and then the value. The help says to write `{n}`, and the fix
removes the `$`. The literal text `${` is written `$\{`.

```ail error=lex.str.8
fn main()
    n := 1
    print("n is ${n}")   # compile-time error: write "n is {n}"
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

## Runnable examples for the rules above

Each rule ID has a checked example (ADR 0498). Blocks marked `error=` must
report their rule; blocks marked `rules=` must check and run with the printed
output shown in the comments.

```ail error=lex.kw.1
fn main()
    while := 1      # reserved: `while` can't be a name
```

```ail rules=lex.kw.2
fn main()
    c := 2          # `c` is not reserved outside `extern` and `use`
    print(c)        # 2
```

```ail rules=lex.kw.3
struct Reply
    in str          # a keyword names a field
    match bool

fn main()
    r := Reply(in: "ping", match: true)
    print(r.in)     # ping
    print(r.match)  # true
```

```ail rules=lex.name.1
fn main()
    é := 1          # a letter outside ASCII
    _x := 2         # `_` may start a name
    print(é + _x)   # 3
```

```ail rules=lex.block.1
fn main()
    if true
        print("inside")   # one unit deeper opens a block
    print("outside")      # less deep closes it
```

```ail error=lex.block.2
fn main()
   print(1)         # three spaces: the unit is exactly four
```

```ail error=lex.block.3
fn main()
        print(1)    # eight spaces from the line that opens the block: two units
```

```ail error=lex.block.4
fn main()
	print(1)        # a tab in the indentation
```

```ail rules=lex.stmt.1,lex.stmt.3
fn main()
    a := 1          # a newline ends the statement
    print(a)        # 1; there is no `;` token
```

```ail rules=lex.stmt.2
fn main()
    print(1 +       # a newline after a binary operator continues the statement
        2)
    print([1,       # ... after a comma, and inside brackets
        2])         # 3 then [1, 2]
```

```ail rules=lex.comment.1
fn main()
    a := 1  # a comment runs to the end of the line
    print(a)        # 1
```

```ail rules=lex.comment.2
## Adds two numbers.
fn add(a int, b int) int
    a + b

fn main()
    print(add(2, 3))    # 5
```

```ail rules=lex.comment.3
fn main()
    # a comment with /* markers */ is still one line; there are no block comments
    print(1)            # 1
```

```ail error=lex.str.1
fn main()
    print("\q")          # `\q` is not an escape sequence
```

```ail rules=lex.str.3
fn main()
    s := """
    one
    two
    """
    print(s)            # one, then two
```

```ail rules=lex.str.4
fn main()
    print('a')          # a single character
    print('\n')         # an escape: a line break
```

```ail rules=lex.str.5
fn main()
    print("héllo")      # UTF-8 text, prints héllo
```

```ail rules=lex.str.6
fn main()
    print(r"a\b")       # raw: the backslash is literal; prints a\b
```

```ail rules=lex.bytes.1
fn main()
    print(b"ab\x41")    # b"abA"
```

```ail error=lex.bytes.1
fn main()
    print(b"café")      # bytes literals are ASCII
```

```ail rules=lex.num.1
fn main()
    print(0x1F + 0b101 + 0o17 + 1_000)   # 31 + 5 + 15 + 1000 = 1051
```

```ail rules=lex.num.2
fn main()
    print(1e3)          # 1000.0
    print(2.5)          # 2.5
```
