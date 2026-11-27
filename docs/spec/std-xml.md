# Standard library: `std.xml`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR 1212; XPath and the pull reader, ADR TBD).
> Reads and writes XML text for feeds, vector graphics and property lists, queries it with slash paths or
> XPath 1.0 (`std.xml.17` to `std.xml.22`) and reads it event by event (`std.xml.23` to `std.xml.26`).
> Schemas, XSLT, XPath 2.0 and later, and DOCTYPEs are out of scope (`std.xml.6`, `std.xml.20`).

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

**std.xml.14** — `parse`, `find` and `find_all` do not interpret namespaces: a prefix is part of the name,
`svg:rect` is the tag `svg:rect`, and `xmlns:svg` is an ordinary attribute. `xpath_ns` and `select_ns`
do (`std.xml.21`). A name starts with a letter, `_`, `:` or any
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

**std.xml.16** — `parse`, `write` and the XPath methods work on a whole document in memory as a tree; a
document's size is bounded by memory only. A document too large for a tree is read with `xml.reader`
(`std.xml.23` to `std.xml.26`). *(ADR 1212)*

```ail rules=std.xml.16
fn main() !
    items := "<item/>".repeat(1000)
    big := xml.parse("<list>{items}</list>")?
    print(big.elements().len())
```

## XPath

**std.xml.17** — `n.xpath(expr) XValue ! XPathError` evaluates the XPath 1.0 expression `expr` with `n` as
the context node, and the tree below `n` as the whole document: `/` is `n`'s own document node, whose one
child is `n`, and nothing is above, beside or after `n`. `enum XValue` is `Nodes(nodes List[Node])`,
`Str(value str)`, `Num(value float)` or `Bool(value bool)`. A node-set is in document order with no
repeats; an attribute is a text node holding its value (`std.xml.7`). `n.select(expr) List[Node] !
XPathError` is the node-set of an expression that has one, and the error kind `BadType` otherwise.
`v.string()` is the XPath string of a value: the first node's text, a number without a trailing `.0`,
`true` or `false`. *(ADR TBD)*

```ail rules=std.xml.17
fn main() !
    feed := xml.parse("<rss><item id='1'><t>A</t></item><item id='2'><t>B</t></item></rss>")?
    print(feed.select("//item[@id='2']/t")?.map(|n| n.text()))
    print(feed.xpath("count(//item)")?.string())
    print(feed.xpath("//t = 'B'")?.string())
```

**std.xml.18** — A location path is absolute (`/a`, `//a`, `/`) or relative to the context node, with steps
separated by `/`; `//` is `/descendant-or-self::node()/`. A step is `axis::test` with the predicates after
it, or `.` (`self::node()`), `..` (`parent::node()`), `@name` (`attribute::name`) or a bare test (`child::`).
The axes are `child`, `descendant`, `descendant-or-self`, `parent`, `ancestor`, `ancestor-or-self`,
`following-sibling`, `preceding-sibling`, `following`, `preceding`, `self` and `attribute`. A test is a name,
`*`, `p:*`, `text()` or `node()`; `comment()` and `processing-instruction()` are accepted and select nothing,
because the tree has neither (`std.xml.4`). The `xmlns` and `xmlns:p` declarations are not attributes of the
attribute axis. *(ADR TBD)*

```ail rules=std.xml.18
fn main() !
    t := xml.parse("<a><b id='1'><c/></b><b id='2'/><d/></a>")?
    print(t.select("//c/ancestor::*")?.map(|n| n.name()))
    print(t.select("b[1]/following-sibling::*")?.map(|n| n.name()))
    print(t.select("//@id")?.map(|n| n.text()))
    print(t.select("//d/preceding::b")?.map(|n| (n.attr("id") else "")))
```

