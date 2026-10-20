# 0291. The editor's command palette and README key table come from one list

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling — the editor ([ADR 0180](0180-editor-written-in-ail.md))
- **Spec rules:** none (tool behaviour)
- **Extends:** [ADR 0241](0241-editor-tabs-and-running.md)

## Context

The editor has about 45 keys. A person or an agent has to find them, and
the README's key table had already drifted twice from the code (Ctrl-Shift-F,
F2). Project search wants Ctrl-Shift-F, which formatted.

## Options

### Option A — one list of key rows, the palette and the README generated from it (chosen)
`key_rows()` in `edit.ail` holds each row's keys as shown, the palette
name, the key it presses (`ctrl+shift+f` form) and the description.
`edit.ail --keys` prints the README table; a selftest fails when the README
differs. The palette (Ctrl-Shift-P) lists the rows with a key, filtered as
you type (letters in order); Enter presses that key, so a palette command
runs exactly what its key runs.
- **Pros:** one place to add a key; the docs can't drift; no second dispatch.
- **Cons:** the README table is pasted output, not hand-written prose.

### Option B — a palette with its own command enum
- **Cons:** a second dispatch path to keep in step with the keys.

### Option C — keep the hand-written table, check only key names
- **Cons:** descriptions still drift; the check is weaker than generation.

## Decision

Option A. Formatting moves from Ctrl-Shift-F to **Alt-Shift-F** (the
common format chord) so Ctrl-Shift-F is free for project search
([ADR 0292](0292-editor-project-search-and-file-tree.md)). A test checks
every palette row's key reaches a handler and palette names are unique.

## Consequences

- New keys are added to `key_rows()`; `--keys` regenerates the table.
- Workspace keys go through `workspace_key`, an enum, like editor keys.

## Rejected alternatives

- **B:** two dispatch paths. **C:** still drifts.
