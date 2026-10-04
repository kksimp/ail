# 0040. The indentation unit is four spaces; tabs are not allowed in indentation

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 2 — Core syntax
- **Spec rules:** [lex.block.2, lex.block.4](../spec/02-lexical.md)
- **Resolves:** OQ-026
- **Evidence:** [syntax-measurements-r1.md](../design/syntax-measurements-r1.md#addendum-visible-indentation-markers-designers-question), [indentation-examples.md](../design/indentation-examples.md)

## Context

[ADR 0033](0033-indentation-blocks.md) made indentation significant and
required a single unit. Measured costs relative to 4 spaces: 2 spaces −0.1%,
one period per level +0.5%, tabs +6.0%. The designer reviewed visual examples
of each option.

## Options considered

### Option A — 4 spaces (chosen)
- **Pros:** The clearest nesting, at the same token cost as 2 spaces.
  Editors insert it automatically and can draw guides. Universal tool support.
- **Cons:** Deep nesting runs wide. Some chat and web tools collapse spaces.

### Option B — 2 spaces
- **Pros:** Compact.
- **Cons:** Levels are harder to tell apart.

### Option C — Tabs
- **Pros:** Display width is each reader's choice.
- **Cons:** +6% tokens, inconsistent display, and confusable with spaces.

### Option D — Periods, one per level (designer's idea)
- **Pros:** Visible, and never collapsed by web tools.
- **Cons:** Clashes with `.` and `..`, has no tool support, and is noisier.

## Decision

Each indentation level is exactly four space characters (U+0020). Tab
characters are not permitted in indentation.

## Consequences

- Because tabs can't appear in indentation, the tabs-vs-spaces mixing hazard
  can't occur.
- Editors and `ail fmt` insert four spaces when Tab is pressed. The language
  server can offer indentation guides.
- Whether tabs may appear elsewhere (e.g. in string literals or alignment
  inside comments) is decided with the lexical grammar. Inside string
  literals they would normally be written as an escape.

## Rejected alternatives

- **2 spaces:** weaker visual structure for no token gain.
- **Tabs:** more tokens and display inconsistency.
- **Periods:** collide with existing operators and lack tooling.
