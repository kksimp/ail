# Standard library: `std.xml`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR 1212). Reads and writes XML text for
> feeds, vector graphics and property lists. Streaming, namespaces, schemas, XPath beyond slash paths
> and DOCTYPEs are out of scope (`std.xml.6`, `std.xml.14`).

## The module

**std.xml.1** — `use std.xml` provides `parse(text str) Node ! XmlError`, which reads one document and
returns its root element. Anything else in the text is dropped (`std.xml.4`) or refused (`std.xml.15`).
The module is pure (`eff.infer.1`): it has no effects and needs no capability. *(ADR 1212)*

```ail rules=std.xml.1
fn main() !
    root := xml.parse("<?xml version=\"1.0\"?><greeting lang=\"en\">hello</greeting>")?
    print("{root.name()} {root.text()}")
```

**std.xml.2** — `enum Node` has the variants `Elem(tag str, attrs List[Attr], kids List[Node])` and
`Text(content str)`. `struct Attr` has the fields `name str` and `value str`. An element keeps its
attributes and children in document order. `elem(tag, attrs, kids) Node` and `text_node(content) Node`
build a node by hand; the tag is not checked, so a tag that is not a name writes text that does not
parse. *(ADR 1212)*

```ail rules=std.xml.2
fn main()
    page := xml.elem("p", [xml.Attr(name: "class", value: "lead")], [xml.text_node("hi")])
    print(xml.write(page, 0))
```

**std.xml.3** — A node has these methods, all total: `is_elem()` is true for an element; `name()` is the
tag, and `""` for text; `text()` is a text node's content, and for an element the texts of its
descendants joined in document order; `attr(key) str?` is the value of the attribute `key`, and `none`
when there is none or the node is text; `attrs()` is the attributes (none for text); `children()` is the
element's children, elements and text; `elements()` is its element children only. *(ADR 1212)*

```ail rules=std.xml.3
fn main() !
    p := xml.parse("<p id='x'>one <b>two</b> three</p>")?
    print("{p.name()} {p.attr("id") else "-"} {p.attr("zz") else "-"}")
    print(p.text())
    print("{p.children().len()} {p.elements().len()} {p.attrs().len()}")
    print(p.is_elem())
```

**std.xml.4** — The text may start with a byte order mark, an XML declaration and comments, and may end with
comments; processing instructions (`<? ... ?>`) and comments are dropped wherever they stand. Text that a
comment or an instruction splits is one node: `a<!-- x -->b` is the text `ab`. A comment ends at the first
`-->`. CRLF and a lone CR in the text read as LF. `<![CDATA[ ... ]]>` is text, with no reference
resolved, joined to the text around it. *(ADR 1212)*

```ail rules=std.xml.4
fn main() !
    a := xml.parse("<?xml version=\"1.0\"?><!-- top --><a>x<!-- c -->y<?pi z?><![CDATA[<&>]]>w</a><!-- end -->")?
    print(a.text())
    print(a.children().len())
```

**std.xml.5** — In text and attribute values the references are `&lt; &gt; &amp; &quot; &apos;` and numeric
ones, `&#65;` and `&#x41;` (lower-case `x`). A numeric reference to a character that XML does not allow
(`&#0;`, a surrogate, a value above `U+10FFFF`) and any other name are errors of kind `Entity`, as is a
reference with no `;` within ten characters. A literal tab or line break in an attribute value reads as
a space; the references `&#9;` and `&#10;` keep theirs. *(ADR 1212)*

```ail rules=std.xml.5
fn main() !
    t := xml.parse("<a v=\"1&#10;2\tx\">&#65;&#x42;&lt;&apos;</a>")?
    print(t.text())
    print((t.attr("v") else "").len())
    bad := xml.parse("<a>&nbsp;</a>") else err
        print(err.kind.name())
        t
    print(bad.name())
```

**std.xml.6** — A DOCTYPE is refused with the kind `Doctype`, in the prolog or elsewhere, so no entity is
ever defined by the text and no reference can expand into more than one character: the output is never
longer than the input. Elements nested more than 200 deep fail with the kind `Depth`. The two limits are
the whole defence against entity-expansion and nesting attacks, and no option lifts them. *(ADR 1212)*

```ail rules=std.xml.6
fn main()
    lolz := "<!DOCTYPE lolz [<!ENTITY a \"aaaa\">]><lolz>&a;</lolz>"
    _ := xml.parse(lolz) else err
        print("{err.kind.name()}: {err}")
        return
    print("parsed")
```

**std.xml.7** — `n.find_all(path) List[Node]` follows a slash path from `n`: each step is an element name
or `*` (any element) and selects, from every node so far, the element children it names, in document
order. The last step may be `@name` or `@*`: it selects one text node per matching attribute, holding the
value, so `n.find("a/@id")?.text()` reads an attribute. An `@` step that is not last, an empty path and an
empty step (`a//b`, a leading `/`) select nothing. `n.find(path) Node?` is the first of them. The path is
relative to `n`: it never names `n` itself. *(ADR 1212)*

```ail rules=std.xml.7
fn main() !
    feed := xml.parse("<rss><channel><item id='1'><t>A</t></item><item id='2'><t>B</t></item></channel></rss>")?
    print(feed.find_all("channel/item/t").map(|n| n.text()))
    print(feed.find_all("channel/item/@id").map(|n| n.text()))
    print(feed.find_all("channel/*").len())
    print(feed.find("channel/nothing") == none)
    print(feed.find_all("channel//item").len())
```

