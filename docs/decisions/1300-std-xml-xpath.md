# 1300. `std.xml`: XPath 1.0 queries, namespace-aware names and a pull reader

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 — Standard library
- **Spec rules:** [std.xml.17 to std.xml.26](../spec/std-xml.md); amends std.xml.14 and std.xml.16 ([ADR 1212](1212-std-xml.md))

## Context

ADR 1212 gave `std.xml` a tree, slash paths and a writer, and left three gaps its rejected-alternatives list names:
no real query language (slash paths cannot filter, climb or compare), no namespace handling, and no way to read a
document too large for a tree. Agents asked for these on feeds with `atom:` and `media:` prefixes, SOAP and
multi-gigabyte exports. XPath 1.0 is what every XML tool, and every model, already knows.

## Decision

- **XPath 1.0 subset, written in AIL, pure.** `Node.xpath`, `xpath_ns`, `select`, `select_ns` return an `XValue`
  (`Nodes`, `Str`, `Num`, `Bool`) or a typed `XPathError`. All twelve axes except `namespace`, every node test,
  predicates, the XPath 1.0 operators and comparison rules, and 25 core functions. The tree has no parent pointers, so
  each query first lays the queried tree out as an indexed table in document order; axes, positions and node-set
  union then follow from indexes, and a node-set comes back in document order without repeats by construction.
- **The queried node is its own document.** Nothing is above, beside or after it, so `/` and `..` are bounded by the
  node the call is on. A reader who needs the whole document queries the root.
- **Namespaces only where asked.** `xpath_ns` takes bindings as a `List[Attr]` (prefix, URI) and resolves
  each element's namespace from the `xmlns` attributes in the queried tree. Plain `xpath` and `find` keep comparing
  names as written, so std.xml.14 and every existing program are unchanged. `xmlns` declarations are not attributes
  of the attribute axis, as XPath 1.0 has it.
- **Fail at parse time where possible.** An unknown function, an unknown axis, a wrong argument count and bad syntax
  are found before evaluation, with the character they stop at; only an unbound prefix and a non-node-set operand
  are found while evaluating.
- **`Reader`: a pull reader over the text.** `xml.reader(text)`, `next()` giving `StartTag`, `EndTag` and `Chars`,
  `depth()`, `read_element()` (builds one element as `parse` would) and `skip()`. It shares the tokenizer of
  `parse`, so the same documents are refused with the same errors and positions. A whitespace-only run is dropped
  unless it is an element's whole content, which gives the events of exactly the nodes `parse` keeps.
- **Test oracle.** The expected output of every XPath and namespace test line is lxml's answer to the same expression
  and bindings (the generator and its Python mirror are `std-xml/gen.py` in the project files), and the reader's
  event streams are lxml's tree under the same whitespace rule. Where lxml and the spec differ (a number such as
  `1e3`, which libxml2 accepts), the test leaves the case out and the spec says what std.xml does.

## Rejected alternatives

- **Compile paths to closures.** More code for no measured need; a query reads a tree once, and the indexed table is
  built in one pass.
- **Parent pointers in `Node`.** `Node` is a plain value (`==`, copies, `match`); a parent link makes a cycle.
- **`select_one` returning `Node? ! XPathError`.** Two ways to be empty; `select(e)?.first()` says it.
- **XPath 2.0 or 3.1.** Sequences, a type system and a function library of hundreds; XPath 1.0 answers the questions
  feeds and configs ask. A later ADR can add `fn:matches` style helpers if measurements want them.
- **`id()`, `lang()` and the `namespace` axis.** `id()` needs a DTD, `lang()` and `namespace::` are rare; each is an
  `Unsupported` error naming the feature, not a wrong answer.
- **A SAX callback API.** Callbacks cannot stop reading at the first match and cannot hand one element to a
  function; pulling events can.
- **Streaming from a file or socket.** Needs a chunked refill in the tokenizer and a decision on where a multi-byte
  character or a name may split; left to a follow-up. The reader's memory is the text plus the open names.
- **A pretty writer with its own options.** `write(node, indent)` (std.xml.11) already indents element-only content;
  mixed content cannot be re-indented without changing its text, so no new options.

## Consequences

`std/xml/xml.ail` grows by the query engine and the reader (about 1,100 lines; the file stays one module and needs no
new registration line). Three new test files, `tests/cases/std_xml_xpath`, `std_xml_ns` and `std_xml_pull`, ten spec
rules with runnable examples. No syntax change, no checker or runtime change. A query costs one pass over the queried
subtree to build the table plus the work of the expression; for one lookup in a large tree, `find` is still cheaper.
