# 0310. `ail api --card`: a program's own card

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** [api.card.1–2, api.card.4–5](../design/semantic-api.md#program-card)
- **Extends:** [ADR 0150](0150-ail-api-outline.md)

## Context

An agent joining a project reads the language card for the language and
then has to read source for the project. `ail api` outlines one file; a
package has many modules, tests, a manifest and examples. Deciding
criteria: **tokens** first, then **AI understanding** (one read, no
guessing).

## Options considered

### Option A — one Markdown card per package or file (chosen)
`ail api --card DIR|FILE` prints the program's name, entry point, test
names, `ail.pkg` lines, examples with their first doc line, then one fenced
block per module with each declaration's signature, effects in `[...]`,
contracts and error sets under it, and a one-line `##` doc after it. Items
are the outline of `ail api --text` (same signatures, ADR 0150), nested the
same way, so a reader who knows one knows the other. `--json` prints the
same content for tools (the editor may show it).

### Option B — generate full documentation (HTML or long Markdown)
- **Cons:** bodies of docs, examples and cross-links cost many times the
  tokens and the reader still has to find the signatures.

### Option C — `ail api --text` over every file, concatenated
- **Cons:** no entry point, tests, manifest or examples; private helpers
  fill the budget; no ordering across modules.

## Decision

Option A. Order is deterministic: header lines, then the root module, then
modules by import path, each in source order. Test functions (`@test`) are
listed by name in the header and left out of the module blocks. The card of
a program the checker rejects is the diagnostics (exit 2), as for
`ail api` (api.out.5). `docs/design/cards/` holds the cards of `tools/edit`
and `bootstrap/src/std`, and a test fails when they are stale
(api.card.5).

## Rejected alternatives

Options B and C above; a YAML header (more tokens than plain lines).
