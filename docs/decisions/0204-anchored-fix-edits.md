# 0204. The checker names some fix positions by rule, and the tool resolves them

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** AI semantic API
- **Spec rules:** `fix.edit.2`, `fix.edit.3`, `fix.edit.4` (non-normative)

## Context

An edit is `{line, col, end_line, end_col, text}` ([ADR 0145](0145-diagnostics-schema-v1.md)). The AST has no end positions, so the checker can't say where a block or an expression ends. Lane 1 solved the same problem for a word with an anchor: "the first whole-word `old` at or after `pos`", resolved against the source when the diagnostic is reported.

## Decision

Add two more anchors, resolved in the same place (`diag::settle`), so the output schema stays api 1 (an edit is always plain coordinates):

- **After block**: insert whole lines after the block below `pos` (the last following line that is blank or indented deeper than `pos`'s column). Used to add match arms.
- **Line end**: insert at the end of the code on `pos`'s line. Dropped if the line has a `#` or doesn't end like a complete expression (`)`, `]`, a word), so a multi-line call never gets a misplaced `?`.

A fix whose anchor doesn't resolve is dropped, as before.

## Rationale

Tokens and local reasoning (#1, #2): no AST changes (ten lanes edit `ast.rs` and `parser.rs`), and the resolved edit is checkable text.

## Consequences

- Line-end is conservative: a call spread over lines gets no `?` fix.
- Recording end positions in the AST would remove the anchors; a later change can do that without changing the schema.

## Rejected alternatives

- **End spans in every AST node.** Large cross-cutting change during a ten-lane merge.
- **Token-based re-parse in `ail fix`.** Duplicates the checker's knowledge.