**std.xml.8** — A text node that holds only spaces, tabs and line breaks is dropped when its parent has at
least one element child, so an indented document reads as a tree of elements. Where the parent has no
element child, whitespace is kept: `<a> </a>` has the text `" "`. Text beside an element, as in
`one <b>two</b>`, is kept as it stands. *(ADR 1212)*

```ail rules=std.xml.8
fn main() !
    spaced := xml.parse("<a>\n  <b/>\n  <c/>\n</a>")?
    print(spaced.children().len())
    print(xml.parse("<a> </a>")?.text().len())
```

**std.xml.9** — An attribute is `name="value"` or `name='value'`, after at least one space. A value may
not hold a literal `<`. An attribute written twice in one element is an error. Names keep their
spelling, case and any prefix (`std.xml.14`). *(ADR 1212)*

```ail rules=std.xml.9
fn main() !
    e := xml.parse("<a x=\"1\" y='two' z=\"a'b\"/>")?
    print("{e.attr("x") else ""} {e.attr("y") else ""} {e.attr("z") else ""}")
    _ := xml.parse("<a x=\"1\" x=\"2\"/>") else err
        print(err)
        return
```

**std.xml.10** — `escape(s) str` writes `&`, `<` and `>` as `&amp;`, `&lt;` and `&gt;`, which is the text
an element holds. `escape_attr(s) str` does the same and also writes `"` and `'` as `&quot;` and `&apos;`
and tab, line break and carriage return as `&#9;`, `&#10;` and `&#13;`, so the result is safe in a value
quoted either way and in HTML text. Nothing else changes. *(ADR 1212)*

```ail rules=std.xml.10
fn main()
    print(xml.escape("1 < 2 && \"x\""))
    print(xml.escape_attr("it's \"x\"\n"))
```

**std.xml.11** — `write(node, indent) str` is the XML text of `node`, with the references of `std.xml.10`
and an empty element as `<a/>`. With `indent` 0 the result is one line. With `indent` above 0, an
element whose children are all elements puts each child on a line of its own, indented by `indent`
spaces per level, and the end tag on a line of its own; an element with any text child, and an empty
element, stay on one line with their content as it is. There is no final line break. A negative
`indent` traps. `parse(write(n, k)) == n` for a tree `parse` produced. *(ADR 1212)*

```ail rules=std.xml.11
fn main() !
    tree := xml.parse("<a><b x='1'/><c>t &amp; u</c></a>")?
    print(xml.write(tree, 0))
    print(xml.write(tree, 2))
    print(xml.parse(xml.write(tree, 2))? == tree)
```

**std.xml.12** — `write_doc(node, indent) str` is `<?xml version="1.0" encoding="UTF-8"?>`, a line
break, then `write(node, indent)`. *(ADR 1212)*

```ail rules=std.xml.12
fn main()
    print(xml.write_doc(xml.elem("empty", [], []), 0))
```

**std.xml.13** — `XmlError` is an error with `kind XmlKind`, `why str`, `line` and
`col`, the 1-based place where the reader stopped; it displays as `why (line L, column C)`. `XmlKind` is
`Syntax` (not well-formed), `Doctype` (`std.xml.6`), `Entity` (`std.xml.5`), `Mismatch` (an end tag that
does not match, or an element never closed), `Depth` (`std.xml.6`) and `Empty` (no root element). Compare
kinds as variants or by `kind.name()`. *(ADR 1212, type.enum.7)*

```ail rules=std.xml.13
fn main()
    _ := xml.parse("<a>\n  <b>\n</a>") else err
        print("{err.kind.name()} at {err.line}:{err.col}")
        print(err)
        return
```

**std.xml.14** — Namespaces are not interpreted: a prefix is part of the name, `svg:rect` is the tag
`svg:rect`, and `xmlns:svg` is an ordinary attribute. A name starts with a letter, `_`, `:` or any
non-ASCII character and continues with those, digits, `-` and `.`. *(ADR 1212)*

```ail rules=std.xml.14
fn main() !
    g := xml.parse("<svg:g xmlns:svg=\"u\"><svg:rect id=\"r\"/></svg:g>")?
    print(g.name())
    print(g.find("svg:rect/@id")?.text())
```

**std.xml.15** — Text that is not well-formed fails with kind `Syntax` or `Mismatch`: text outside the root
element, content after it, a second root, an unclosed or unmatched tag, an unterminated comment, CDATA
section or instruction, a missing or unquoted attribute value. A document with nothing but a prolog and
comments fails with kind `Empty`. *(ADR 1212)*

```ail rules=std.xml.15
fn main()
    for text in ["<a/><b/>", "<a><b></a>", "x<a/>", "<?xml version=\"1.0\"?>", "<a b=1/>"]
        _ := xml.parse(text) else err
            print("{err.kind.name()}: {err.why}")
            continue
        print("ok")
```

**std.xml.16** — The reader and writer work on whole text in memory; there is no streaming. A document's
size is bounded by memory only. *(ADR 1212)*

```ail rules=std.xml.16
fn main() !
    items := "<item/>".repeat(1000)
    big := xml.parse("<list>{items}</list>")?
    print(big.elements().len())
```
