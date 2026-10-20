# 0241. The editor gets tabs, a file picker, mouse input and F5/F6 runs in its window front end

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API (tools); 28 — Testing strategy (tool)
- **Spec rules:** none (a tool); uses [std.ui.10–13](../spec/17-std.md)
- **Extends:** [ADR 0180](0180-editor-written-in-ail.md)

## Context

`tools/edit/edit.ail` had one file per window and keys only. The designer
asked for the nice-to-haves. Lane 12 owns the editor core (autocomplete,
formatting, navigation); this ADR covers the window front end. Criterion:
**AI understanding** of the tool's code: the `Editor` value stays the
unit of editing, and new behaviour sits in the front-end layer.

## Options considered

### Option A — a `Workspace` of `Editor`s in the front end (chosen)
- **Pros:** `Editor` and its tests don't change; tabs, the picker and
  Ctrl-Q over several files live in one struct with its own headless tests.
- **Cons:** each tab keeps its own undo and editor-local clipboard (the
  system clipboard is shared).

### Option B — several buffers inside `Editor`
- **Pros:** one struct.
- **Cons:** every core function would grow a buffer index; conflicts with
  lane 12's work in the core.

## Decision

The window front end runs a `Workspace`: a tab bar on row 0, Ctrl-Tab and
Ctrl-Shift-Tab switch, a click on a tab selects it, Ctrl-W closes a tab
(twice when unsaved), Ctrl-Q quits (twice when any tab is unsaved), and
Ctrl-P opens a fuzzy file picker over the files under the working
directory (four levels, hidden and `target` directories skipped). A click
places the caret, a drag selects, the wheel scrolls the view without moving
the caret, a click on a diagnostic jumps to it. Copy, cut and paste also use
the system clipboard. F5 runs the buffer with `ail run` and F6 with
`ail test`; the output fills the pane. The window title shows the file and
an unsaved mark.

## Rationale

It keeps the core untouched for the parallel lane and is testable headless
by feeding `ui.Event` values.

## Consequences

- F5 and F6 block the editor until the program ends; a program that waits
  for input gets none (standard input is empty).
- Shift-click selection waits for modifier state on mouse events (ADR 0240).

## Rejected alternatives

- **Buffers inside `Editor`:** touches every core function and the other
  lane's code.
