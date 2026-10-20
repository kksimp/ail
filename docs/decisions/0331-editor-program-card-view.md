# 0331. Editor program card view (Ctrl-Shift-I)

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling — the editor ([ADR 0180](0180-editor-written-in-ail.md))
- **Spec rules:** none (tool behaviour); reads `ail api --card --json` (lane 67)
- **Extends:** [ADR 0312](0312-editor-agent-panel.md)
- **Amended by:** [ADR 0338](0338-card-json-positions.md) (rows jump by the card's own positions)

## Context

`ail api --card` summarises a whole program for an agent: entry point,
tests, and each module's declarations with effects, contracts and error
sets. The editor should show the same view and let the reader go from an
entry to its code.

## Options

### Option A — the card as a list in the pane, rows jump to declarations (chosen)
Ctrl-Shift-I runs `ail api --card --json --budget 100000 FILE` on the
active file, which covers its whole program (the modules beside it), so
nothing is dropped for size in the editor. Rows: the entry point, the
tests, then per module each item's signature with `[effects]`, its
contracts and `fails with` set indented, and members (fields, variants,
methods) below; a variant's fields are in its signature, so they aren't
repeated. The card JSON has no positions, so a row finds its declaration
by matching the signature against the program's files (the active file
first, buffers as edited; a constant by `NAME := `); members without a
match use their parent's line. The rows use the list of ADR 0292 (Up/Down,
Enter jumps, Esc closes) and are rebuilt on save while shown.
- **Pros:** the agent's view of the program is the person's too; no new
  API; one navigation model.
- **Cons:** position lookup by text can miss an unusual layout (the row
  then doesn't jump).

### Option B — add positions to `ail api --card --json`
- **Pros:** exact jumps.
- **Cons:** a change to lane 67's format past this round's cutoff; the
  card is meant to stay small for agents. Recorded as a follow-up.

### Option C — the Markdown card as plain text
- **Cons:** no jumps, no structure.

## Decision

Option A now; Option B is the natural follow-up when `--card --json`
next changes.

## Rejected alternatives

- **C:** no navigation.
