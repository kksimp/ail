# Standard library: `std.markdown`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR-TBD, assigned by the integrator). Reads
> CommonMark text into a tree and writes it as HTML with GitHub's tables, strikethrough and bare links.
> Raw HTML, link reference definitions, footnotes and the HTML-block rules are out of scope
> (`std.markdown.15`).

## The module

**std.markdown.1** — `use std.markdown` provides `parse(text str) List[Block]`. Every text is a document,
so `parse` cannot fail: text that is not valid Markdown is read as plain text. CRLF and a lone CR read as
LF, and tabs in the leading whitespace of a line stand for spaces up to the next multiple of 4. The
module is pure (`eff.infer.1`): it has no effects and needs no capability. *(ADR-TBD)*

```ail rules=std.markdown.1
fn main()
    blocks := markdown.parse("# Hi\r\n\r\nthere")
    print(blocks.len())
    print(markdown.parse("").len())
```

**std.markdown.2** — `enum Block` has the variants `Heading(level, kids)`, `Para(kids)`, `Rule`,
`Quote(kids)`, `CodeBlock(info, code)`, `ListBlock(ordered, start, tight, items)` and
`Table(aligns, head, rows)`. `enum Inline` has `Text(content)`, `Code(content)`, `Emph(kids)`,
`Strong(kids)`, `Strike(kids)`, `Link(href, title, kids)`, `Image(src, title, alt)`, `SoftBreak` and
`HardBreak`. A list item is a `List[Block]`; a table cell is a `List[Inline]`. Text nodes hold the
characters with escapes and entities resolved, never HTML. *(ADR-TBD)*

```ail rules=std.markdown.2
fn main()
    blocks := markdown.parse("one *two*")
    print(blocks.len())
    print(markdown.render(blocks))
```

**std.markdown.3** — A heading is an ATX heading (one to six `#`, then a space or the end of the line, with an
optional closing run of `#`) or a setext heading (a paragraph followed by a line of `=` for level 1 or
`-` for level 2). `#hashtag` and seven `#` are paragraphs. *(ADR-TBD)*

```ail rules=std.markdown.3
fn main()
    print(markdown.html("# One ##\n\nTwo\n---\n\n#hashtag"))
```

**std.markdown.4** — A paragraph runs to a blank line or to the start of another block. A thematic break is
three or more `-`, `*` or `_` alone on a line (spaces allowed). A line indented four spaces or more
outside a paragraph starts an indented code block. A fenced code block opens with three or more backticks
or tildes and closes with a fence of the same character that is at least as long, or at the end of the
text; the first word of the text after the opening fence is the language, written as
`class="language-NAME"`. *(ADR-TBD)*

```ail rules=std.markdown.4
fn main()
    print(markdown.html("~~~ail\nprint(1 < 2)\n~~~\n\n    indented\n\n***"))
```

**std.markdown.5** — A block quote is lines that start with `>`; a line without it continues the quote's
paragraph (lazy continuation). A list item starts with `-`, `+` or `*`, or with up to nine digits and `.`
or `)`; its content is the lines indented to the text after the marker, and a line without that indent
continues a paragraph. Changing the bullet character or the number delimiter starts a new list. A list is
loose when a blank line separates two items or two blocks inside an item, and then each item's paragraphs
are wrapped in `<p>`; otherwise it is tight. An ordered list keeps its first number as `start`. A number
other than 1 cannot interrupt a paragraph. *(ADR-TBD)*

```ail rules=std.markdown.5
fn main()
    print(markdown.html("> quoted\nlazy\n\n- a\n- b\n\n3. x\n\n4. y"))
```

**std.markdown.6** — In inline text a backslash before an ASCII punctuation character gives that character;
`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, `&nbsp;`, `&copy;`, `&reg;`, `&hellip;`, `&mdash;`, `&ndash;`
and numeric references (`&#65;`, `&#x41;`) give their character, and any other `&` stays. A run of
backticks opens a code span that ends at the next run of the same length; a span with no end is text. A
line break is a `HardBreak` after two spaces or a backslash, and a `SoftBreak` otherwise. *(ADR-TBD)*

```ail rules=std.markdown.6
fn main()
    print(markdown.html("`a < b` \\* &copy; &#65;\none  \ntwo"))
```

**std.markdown.7** — `*em*` and `_em_` give `Emph`, `**strong**` and `__strong__` give `Strong`, and
`~~strike~~` gives `Strike`, paired by the CommonMark delimiter rules (an `_` inside a word is text). A
link is `[text](dest "title")`, an image `![alt](src "title")`; the destination may be written `<...>`
and the title in `"..."`, `'...'` or `(...)`. A link text holds no link, but may hold an image. `<scheme:rest>` and `<a@b.c>`
are links, and so is a bare `http://` or `https://` address after a space or an opening mark, without
its trailing punctuation. Reference links (`[text][ref]`) are not read (`std.markdown.15`). *(ADR-TBD)*

```ail rules=std.markdown.7
fn main()
    print(markdown.html("**a** ~~b~~ [c](/d \"e\") <me@x.org> see http://x.org/a."))
```

