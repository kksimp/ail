# 0152. A doc comment is the run of `##` lines directly above a declaration

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** api.doc.1-2 ([semantic-api.md](../design/semantic-api.md#doc-comments)); refines lex.comment.2

## Context

lex.comment.2 says `##` documents "the declaration that immediately
follows", without saying what breaks a run or which declarations take docs.

## Options considered

### Option A — contiguous `##` lines ending on the line above (chosen)
- **Pros:** one reading only; a blank line or a `#` comment detaches a
  stray doc, so a doc can't land on the wrong declaration.
- **Cons:** a blank line between doc and declaration silently drops the doc
  (the formatter can warn later).

### Option B — skip blank lines between the run and the declaration
- **Cons:** a doc for something deleted attaches to whatever follows.

## Decision

A doc comment is a run of `##` lines with nothing between it and the
declaration's line; the doc is each line's text after `##` and one
optional space, joined with newlines. Structs, fields, enums, variants,
traits, functions, methods and constants take docs. The bootstrap reads them
from the source text (`docs.rs`) into a `doc` field on those AST nodes,
since the lexer drops comments.

## Rationale

AI understanding: an exact, local rule, with no doc ever attached at a
distance. Reading the source again kept the lexer and parser unchanged
apart from the new field.

## Consequences

- `ail api` and later tools (`ail fmt`, hover) share one attachment rule.

## Rejected alternatives

- Skipping blank lines: attaches orphaned docs to the wrong declaration.
- Doc tokens in the token stream: touches every parser path for no gain.
