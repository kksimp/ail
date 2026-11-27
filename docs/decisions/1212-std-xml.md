# 1212. `std.xml`: read, search, write and escape XML

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 — Standard library
- **Spec rules:** [std.xml.1 to std.xml.16](../spec/std-xml.md)

## Context

Agents meet XML in RSS and Atom feeds, SVG, property lists, Maven and Android files, and SOAP. `ail ask std xml`
answered "not in std" (`docs/design/not-in-std.txt`), so each program carried its own reader. The std-gap plan
(`std-gap-plan.md`, lane 10) asks for parse, path queries, write and escape, with no entity expansion.

## Decision

- A new module `std.xml`, written in AIL (`std/xml/xml.ail`), pure, no effects. Fresh code: nothing is copied from
  other projects.
- `parse(text) Node ! XmlError` returns the root element. `Node` is an enum, `Elem(tag, attrs, kids)` or
  `Text(content)`, so a tree is a plain value (copies are independent, `==` works) and a `match` covers both.
- Methods: `name`, `text`, `attr`, `attrs`, `children`, `elements`, `is_elem`, `find`, `find_all`.
- A path is `a/b`, with `*` for any element and a last step `@name` or `@*`. An attribute step yields text nodes, so
  one method set (`text()`) reads both elements and attributes and `find` keeps one return type.
- Comments and processing instructions are dropped; CDATA becomes text; whitespace-only text beside elements is
  dropped. CRLF reads as LF, and a literal tab or line break in an attribute value as a space.
- References: the five predefined entities and numeric references only. A DOCTYPE is refused, so nothing can define
  an entity and the output is never longer than the input. Nesting is limited to 200 levels.
- `write(node, indent)` and `write_doc`, `escape` (`& < >`) and `escape_attr` (adds quotes and control whitespace);
  `escape_attr` is also the HTML escape agents ask for.
- `XmlError` carries `kind` (an enum, `std.err.1`), `why`, `line` and `col`.

## Rejected alternatives

- **Accept DOCTYPE and cap expansion.** Needs an entity table, a size budget and external-entity rules; refusing it
  costs nothing for feeds, SVG and plists, and removes the whole attack class.
- **A struct `Node` with `tag`, `attrs`, `kids` and `text`.** It cannot say a text child apart from an element
  without a flag or a second list, and loses the order of mixed content.
- **Attribute lookup as a separate `find_attr`.** A second search function for one extra step; `@name` in the path
  covers it with the same function.
- **Namespace resolution.** Needs scope tracking and a second name form (`{uri}local`); prefixes stay part of the
  name (`std.xml.14`), which is enough for the formats above.
- **Streaming (SAX or pull) reader.** Out of scope; a document is read whole (`std.xml.16`). A follow-up lane can add one.
- **Keeping whitespace text and comments.** Every indented file would parse into a tree full of blank text nodes
  that each caller has to skip.

## Consequences

One new file under `std/`, one spec chapter, registration lines in the compiler's module lists, and tests
`tests/cases/std_xml*.ail`. No syntax change and no runtime or checker change.