**std.markdown.8** — A table is a header row with a `|` in it, then a delimiter row of cells like `---`,
`:--`, `--:` or `:-:` with as many cells as the header; the rows follow until a blank line or another
block. A cell is inline text, `\|` is a pipe in a cell, a short row is padded with empty cells and a long
one is cut. The delimiter row sets each column's alignment, written `align="left"`, `"center"` or
`"right"`. A header and delimiter row that disagree in count are a paragraph. *(ADR-TBD)*

```ail rules=std.markdown.8
fn main()
    print(markdown.html("| a | b |\n|:--|--:|\n| 1 | x \\| y |"))
```

**std.markdown.9** — `html(text str) str` is `render(parse(text))`, and `render(blocks List[Block]) str` writes
HTML: `<h1>` to `<h6>`, `<p>`, `<hr />`, `<blockquote>`, `<pre><code>`, `<ul>`, `<ol>` (with `start` when
not 1), `<li>`, `<table>`, `<em>`, `<strong>`, `<del>`, `<code>`, `<a>`, `<img />` and `<br />`. Every
text is escaped (`&`, `<`, `>` and `"`), and raw HTML in the Markdown is never passed through: `<b>` is
shown as the text `<b>`. A text is safe to serve as the result, whatever it holds. *(ADR-TBD)*

```ail rules=std.markdown.9
fn main()
    print(markdown.html("<script>alert(1)</script> & **ok**"))
```

**std.markdown.10** — `render` writes a link or image only when its destination is relative or its scheme is
`http`, `https`, `mailto`, `tel`, `ftp` or `sms` (compared in lower case, after trimming); any other
scheme (`javascript:`, `data:`, `vbscript:`, `file:`) is dropped: the link shows its text and the image
its `alt`. In `href` and `src`, every character outside letters, digits and ``-._~:/?#@!$&'()*+,;=%[]``
is written as the percent-escapes of its UTF-8 bytes, and the result is then escaped as HTML text.
*(ADR-TBD)*

```ail rules=std.markdown.10
fn main()
    print(markdown.html("[x](javascript:alert(1)) [y](/café) ![z](data:text/html,a)"))
```

**std.markdown.11** — Block quotes and lists nest at most 32 deep, and links and images at most 32 deep in
one another's text; beyond that the text is read as a paragraph or plain text, so no input makes `parse`
recurse without bound. The work is bounded by the input: a long run of `*`, `[` or `>` is read in time
quadratic in its length at worst. *(ADR-TBD)*

```ail rules=std.markdown.11
fn main()
    deep := "> ".repeat(40)
    h := markdown.html("{deep}x")
    print(h.count("<blockquote>"))
```

**std.markdown.12** — `plain(text str) str` is the text without markup: headings and paragraphs, code
blocks, list items and table rows (cells joined by ` | `), one block per line; emphasis, link and image
marks are dropped (an image gives its `alt`) and a line break is a space. *(ADR-TBD)*

```ail rules=std.markdown.12
fn main()
    print(markdown.plain("# T\n\nsome *bold* [link](/x)\n\n- a\n- b"))
```

**std.markdown.13** — `headings(text str) List[Head]` lists the headings in document order, including those
inside block quotes and list items. A `Head` has `level int` and `text str` (the plain text). *(ADR-TBD)*

```ail rules=std.markdown.13
fn main()
    for h in markdown.headings("# A\n\n## B *c*")
        print("{h.level} {h.text}")
```

**std.markdown.14** — The HTML follows the CommonMark reference output for the supported constructs: one
block per line, a tight list item's paragraph without `<p>`, a soft break as a newline, `<hr />` and
`<br />` with a space before the slash. `render` ends every block with a line break, so `html("a")` is
`<p>a</p>` followed by `\n`. *(ADR-TBD)*

```ail rules=std.markdown.14
fn main()
    print(markdown.html("a\nb").len())
```

**std.markdown.15** — Not supported: raw HTML and HTML blocks (shown as text, `std.markdown.9`), link
reference definitions and reference links (`[text][ref]` and `[ref]: /url` stay text), footnotes,
task-list boxes, bare `www.` links, and named entities beyond those of `std.markdown.6`. The
interaction of blank lines, lazy lines and nested containers is read by the simple rules of
`std.markdown.5`, so a few exotic inputs differ from the CommonMark reference; a difference is a bug
to report with its input. *(ADR-TBD)*

```ail rules=std.markdown.15
fn main()
    print(markdown.html("[a][ref]\n\n[ref]: /x"))
```

**std.markdown.16** — `html` is deterministic: the same text gives the same bytes on every host, and no
text can fail, trap or read outside the document. *(ADR-TBD)*

```ail rules=std.markdown.16
fn main()
    print(markdown.html("x") == markdown.html("x"))
```

## Rejected alternatives

- **Passing raw HTML through, with a sanitiser:** a sanitiser is a second parser with its own bugs;
  escaping everything cannot let a script through. A program that needs trusted HTML can write it.
- **A callback or visitor API:** `parse` and `render` over plain enums let a program match, change a
  tree and render it again without new concepts.
- **Link reference definitions in version one:** they need a second pass over the whole document and
  an extra map in the parser; deferred, not refused.
- **Returning `! MarkdownError`:** CommonMark has no invalid input, so an error type would be a case
  nothing can hit.
