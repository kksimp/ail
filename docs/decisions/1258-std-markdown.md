# 1258. `std.markdown`: CommonMark and GFM tables to a tree and to escaped HTML

- **Status:** Accepted (the number is assigned by the integrator; rename this file and add the index row then)
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the std library to be filled out overnight on 2026-11-27
- **Topic:** 17 — Standard library; AI understanding
- **Spec rules:** std.markdown.1 to std.markdown.16 in [std-markdown.md](../spec/std-markdown.md)
- **Extends:** [ADR 1126](1126-std-modules-in-ail-without-use.md)

## Context

Agents that write READMEs, docs servers, chat output or static sites need Markdown to HTML, and today they
hand-roll a regex converter that mishandles nesting and lets raw HTML and `javascript:` links through. The std
plan lists it as a pure text module, so it is written in AIL with no runtime or checker work.

## Decision

A new module, `std/markdown/markdown.ail`, in scope without `use` (std.use.4).

- `parse(text) List[Block]` is total: CommonMark has no invalid input, so there is no error type.
- `Block` and `Inline` are plain enums, so a program matches, edits and renders a tree.
- `html(text)` and `render(blocks)` escape everything. Raw HTML in the text is shown as text; links and images
  whose scheme is not `http`, `https`, `mailto`, `tel`, `ftp` or `sms` (or relative) are dropped to their text.
- GFM tables, `~~strike~~` and bare `http(s)://` links are included, since READMEs use them.
- `plain` and `headings` cover the two reads agents ask for most (text for search or a summary, an outline).
- Nesting is capped at 32 so no input recurses without bound.
- Emphasis uses the CommonMark delimiter-run algorithm, including the rule of 3.

Block parsing is a line-oriented recursive reader. Quotes and list items collect their lines and parse them again,
which is simple to check against the spec examples but approximates a few exotic CommonMark corner cases
(std.markdown.15).

## Rejected alternatives

- **Raw HTML pass-through with a sanitiser:** a second parser with its own bugs; escaping cannot leak a script.
- **A visitor or callback API:** plain enums need no new concept.
- **Link reference definitions now:** they need a whole-document pre-pass; deferred, listed in std.markdown.15.
- **An error type:** nothing can fail.
- **A runtime (native) implementation:** Markdown is text work, not speed-critical; AIL keeps arm64 and x86-64 identical.

## Consequences

The first compile is the Mac gate: the module and its goldens were written and hand-traced without a build.
Follow-ups: reference links, HTML blocks as an opt-in `raw` flag, heading ids for anchors.