**std.xml.19** — A predicate `[e]` keeps the nodes for which `e` is the number equal to their position on the
step's axis (nearest first on a reverse axis), or otherwise a true value (`boolean()` of it). `[1]` binds
to each step, and `(//a)[1]` is the first of all. The operators are `or`, `and`, `=`, `!=`, `<`, `<=`, `>`,
`>=`, `+`, `-`, `*`, `div`, `mod`, unary `-` and `|`, with their XPath 1.0 precedence. A comparison with a
node-set is true when it holds for some node (so `a = 1` and `a != 1` can both be true); a boolean
compares as a boolean, then a number as a number, else strings; `<` and its kin compare numbers. `div`
by zero is infinity and `0 div 0` is `NaN`; `mod` keeps the sign of its left side. A string is a number
when it is an optional `-` and digits with at most one point, else `NaN`. `a|b` needs two node-sets. *(ADR TBD)*

```ail rules=std.xml.19
fn main() !
    t := xml.parse("<a><n>3</n><n>10</n><n>7</n></a>")?
    print(t.select("n[. > 5][2]")?.map(|x| x.text()))
    print(t.select("n[position() = last()]")?.map(|x| x.text()))
    print(t.xpath("n = 10")?.string())
    print(t.xpath("n != 10")?.string())
    print(t.xpath("7 mod 4 + 5 div 2")?.string())
```

**std.xml.20** — The functions are `last`, `position`, `count`, `string`, `number`, `boolean`, `not`, `true`,
`false`, `local-name`, `name`, `namespace-uri`, `concat`, `starts-with`, `contains`, `substring-before`,
`substring-after`, `substring`, `string-length`, `normalize-space`, `translate`, `sum`, `floor`, `ceiling`
and `round`, as XPath 1.0 defines them (`round` takes halves toward positive infinity; `normalize-space`
splits on any Unicode white space). `id`, `lang`, the `namespace` axis, variables and processing-instruction
targets are not supported: a name that is not in the list is an error of kind `Unsupported`, and so is
a function called with the wrong number of arguments (kind `BadPath`). *(ADR TBD)*

```ail rules=std.xml.20
fn main() !
    t := xml.parse("<a><w>  one  two </w></a>")?
    print(t.xpath("normalize-space(w)")?.string())
    print(t.xpath("concat(substring-before('k=v', '='), '|', translate('abc', 'ab', 'AB'))")?.string())
    print(t.xpath("round(2.5) + floor(-0.5)")?.string())
    _ := t.xpath("id('x')") else err
        print(err.kind.name())
        return
```

**std.xml.21** — `n.xpath_ns(expr, bindings)` and `n.select_ns(expr, bindings)` match names by namespace.
`bindings` is a `List[Attr]` whose `name` is a prefix and whose `value` is a URI. A prefixed name test
`p:name` matches elements (or attributes, on the attribute axis) in the URI bound to `p` whose local name is
`name`; an unprefixed test matches names in no namespace, as XPath 1.0 says, however the document spells
the default namespace. A prefix that is not bound is an error of kind `BadPrefix`. The namespace of an element
is found from the `xmlns` and `xmlns:p` attributes of it and its ancestors in the queried tree (the prefix
`xml` is built in); an unprefixed attribute has none. `namespace-uri()` and `local-name()` read it. Without
bindings `xpath` and `select` compare a name as written, so `svg:rect` is the name `svg:rect`
(`std.xml.14`). `n.prefix()` and `n.local_name()` split a tag or attribute name at its colon. *(ADR TBD)*

```ail rules=std.xml.21
fn main() !
    doc := xml.parse("<f:feed xmlns:f='urn:feed' xmlns='urn:d'><f:item><t>x</t></f:item></f:feed>")?
    ns := [xml.Attr(name: "f", value: "urn:feed"), xml.Attr(name: "d", value: "urn:d")]
    print(doc.select_ns("//f:item/d:t", ns)?.map(|n| n.text()))
    print(doc.select_ns("//t", ns)?.len())
    print(doc.xpath_ns("namespace-uri(//d:t)", ns)?.string())
    print(doc.select("f:item")?.len())
    print("{doc.prefix()} {doc.local_name()}")
```

