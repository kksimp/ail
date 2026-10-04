# 0043. Comments use `#`; documentation comments use `##`; there are no block comments

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 2 — Core syntax
- **Spec rules:** [lex.comment.1–3](../spec/02-lexical.md)
- **Evidence:** `#`, `//` and `--` measured identical. `##` and `///` measured identical for doc comments. `--|` cost ~8% more.

## Context

Comment syntax costs the same in every candidate, so readability, fit with an
indentation-based language, and keeping one canonical form decide it.

## Options considered

- **`#` / `##`, line comments only (chosen):** one character and visually
  light. Fits an indentation language, and gives a single canonical form.
  Con: `#` can't be used for attributes or directives.
- **`//` / `///`:** familiar to C-family programmers, and leaves `#` free.
  Con: visually heavier, and a C-family look.
- **`--` / `--|`:** looks like decrement or negation. Doc form costs more.

## Decision

- `#` begins a comment that runs to the end of the line.
- `##` begins a documentation comment, attached to the declaration that follows.
- There are no block comments.

## Consequences

- `#` is unavailable for attributes, annotations or directives. Their
  syntax is OQ-027 (`@` is the natural candidate).
- Doc comments become structured documentation (wishlist 4.2). Their internal
  format is to be designed with the docs tool.
- Commenting out a block means prefixing each line. Editors and the AST edit
  API do this, which keeps comments canonical.

## Rejected alternatives

- **`//`:** heavier, for familiarity only (ADR 0011).
- **`--`:** confusable with operators.
- **Block comments:** a second comment form, and nesting ambiguities.
