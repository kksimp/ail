# 0338. `ail api --card --json` gives each declaration's position

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling — the semantic API ([semantic-api.md](../design/semantic-api.md), rule api.card.2)
- **Spec rules:** none (tool output)
- **Amends:** [ADR 0331](0331-editor-program-card-view.md) (Option B, its recorded follow-up)

## Context

The editor's program card (Ctrl-Shift-I, ADR 0331) found each row's
declaration by matching its signature text against the sources, which can
miss an unusual layout. Agents reading the card JSON had the same gap:
they could see what a program declares but not where.

## Options

### Option A — `file`, `line`, `col` on every JSON item (chosen)
Each item in `--card --json` gains `file` (as the program names it),
`line` and `col` (1-based, the declaration's name position, the same
convention as `ail check --json`). The Markdown `--card` text is
unchanged, so its token budget and the committed cards don't move.
- **Pros:** exact jumps; the same position convention as diagnostics; no
  cost to the text card agents read most.
- **Cons:** about 20 more bytes per item in the JSON.

### Option B — positions in both forms
- **Cons:** the text card exists to be small; positions there cost tokens
  for every reader.

### Option C — keep matching text
- **Cons:** misses layouts it doesn't expect; every client repeats it.

## Decision

Option A. The editor uses the positions and keeps text matching only as a
fallback for an older `ail` without them.

## Rejected alternatives

- **B:** costs the text card's tokens for a need only tools have.
- **C:** fragile and duplicated per client.
