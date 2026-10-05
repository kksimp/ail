# 0135. String literals (multi-line, raw, `\'`) and the `str` methods

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 2 — Core syntax; 4 — Type system; 21 — Standard library
- **Spec rules:** [lex.str.1–3, lex.str.6, lex.stmt.2, lex.block.1](../spec/02-lexical.md), [type.str.2, type.str.6–10](../spec/04-types.md), [trait.auto.2](../spec/10-traits.md)
- **Amends:** [ADR 0089](0089-strings.md) (escapes, multi-line strings, "raw strings may be added later", `s.bytes`)
- **Extends:** [ADR 0124](0124-string-split-and-to-int.md) (`split`)
- **Evidence:** `examples/interp.ail`, `examples/json.ail`, `examples/sheet.ail`

## Context

ADR 0089 left multi-line strings at one sentence ("common leading indentation
is removed") and gave `str` no methods beyond ADR 0124's two. The three
programs embed text (16 scripts for the interpreter, a JSON document and
malformed inputs, a spreadsheet's budget) and search, trim, split and pad
text. The multi-line rule was read two ways: the interpreter's drafter kept the
newline before the closing `"""`, the spreadsheet's dropped it. The sheet
splits its text with `split("\n")`, so keeping it would add an empty last piece
and print one stray error line that the expected output doesn't have.

## Options considered

Measured with tokbench (mean of five tokenizers):

| Question | Result |
|---|---|
| Raw strings, interp scripts | 1272.6 escaped, 1267.4 raw (−0.4%); whole file −0.08% |
| Raw strings, json | document 340.8 → 327.2 (−4.0%); malformed inputs 124.6 → 133.2 (+6.9%); whole file −0.12% |
| Short escapes (`"\n"` vs `r"\n"`) | 65.8 vs 70.4 (+7% for raw) |
| Padding, json hex escape plus sheet table | workaround without fill 74.6; `pad_left(w, fill)` 62.2; `pad_left` plus `pad_left_with` 61.2; `to_hex(width)` 53.2 |
| Padding, json site alone | 21.0 with a fill argument, 30.0 without (+43%) |
| `"+-*/(),:<>=".contains(...)` | `(c)` 22.0; `("{c}")` 23.6; `.chars().contains(c)` 26.4 |

## Deciding criterion

**AI understanding** decides the literals. Raw strings tie on tokens (−0.08%
and −0.12% of whole files; two tokenizers get worse, three better) but remove
escape layering: the interpreter's scripts hold 34 `\{` and 3 `\\`, and a Mini
`\"` is written `\\"`. The multi-line rule ties on tokens and must be exact,
because two readers got opposite meanings. **Token usage** decides the fill
argument (a required fill saves 16.6% over the workaround; `pad_left_with` is
1.6%, a tie, but needs a second name). The `contains` char argument is a tie
(a rare construct, +0.8 tokens a site); AI understanding decides, because
`contains(c)` is what an author writes first (Rust's pattern argument,
Kotlin's `contains(Char)`).

## Decision

**Literals**

1. **Escapes (lex.str.1).** The escapes are `\n \t \r \0 \\ \" \' \{ \u{hex}`.
   An unknown escape is still an error. Its help lists the valid ones and maps
   `\x41`, `\b`, `\f`, `\a` and `\e` to `\u{41}`, `\u{8}`, `\u{c}`, `\u{7}` and
   `\u{1b}`.
2. **`{{` (lex.str.2).** An interpolation that starts with `{`, written `{{`,
   is an error: "a literal brace is `\{`". (With set literals, `"{{x}}"`
   would otherwise print `{1}}`.)
3. **Multi-line strings (lex.str.3).** Exactly:
   1. `"""` then text then `"""` on one line is a one-line string (no dedent),
      useful for text containing `"`. The closing delimiter is the last three
      quotes of a run, so `"""say "hi""""` is `say "hi"`. Quotes right after
      the opening belong to the text.
   2. Otherwise the opening `"""` must end its line (only spaces may follow).
      That newline isn't part of the text.
   3. The closing line is the line with the closing `"""`. If only spaces
      precede the `"""`, those spaces count toward the common indentation, and
      the text ends at the end of the previous line: the newline before the
      closing line is **not** part of the text (the Swift rule). Code may
      follow the delimiter (`"""),`, or a comment). If text precedes the
      `"""`, that line is the last text line.
   4. The common indentation is the fewest leading spaces among the non-blank
      text lines and the closing line. It is removed from every line. Lines of
      only spaces become empty. Only spaces indent: a tab in a text line's
      leading whitespace is an error.
   5. Dedent happens on the source text, before escapes and interpolation, so
      a `\t` or `\u{20}` is content, not indentation. Escapes and
      interpolation then work as in lex.str.1–2 (an interpolation can't span
      lines). Source line endings (CRLF) become `\n`.
   6. lex.stmt.2 (amended): a newline inside an unclosed multi-line string
      continues the statement. lex.block.1 (amended): lines inside a string
      literal aren't block structure.
4. **Raw strings (lex.str.6).** `r"..."` ends at the next `"` and can't
   contain `"` or a newline. `r"""..."""` is multi-line by lex.str.3, or one
   line by its rule 1. Neither has escapes or interpolation: `\` and `{` are
   literal. `r` must be immediately followed by `"` (`r "x"` is the name `r`,
   then a string).

```ail
a := """
    a
    b
    """                  # "a\nb": the closing line is layout, not text
b := """
    a
    b

    """                  # "a\nb\n": to end with a newline, leave a blank line
c := """say "hi""""      # say "hi"
d := r"C:\dir\new"       # backslashes are literal
e := r""""bad \u12G4""""  # "bad \u12G4"
```

**Methods and text form**

5. **`s.bytes()` (type.str.2).** The byte view is `s.bytes()`, with parentheses
   like every method (expr.op.3). It isn't implemented yet.
6. **Char arguments (type.str.6).** `contains`, `starts_with`, `ends_with` and
   `split` accept a `char` as well as a `str`; a char means the one-character
   string. This is a built-in exception to expr.call.2, limited to these four
   methods and not available to user code. `strip_prefix` and `strip_suffix`
   take a `str` only.
7. **Methods (type.str.7).** `s.chars() List[char]` (a new list; strings stay
   unindexable), `s.lines() List[str]` (splits on `\n`, drops one trailing empty
   piece, strips a `\r` before each `\n`: Python's `splitlines` for `\n` and
   `\r\n`), `s.trim()` (removes leading and trailing `is_space` characters,
   [ADR 0134](0134-characters.md)), `starts_with(p)`, `ends_with(p)`,
   `strip_prefix(p) str?`, `strip_suffix(p) str?`, `contains(sub)`.
8. **Numbers (type.str.8).** `s.to_float()` is defined by type.num.8 ([ADR 0130](0130-floats.md)).
   `n.to_hex()` (int) is lowercase, with no prefix and a `-` before the
   magnitude, so `int`'s minimum is `-8000000000000000`.
9. **Padding (type.str.9).** `s.pad_left(width, fill char)` and
   `s.pad_right(width, fill char)` add `fill` until the string has `width`
   characters (scalar values). A string already that wide, or any `width <= 0`,
   leaves it unchanged. The fill is required: AIL has no default arguments.
10. **`s.repeat(n)` (type.str.10)** gives `n` copies and traps if `n < 0`.
11. **Not added.** `s.len()` stays absent (bytes or characters is a separate
    open question); its help says `s.chars().len()`. `for c in s` gets the help
    `s.chars()`, and `s[i]` gets a help. Joining is `xs.join(sep)` on any list
    ([ADR 0136](0136-list-methods-and-slices.md)); there is no `str.from_chars`. Helps for `rjust`, `ljust`,
    `zfill` and `padStart` point to `pad_left`.
12. **Text form inside values (trait.auto.2).** A `str` is written in `"`
    and a `char` in `'`. Escaped: `\\`, the delimiting quote, `\n \t \r \0`,
    and other control characters (Cc) as `\u{hex}` (lowercase, no padding).
    Everything else is written as is, including `{` and the other quote.

```ail
"a,b".split(',')                          # ["a", "b"]
"+-*/".contains(c)                        # c is a char
"a\nb\n".lines()                          # ["a", "b"]
"v1.2".strip_prefix("v")                  # "1.2"
"\\u{c.code().to_hex().pad_left(4, '0')}" # for '\u{7}': \u0007
print(["a\"b", "\{x}"])                   # ["a\"b", "{x}"]
```

## Consequences

- Sheet's `split("\n")` and the interpreter's `"""),` closings work unchanged.
  `s.lines()` ports the Python `splitlines` idiom under either reading.
- A multi-line string can't hold three quotes in a row unescaped. A plain one
  writes `\"`; a raw one can't.
- The text form is defined by the spec, not by the bootstrap's Rust `{:?}`.
- The quick reference needs the multi-line example, `{{`, `to_hex()` (`ff`, no
  `0x`) and "plain strings are cheaper for short escapes".

## Rejected alternatives

- **Keep the final newline (Java text blocks, Python):** sheet's `split("\n")`
  would print a stray error line, and two drafting agents read the old wording
  in opposite ways. The Swift rule makes both programs right, and `lines()`
  covers the idiom.
- **Keeping `"""` multi-line only:** the JSON malformed inputs need quotes
  inside a one-line string, so rule 1 allows the one-line form.
- **`pad_left(width)` without a fill:** +43% on the JSON site, and a second name
  (`pad_left_with`) would be needed for the fill.
- **`contains(char)` only through `"{c}"`:** a tie on tokens, but the first
  attempt fails to type-check.
- **`str.from_chars`:** `join` measured −13% on seven sites, and there should
  be one way.
- **`{{` as a brace escape (Python, Rust, C#):** `\{` is already AIL's escape,
  and an error teaches it.
- **Rust's `{:?}` escapes for text forms:** they follow the toolchain's Unicode
  printability tables, which change between versions.
