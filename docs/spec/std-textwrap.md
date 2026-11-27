# Standard library: `std.textwrap`

Chapter key: `std`

> **Status:** Drafted. Written in AIL (`std/textwrap/textwrap.ail`), ADR 1223.

Text laid out in terminal columns: widths, truncation, padding, wrapping,
indentation, aligned tables and natural sort order. Every function is pure
(`eff.infer.1`). The module is in scope without `use` (`std.use.4`).

## Width

**std.textwrap.1** — `char_width(c char) int` is the columns one character
takes: 0 for a control character (code under 32, or 127 to 159, a tab
included), a combining mark or a zero-width character; 2 for an East Asian
wide or fullwidth character or an emoji presentation character; 1 otherwise.
It is `unicode.char_width` ([std.unicode.9](std-unicode.md)), from the Unicode
East Asian Width and general category data. *(ADR 1223, ADR 1296)*

**std.textwrap.2** — `width(s str) int` is the columns `s` takes, where an ANSI
escape sequence (`ESC [ ... final`, `ESC ] ... BEL` or `ESC ] ... ESC \`, or
ESC and the next character) takes 0 columns and the text between escape
sequences is measured by grapheme cluster with `unicode.width`
([std.unicode.10](std-unicode.md)): a flag, an emoji `ZWJ` sequence and an
emoji followed by U+FE0F take 2, a letter with its accents takes its base's
width. `s.len()` counts characters; `width` counts what a terminal shows.
`truncate`, `wrap` (for a word wider than the limit) and the tables cut and
pad by the same clusters, never inside one. *(ADR 1223, ADR 1296)*

```ail rules=std.textwrap.1,std.textwrap.2
fn main()
    print(textwrap.width("hello"))      # 5
    print(textwrap.width("日本語"))       # 6
    print(textwrap.char_width('a'))     # 1
    print(textwrap.char_width('日'))    # 2
```

**std.textwrap.3** — `strip_ansi(s str) str` is `s` without its escape
sequences, as `width` reads them. *(ADR 1223)*

```ail rules=std.textwrap.3
fn main()
    esc := char.from_code(27) else ' '
    red := "{esc}[31mred{esc}[0m"
    print(textwrap.strip_ansi(red))     # red
    print(textwrap.width(red))          # 3
```

## Cutting and padding

**std.textwrap.4** — `truncate(s str, cols int, tail str = "…") str` is `s`
when `width(s) <= cols`; otherwise the longest start of `s` that leaves room
for `tail`, followed by `tail`, so the result takes at most `cols` columns.
A wide character that does not fit is dropped whole. A `tail` that is itself
`cols` columns or wider is cut to `cols` columns and replaces the text. An
escape sequence inside the kept start is kept whole, so a colour may stay
open: end it yourself. *(ADR 1223)*

```ail rules=std.textwrap.4
fn main()
    print(textwrap.truncate("hello world", 8))               # hello w…
    print(textwrap.truncate("hello world", 8, tail: ".."))   # hello ..
    print(textwrap.truncate("hello", 8))                     # hello
```

**std.textwrap.5** — `pad_to_width(s str, cols int, align char = 'l') str`
adds spaces until `s` takes `cols` columns: `'l'` after the text, `'r'`
before it, `'c'` on both sides (the odd space goes after). Any other `align`
is `'l'`. A text already `cols` columns wide or more is returned whole;
padding never cuts. *(ADR 1223)*

```ail rules=std.textwrap.5
fn main()
    a := textwrap.pad_to_width("ab", 5, align: 'r')
    b := textwrap.pad_to_width("日", 4)
    print("[{a}]")    # [   ab]
    print("[{b}]")    # [日  ]
```

## Wrapping

**std.textwrap.6** — `wrap(s str, cols int) List[str]` fills lines greedily
with at most `cols` columns each. Words are the pieces between whitespace
(`s.words()`): runs of whitespace, line breaks included, become one space.
A blank line ends a paragraph and shows as one empty string between the two
paragraphs' lines. A word wider than `cols` is cut into pieces of at most
`cols` columns (a character wider than `cols` stands alone); the last piece
can take following words. A `cols` under 1 is 1. `fill(s, cols) str` is
`wrap(s, cols).join("\n")`. *(ADR 1223)*

```ail rules=std.textwrap.6
fn main()
    for ln in textwrap.wrap("the quick brown fox jumps over the lazy dog", 15)
        print(ln)        # the quick brown, fox jumps over, the lazy dog
    print(textwrap.fill("aa bb cc", 5))    # aa bb, then cc on the next line
```

## Indentation

**std.textwrap.7** — `dedent(s str) str` removes the leading blanks (spaces
and tabs, compared as characters) that every non-blank line of `s` starts
with, and turns each blank line (empty or only whitespace) into an empty
line. Lines are split at `\n` and joined with `\n`, so a final `\n` stays. *(ADR 1223)*

**std.textwrap.8** — `indent(s str, prefix str) str` puts `prefix` at the
start of every non-blank line of `s`; blank lines stay as they are. *(ADR 1223)*

```ail rules=std.textwrap.7,std.textwrap.8
fn main()
    src := ["    a", "      b", "", "    c"].join("\n")
    print(textwrap.dedent(src))            # a, "  b", an empty line, c
    print(textwrap.indent("a\n\nb", "> ")) # "> a", an empty line, "> b"
```

## Tables

**std.textwrap.9** — `table(rows List[List[str]], align str = "") str` lays
`rows` out in columns: each column is as wide (by `width`) as its widest
cell, cells are padded with `pad_to_width`, columns are separated by two
spaces, and trailing spaces of each line are removed. `align` has one
character per column, `l`, `r` or `c` (a missing or other character is `l`).
A row with fewer cells than the longest row is read as if padded with empty
cells. No rows give `""`. A cell is one line: a `\n` in it is not handled. *(ADR 1223)*

```ail rules=std.textwrap.9
fn main()
    rows := [["name", "qty"], ["apple", "3"], ["fig", "12"]]
    print(textwrap.table(rows, align: "lr"))
    # name   qty
    # apple    3
    # fig     12
```

**std.textwrap.10** — `table_grid(rows List[List[str]], align str = "") str`
lays the same columns out as a grid: a rule `+----+` above and below, each
row as `| a | b |` with the cell padded to its column, and a rule after the
first row (the header) when more rows follow. *(ADR 1223)*

```ail rules=std.textwrap.10
fn main()
    print(textwrap.table_grid([["name", "qty"], ["apple", "3"]], align: "lr"))
    # +-------+-----+
    # | name  | qty |
    # +-------+-----+
    # | apple |   3 |
    # +-------+-----+
```

## Natural order

**std.textwrap.11** — `natural_key(s str) str` is a text whose byte order
is the natural order of `s`: a run of ASCII digits compares by its number,
leading zeros ignored, and sorts before any other character at the same
place, so `img2` comes before `img10`. Letters keep their case order
(`"B" < "a"`): for case-blind order pass `s.to_lower()`. `natural_cmp(a, b)
int` is -1, 0 or 1; strings equal in natural order (`a01`, `a1`) compare by
their text. Use the key with `sorted(by:)`. *(ADR 1223)*

```ail rules=std.textwrap.11
fn main()
    names := ["img12.png", "img10.png", "img2.png", "img1.png"]
    print(names.sorted(by: |n| textwrap.natural_key(n)).join(", "))
    # img1.png, img2.png, img10.png, img12.png
    print(textwrap.natural_cmp("a2", "a10"))    # -1
```

## Limits

**std.textwrap.12** — Width follows the Unicode data, not the terminal: a
character that is unassigned in the Unicode version of `std.unicode`
(`unicode.version()`) is 1 column (2 in the CJK blocks and planes 2 and 3),
ambiguous-width characters are 1, and a
terminal that draws a character or sequence differently will misalign text;
the rules are not configurable. `wrap` breaks only at whitespace; for text
without spaces use `unicode.line_segments` ([std.unicode.11](std-unicode.md)).
*(ADR 1223, ADR 1296)*

```ail rules=std.textwrap.12
fn main()
    print(textwrap.char_width('é'))     # 1
    print(textwrap.char_width('€'))     # 1
```