**std.xml.22** — `XPathError` is an error with `kind XPathKind`, `why str` and `pos int`, the 0-based
character of the expression where the reader stopped (0 for a fault found in a value). `XPathKind` is
`BadPath` (not valid XPath 1.0, or a wrong argument count), `Unsupported` (`std.xml.20`), `BadPrefix`
(`std.xml.21`) and `BadType` (a step, `|`, `count` or `sum` on a value that is not a node-set; `select`
on an expression that is not one). *(ADR TBD)*

```ail rules=std.xml.22
fn main() !
    t := xml.parse("<a/>")?
    for expr in ["//a[", "count(1)", "foo()", "1 +"]
        _ := t.xpath(expr) else err
            print("{err.kind.name()}: {expr}")
            continue
        print("ok")
```

## Reading as events

**std.xml.23** — `xml.reader(text) Reader` makes a reader that reads nothing until asked, and
`r.next() XmlEvent? ! XmlError` gives the next event, `none` after the root element's end tag. `enum
XmlEvent` is `StartTag(tag str, attrs List[Attr])`, `EndTag(tag str)` and `Chars(content str)`. An empty element
`<a/>` is a `StartTag` and then an `EndTag`. The text is read as `parse` reads it: the prolog, comments and
instructions are skipped, a reference is resolved and CDATA joined to the text around it, and text that a
comment splits is one `Chars`. A whitespace-only run is not reported unless it is the whole content of an
element (`<a> </a>`): the events are the nodes of `parse` with the same text. *(ADR TBD)*

```ail rules=std.xml.23
fn main() !
    mut r := xml.reader("<?xml version='1.0'?><a x='1'>hi<b/> </a>")
    while true
        got := r.next()?
        ev := got else break
        match ev
            StartTag(tag, attrs) => print("start {tag} {attrs.len()}")
            EndTag(tag) => print("end {tag}")
            Chars(content) => print("chars {content}")
```

**std.xml.24** — `r.depth() int` is how many elements are open. After a `StartTag`, `r.read_element() Node !
XmlError` reads the rest of that element and returns it as `parse` would have built it, leaving the reader
after its end tag, and `r.skip() ! XmlError` reads past its end tag without building it. At any other time
each is an error of kind `Syntax`. Reading a long list one element at a time, with `read_element` on each
`item`, holds one item in memory at once. *(ADR TBD)*

```ail rules=std.xml.24
fn main() !
    mut r := xml.reader("<list><item>a</item><skip><x/></skip><item>b</item></list>")
    mut seen := List[str]()
    while true
        got := r.next()?
        ev := got else break
        match ev
            StartTag(tag, _) =>
                if tag == "item"
                    seen.push(r.read_element()?.text())
                elif tag == "skip"
                    r.skip()?
            _ => print("-")
    print(seen)
```

**std.xml.25** — The reader fails where and how `parse` fails (`std.xml.13`, `std.xml.15`): the same kinds,
messages, lines and columns, found when the reader reaches them, not before. A reader that has returned an
error is not to be used again. *(ADR TBD)*

```ail rules=std.xml.25
fn main()
    mut r := xml.reader("<a><b></a>")
    while true
        step := r.next() else err
            print("{err.kind.name()}: {err}")
            return
        if step == none
            break
```

**std.xml.26** — The reader holds the text, not a tree: its memory is the text plus one open element's
name per level, and nesting is limited to 200 as in `std.xml.6`. The text is a `str` held whole; reading
from a file or a socket as it arrives is not provided. *(ADR TBD)*

```ail rules=std.xml.26
fn main() !
    mut r := xml.reader("<a>".repeat(201))
    mut n := 0
    while true
        got := r.next() else err
            print("{err.kind.name()} after {n}")
            return
        if got == none
            break
        n += 1
```
