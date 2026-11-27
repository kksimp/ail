# TBD. `std.html`: read HTML like a browser, query it, write it and sanitize it

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 — Standard library
- **Spec rules:** [std.html.1 to std.html.20](../spec/std-html.md)

## Context

Agents scrape pages, pull links and text out of HTML, render user content and must clean it before
showing it. `std.xml` refuses most real HTML (unclosed `<p>`, `<br>`, unquoted attributes, `&nbsp;`), so
each program carried its own regex-based reader, which is wrong on real pages and unsafe for cleaning.
The overnight std refill asks for an HTML5 reader, CSS-selector-lite queries, text extraction, entity
escape and unescape, and an allowlist sanitizer. The garlic browser's HTML code is AGPL; std is Apache
2.0, so this is fresh code.

## Decision

- A new module `std.html`, written in AIL (`std/html/html.ail`), pure, no effects.
- `parse(text) Node` never fails and always returns `html` holding `head` and `body`, as a browser's DOM
  does; `parse_fragment(text) List[Node]` reads body content with no wrapper. `Node` is an enum,
  `Elem(tag, attrs, kids)`, `Text(content)` or `Comment(content)`, the same shape as `std.xml`'s plus
  comments, so the two modules read alike and a tree is a plain value.
- The tokenizer follows the WHATWG tokenizer: tag and attribute forms, comments and bogus comments, raw
  text and RCDATA elements, `plaintext`, and character references with the HTML 4 names, the legacy
  no-semicolon rule, the attribute-value exception and the numeric replacements.
- The tree builder follows the WHATWG "in body" rules that decide the shape of everyday pages: block
  tags close `p`, list items and definition terms close their siblings, void elements, `</p>` and
  `</br>`, end tags that close nested elements or are ignored, implied `tbody` and `tr`, and head/body
  placement. A stack of open elements over an index arena builds the tree, then converts it to values.
- Queries are CSS selectors (descendant and child combinators, type, `*`, `#id`, `.class`, the seven
  attribute operators, comma lists), because agents already know them; `select` and `select_one` fail
  with `SelectorError` (`kind` an enum) on a selector they cannot read.
- `to_text` gives the text a reader sees (collapsed whitespace, block line breaks, cells tab-separated),
  which is what agents want from a page; `Node.text()` stays the raw concatenation.
- `sanitize(text)` and `sanitize_with(text, Policy)`: parse as a fragment, keep allowlisted elements and
  attributes, unwrap the rest, drop script-like elements with their content, always drop `on*` and
  `srcset`, check URL schemes after removing control characters and spaces, and write the cleaned tree
  with `write`, so the output is well formed and escaped.

## Rejected alternatives

- **The full WHATWG tree builder (adoption agency, active formatting reconstruction, foster
  parenting, foreign content).** Several times the code for cases that misnest inline tags or put text
  in a table outside its cells; the simplified rules keep all text and give the same tree for valid and
  usual pages. A later slice can add them without changing the API (`std.html.11`).
- **`parse` returning `Node ! HtmlError`.** Browsers never refuse HTML; an error path would only be a
  `?` every caller writes for nothing.
- **Reusing `std.xml.Node`.** XML has no comments in its tree and its writer emits `<br/>`; one type
  would tie the two modules' rules together.
- **The full 2,231-name WHATWG entity table.** About 40 KB of names that pages almost never use; the
  HTML 4 set covers what real pages write, and an unknown name stays as written, which loses nothing.
- **An XPath-like path, as in `std.xml`.** HTML authors and agents query with CSS selectors.
- **Selectors that match nothing when invalid, or trap.** A typo would read as "no results"; a trap
  stops a scraper on untrusted input. An error value names the place.
- **Sibling combinators and pseudo-classes.** They need sibling state in the match; refused with kind
  `Unsupported` so a caller knows rather than getting wrong results.
- **A sanitizer by escaping or by a denylist.** Escaping loses formatting the caller wants; denylists
  miss new attack vectors. An allowlist that rewrites from the tree is the usual safe design.
- **Keeping the text of dropped elements.** The text of `script` or `style` is code, not content.

## Consequences

One new file under `std/`, one spec chapter, registration lines in the compiler's module lists, one
language card line, and tests `tests/cases/std_html*.ail`. No syntax change and no runtime or checker
change.

## Conformance pass

The html5lib tree-construction suite (1516 whole-document cases; fragment and scripting cases left out) ran through
the Python mirror of the module (`/mnt/project-files/std-html/mirror.py`, `tc_run.py`): 782 passed before and 995
after (52% to 66%). Cases that use SVG, MathML, `template` and `frameset` (391 of the 1516) are out of scope by
std.html.11 and pass 4 of them. Of the cases outside those three, 991 of 1125 pass (88%); the rest are the adoption
agency (`formatting`, 76 failing) and the table cases that depend on it. Fixed from the suite: comment and
white-space placement around `head` and `body`, script data escapes, NUL handling, the end of `script` at the end of
the text, foster parenting for tables, quirks mode for `<table>` and `<p>`, `select`, ruby and `dialog`, `noframes` in
`head`, and the HTML 5 values of `&lang;` and `&rang;`. `tests/cases/std_html_conformance.ail` keeps 190 of the
passing cases with the suite's own expected trees, never generated from the module's output.

Rejected for now: the full HTML 5 reference table (2231 names, about 30 KB of source for the rare names), the adoption
agency algorithm (a rewrite of the open-element handling), and comments before `<html>` as part of the returned tree
(`parse` returns the `html` element, so they are dropped).
