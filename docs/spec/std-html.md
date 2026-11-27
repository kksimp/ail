# Standard library: `std.html`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR 1272). Reads HTML the way a browser does,
> finds elements with CSS selectors, gives the text a reader sees, writes HTML back and cleans
> untrusted HTML with an allowlist. Scripting, CSS layout, encodings other than the text's own and
> the parts of the WHATWG tree builder listed in `std.html.11` are out of scope.

## Reading

**std.html.1** — `use std.html` provides `parse(text str) Node`, which reads a whole document and
returns its `html` element. Reading never fails: any text, however broken, gives a tree, as in a
browser. The tokenizer follows the WHATWG HTML tokenizer (`std.html.5` to `std.html.7`) and the tree
builder its rules for a body (`std.html.8` to `std.html.10`), within the limits of `std.html.11`.
The module is pure (`eff.infer.1`): it has no effects and needs no capability. *(ADR 1272)*

```ail rules=std.html.1
fn main()
    doc := html.parse("<title>Hi</title><p>one<p>two")
    print(html.write(doc))
```

**std.html.2** — `enum Node` has the variants `Elem(tag str, attrs List[Attr], kids List[Node])`,
`Text(content str)` and `Comment(content str)`. `struct Attr` has the fields `name str` and
`value str`. Details: [below](#more-on-stdhtml2). *(ADR 1272)*

```ail rules=std.html.2
fn main()
    link := html.elem("a", [html.Attr(name: "href", value: "/x")], [html.text_node("go")])
    print(html.write(link))
```

**std.html.3** — A node has these methods, all total: `is_elem()` is true for an element; `name()` is
the tag, and `""` for text and comments; `text()` is a text node's content, `""` for a comment, and for
an element the texts of its descendants joined in document order, `script` and `style` included;
`attr(key) str?` is the value of the attribute `key`, and `none` when there is none or the node is not
an element; `attrs()` is the attributes; `children()` is the children, elements, text and comments;
`elements()` is the element children only; `has_class(name)` is true when `name` is one of the
whitespace-separated words of the `class` attribute. *(ADR 1272)*

```ail rules=std.html.3
fn main()
    p := html.parse_fragment("<p id=x class='a b'>one <b>two</b><!-- c --></p>")[0]
    print("{p.name()} {p.attr("id") else "-"} {p.attr("zz") else "-"} {p.has_class("b")}")
    print(p.text())
    print("{p.children().len()} {p.elements().len()} {p.attrs().len()} {p.is_elem()}")
```

**std.html.4** — `parse_fragment(text str) List[Node]` reads `text` as the content of a `body`: the same
rules, but no `html`, `head` or `body` element is added, and `<html>`, `<head>` and `<body>` tags are
ignored. Details: [below](#more-on-stdhtml4). *(ADR 1272)*

```ail rules=std.html.4
fn main()
    nodes := html.parse_fragment("<li>a<li>b<body>c")
    print(nodes.len())
    print(html.write_all(nodes))
```

**std.html.5** — Tokens. A line break is CRLF, CR or LF, and reads as LF; a NUL reads as U+FFFD. `<`
followed by an ASCII letter starts a tag; its name runs to whitespace, `/` or `>`. Details: [below](#more-on-stdhtml5). *(ADR 1272)*

```ail rules=std.html.5
fn main()
    print(html.write_all(html.parse_fragment("<DIV Id=a class='x' hidden id=b>a < b</div><?pi?><a href")))
```

**std.html.6** — Character references in text and attribute values: `&name;` for each name of the
HTML 4 set, `&apos;`, and `&AMP;`, `&COPY;`, `&GT;`, `&LT;`, `&QUOT;` and `&REG;`; the legacy names
(the Latin-1 ones and those six, with `amp`, `gt`, `lt` and `quot`) also match with no `;`, the longest
first, so `&notit;` is `¬it;`. Details: [below](#more-on-stdhtml6). *(ADR 1272)*

```ail rules=std.html.6
fn main()
    print(html.unescape("&lt;b&gt; &copy &eacute; &#x41;&#66; &#x80; &nosuch; AT&T"))
    a := html.parse_fragment("<a href='?a=1&copy=2&lt=3'>x</a>")[0]
    print(a.attr("href") else "")
```

**std.html.7** — The content of `script`, `style`, `xmp`, `iframe`, `noembed` and `noframes` is text up to
the matching end tag, in any case, with no tag or reference read inside; that of `title` and `textarea`
is the same, with references resolved. Details: [below](#more-on-stdhtml7). *(ADR 1272)*

```ail rules=std.html.7
fn main()
    s := html.parse_fragment("<script>if (a<b) x('</p>')</SCRIPT>")[0]
    print(s.text())
    print(html.parse_fragment("<textarea>\n&lt;b&gt;</textarea>")[0].text())
```

## Building the tree

**std.html.8** — Elements close as in a browser. Details: [below](#more-on-stdhtml8). *(ADR 1272)*

```ail rules=std.html.8
fn main()
    print(html.write_all(html.parse_fragment("<p>a<div>b</div><ul><li>c<li>d</ul></span>e</p>")))
```

**std.html.9** — Tables. `tr` in a `table` opens a `tbody` first, and `td` or `th` opens a `tbody` and a
`tr` as needed; a new `tr`, `td`, `th` or table section closes the open row or cell. `col` opens a
`colgroup`. Details: [below](#more-on-stdhtml9). *(ADR 1272)*

```ail rules=std.html.9
fn main()
    print(html.write_all(html.parse_fragment("<table><tr><td>1<td>2<tr><td>3</table><td>x")))
```

**std.html.10** — A document's `html` element always holds `head` then `body`. Details: [below](#more-on-stdhtml10). *(ADR 1272)*

```ail rules=std.html.10
fn main()
    doc := html.parse("<meta charset=utf-8>\n<title>T</title>Hello<body class=x><style>p{}</style>")
    print(html.write(doc))
```

**std.html.11** — Limits. Details: [below](#more-on-stdhtml11). *(ADR 1272)*

```ail rules=std.html.11
fn main()
    print(html.write_all(html.parse_fragment("<b><i>x</b>y</i>z")))
    deep := html.parse_fragment("<div>".repeat(600))
    print(deep.len())
```

## Selectors

**std.html.12** — `n.select(sel) List[Node] ! SelectorError` is every element below `n` that matches the
CSS selector `sel`, in document order and each once; `n` itself is never in it, and its ancestors are
not seen. Details: [below](#more-on-stdhtml12). *(ADR 1272)*

```ail rules=std.html.12
fn main() !
    doc := html.parse("<div id=nav><a href=/a>A</a></div><p><a href=/b>B</a></p>")
    print(doc.select("#nav a, p > a")?.map(|n| n.attr("href") else ""))
    first := doc.select_one("p a")? else return
    print(first.text())
```

**std.html.13** — A selector is a comma-separated list of complex selectors; each is compound selectors
joined by whitespace (descendant) or `>` (child). Details: [below](#more-on-stdhtml13). *(ADR 1272)*

```ail rules=std.html.13
fn main() !
    doc := html.parse("<a href='https://x/a.pdf' class='btn big'>1</a><a href='/b' lang=en-GB>2</a>")
    for sel in ["a.btn.big", "a[href^=https][href$='.pdf']", "[LANG|=en]", "a[class~=big]", "A"]
        found := doc.select(sel)?
        print("{sel}: {found.map(|n| n.text())}")
```

**std.html.14** — `SelectorError` is an error with `kind SelectorKind`, `why str` and `at`, the 0-based
character where reading stopped; it displays as `why at N`. Details: [below](#more-on-stdhtml14). *(ADR 1272, type.enum.7)*

```ail rules=std.html.14
fn main()
    doc := html.parse("<p>x</p>")
    for sel in ["", "p:first-child", "p + p", "p["]
        _ := doc.select(sel) else err
            print("{err.kind.name()}: {err}")
            continue
```

## Text and writing

**std.html.15** — `to_text(node) str` is the text a reader sees. Whitespace runs are one space, and
spaces at the start and end of lines go; each block element (`div`, `p`, `li`, `tr`, headings, lists,
tables and the rest of the HTML block elements) is on lines of its own, with a blank line around a
`p`; `br` is a line break; cells of a row are separated by a tab; `pre`, `textarea`, `listing` and
`plaintext` keep their text as written; `head`, `script`, `style`, `template` and comments are left
out. There is no line break at the start or end. *(ADR 1272)*

```ail rules=std.html.15
fn main()
    page := html.parse("<h1>Title</h1><p>Hello   <b>world</b>!<br>Bye</p><ul><li>a<li>b</ul><script>x()</script>")
    print(html.to_text(page))
```

**std.html.16** — `write(node) str` is the HTML text of `node`: every attribute as `name="value"`, void
elements with no end tag, every other element with one, comments as `<!--text-->`. Details: [below](#more-on-stdhtml16). *(ADR 1272)*

```ail rules=std.html.16
fn main()
    print(html.write_all(html.parse_fragment("<img alt='\"x\"' src=a.png><p>1 &lt; 2<br></p><style>a>b{}</style>")))
```

**std.html.17** — `escape(s) str` writes `&`, `<`, `>`, `"` and `'` as `&amp;`, `&lt;`, `&gt;`, `&quot;` and
`&#39;`, so the result is safe as element text and in an attribute value quoted either way. Details: [below](#more-on-stdhtml17). *(ADR 1272)*

```ail rules=std.html.17
fn main()
    e := html.escape("<a title=\"x\">Tom & 'Jerry'</a>")
    print(e)
    print(html.unescape(e))
```

## Sanitizing

**std.html.18** — `sanitize(text) str` is `sanitize_with(text, policy())`. `policy() Policy` keeps the
elements `a`, `abbr`, `b`, `blockquote`, `br`, `caption`, `cite`, `code`, `dd`, `del`, `div`, `dl`, `dt`,
`em`, `figcaption`, `figure`, `h1` to `h6`, `hr`, `i`, `img`, `ins`, `kbd`, `li`, `mark`, `ol`, `p`,
`pre`, `q`, `s`, `small`, `span`, `strong`, `sub`, `sup`, `table`, `tbody`, `td`, `tfoot`, `th`,
`thead`, `tr`, `u` and `ul`; the attributes `title`, `lang` and `dir` on all of them, `href` on `a`,
`src`, `alt`, `width` and `height` on `img`, `colspan` and `rowspan` on `td` and `th` and `start` on
`ol`; the URL schemes `http`, `https` and `mailto`; and drops, with their content, `script`, `style`,
`template`, `iframe`, `object`, `embed`, `frame`, `frameset`, `noscript`, `noembed`, `noframes`, `xmp`,
`plaintext`, `math`, `svg`, `head`, `title`, `meta`, `link`, `base`, `textarea`, `select` and
`option`. *(ADR 1272)*

```ail rules=std.html.18
fn main()
    print(html.sanitize("<p onclick=x>Hi <script>alert(1)</script><a href='javascript:x'>a</a><font>f</font></p>"))
```

**std.html.19** — `struct Policy` has the fields `tags Set[str]`, `attrs Map[str, List[str]]`,
`schemes Set[str]` and `drop Set[str]`. `sanitize_with(text, p) str` reads `text` with `parse_fragment`
and writes back: comments are removed; an element in `p.drop` is removed with its content; any other
element not in `p.tags` is replaced by its cleaned children; an element in `p.tags` keeps the attributes
`p.attrs` names for its tag or for `"*"`. An attribute whose name starts with `on`, and `srcset`, are
always removed. The URL attributes (`action`, `background`, `cite`, `formaction`, `href`, `longdesc`,
`poster`, `src`) keep a value only when, with every character up to U+0020 removed and ASCII case
folded, it has no `:` before its first `/`, `?` or `#` (a relative URL, `//host` included) or its scheme
before the `:` is in `p.schemes`. *(ADR 1272)*

```ail rules=std.html.19
fn main()
    mut p := html.policy()
    p.tags.add("video")
    p.attrs["video"] = ["src", "controls"]
    print(html.sanitize_with("<video src=movie.mp4 controls onplay=x>v</video><a href=' JaVa&#9;script:x'>a</a>", p))
```

**std.html.20** — The result of `sanitize_with` is written by `write` (`std.html.16`) from the cleaned
tree, so every tag is closed, every attribute quoted and every `<` and `&` of text escaped: it holds
only the elements and attributes the policy allows, and reading it again gives the same tree. Details: [below](#more-on-stdhtml20). *(ADR 1272)*

```ail rules=std.html.20
fn main()
    clean := html.sanitize("<p>unclosed <b>bold <i>both</p>1 < 2 & 3")
    print(clean)
    print(html.sanitize(clean) == clean)
```

### `std.html` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on std.html.10

Before the body starts, `base`, `link`, `meta`, `noscript`, `script`, `style`, `template` and `title` go in `head`, and whitespace is dropped; any other tag or text starts the body, and everything after goes in `body`. A second `<html>` or `<body>` tag adds the attributes the element lacks. `</html>`, `</head>` and `</body>` are ignored.

#### More on std.html.11

The tree builder does not run the WHATWG adoption agency or reopen formatting elements, so misnested inline tags close at the first end tag that matches an enclosing element (`<b><i>x</b>y` gives `<b><i>x</i></b>y`). Text and elements inside a `table` but outside its cells move before the table, except `style`, `script`, `form` and `input type=hidden`. SVG and MathML are read as HTML elements with lower case names, apart from `/>` (`std.html.8`). `template`, `frameset`, `noscript` in `head` and the full HTML 5 list of character reference names (the HTML 4 names are read) are not supported. A `select` holds only `option`, `optgroup`, `script` and `template`; `<select>`, `input`, `keygen` and `textarea` close it. An `rb` or `rtc` closes open ruby elements, an `rt` or `rp` closes `rb`, `rt` and `rp`. At most 512 elements are open at once; a deeper start tag adds its element to the deepest open one, so its content becomes that element's. These limits never lose text.

#### More on std.html.12

`n.select_one(sel) Node? ! SelectorError` is the first of them.

#### More on std.html.13

A compound selector is a type (`p`, any case) or `*`, then any number of `#id`, `.class` and attribute tests: `[a]` (present), `[a=v]` (equal), `[a~=v]` (one of its words), `[a^=v]`, `[a$=v]`, `[a*=v]` (starts, ends, contains; never true for an empty `v`) and `[a|=v]` (equal to `v` or starting `v-`). A value is a quoted string or a name; names are ASCII letters, digits, `-`, `_` and any non-ASCII character, with no escapes. Attribute names are any case; ids, classes and values are matched as written.

#### More on std.html.14

`SelectorKind` is `Empty` (only spaces), `Syntax` (not a selector) and `Unsupported` (pseudo-classes such as `:first-child` and the `+` and `~` combinators).

#### More on std.html.16

Text is written with `&`, `<` and `>` as references, and attribute values also `"`; text inside `script`, `style`, `xmp`, `iframe`, `noembed`, `noframes` and `plaintext` is written as it is. `write_all(nodes) str` writes the nodes one after another. No DOCTYPE is written.

#### More on std.html.17

Nothing else changes. `unescape(s) str` resolves every character reference in `s` as in text (`std.html.6`), so `unescape(escape(s)) == s`.

#### More on std.html.2

Tags and attribute names are lower case (ASCII only); an element keeps its attributes and children in source order. `elem(tag, attrs, kids) Node` and `text_node(content) Node` build a node by hand; nothing is checked, so a tag that is not a name writes text that reads back differently.

#### More on std.html.20

Text is never removed except inside a dropped element.

#### More on std.html.4

It is what a template, a comment box or `sanitize` reads.

#### More on std.html.5

An attribute is a name with no value, or `name=value` with the value in double quotes, single quotes or none (up to whitespace or `>`); a name repeated in one tag keeps its first value. A tag that ends in `/>` is self-closing (`std.html.8`). `<!-- ... -->` is a comment, also ended by `--!>`; `<!-->` and `<!--->` are empty comments; a comment the text does not close runs to the end. A comment before the first tag or after `</html>` belongs to the document, not to the `html` element `parse` returns, and is dropped; one after `<html>` is the first child of `html`, one after `</head>` or `</body>` is a child of `html`. `<!DOCTYPE ...>` is dropped, but a text with no `<!DOCTYPE html>` is read in quirks mode, where `<table>` does not close an open `<p>`. U+0000 is dropped from body text and is U+FFFD elsewhere. In `script` text a `<!--` opens an escape in which `<script` opens a second one, and `</script>` ends the element only outside the second. `<!x>`, `<?x>` and `</ x>` (an end tag that starts with no letter) are comments up to the next `>`, and `</>` is dropped. A `<` that starts none of these is text. A tag the text ends inside is dropped.

#### More on std.html.6

In an attribute value a legacy name with no `;` followed by a letter, digit or `=` stays as written, so `?a=1&copy=2` is kept. `&#N;` and `&#xH;` (the `;` optional) name a code point; 0, a surrogate or a value above U+10FFFF is U+FFFD, and 0x80 to 0x9F are the windows-1252 characters. Any other `&` is itself.

#### More on std.html.7

After `<plaintext>` the rest of the text is text. A line break right after `<pre>`, `<listing>` or `<textarea>` is dropped.

#### More on std.html.8

A block start tag (`address`, `article`, `aside`, `blockquote`, `div`, `dl`, `fieldset`, `figure`, `footer`, `form`, `h1` to `h6`, `header`, `hr`, `main`, `nav`, `ol`, `p`, `pre`, `section`, `table`, `ul` and the rest of the WHATWG list) closes an open `p`. `li` closes an open `li`, and `dd` or `dt` an open `dd` or `dt`, unless a block other than `address`, `div` and `p` stands between. A heading closes a heading that is the current element. `option` closes an open `option`; `a` and `button` close an open one of their own. The void elements (`area`, `base`, `br`, `col`, `embed`, `hr`, `img`, `input`, `link`, `meta`, `source`, `track`, `wbr` and the obsolete `basefont`, `bgsound`, `keygen`, `param`) have no content. `<image>` is `<img>`. `/>` is ignored, except on `svg`, `math` and elements inside them, where it closes the element. An end tag closes its element and every element opened inside it; one that matches no open element is ignored, as is one for an inline element past an open block. `</p>` with no open `p` adds an empty `p`, and `</br>` is `<br>`.

#### More on std.html.9

A table part (`caption`, `col`, `colgroup`, `tbody`, `td`, `tfoot`, `th`, `thead`, `tr`) outside any open table is ignored, and its text stays.
