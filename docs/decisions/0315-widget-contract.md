# 0315. Widgets render pure runs and turn events into an `Action`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture; tooling
- **Spec rules:** [std.ui.20–24](../spec/17-std.md)
- **Extends:** [ADR 0314](0314-standard-modules-in-ail.md), [ADR 0181](0181-editor-screen-as-text-runs.md), [ADR 0295](0295-editor-themes.md)

## Context

The editor renders a headless `Screen` of runs that its tests inspect; a
widget that could only draw on a `Window` couldn't serve it, and couldn't
be tested without a display. Deciding criteria: **AI understanding**
(state in plain fields, behaviour testable), then tokens.

## Options considered

### Option A — `render` to runs, `draw` puts them, `on_event` returns an `Action` (chosen)
Widgets are plain structs with `pub` fields. `render` is pure; the editor
turns runs into its spans, a window program calls `draw`, and a test calls
`text_rows`. `on_event` returns `Ignored`, `Changed`, `Submit` or `Cancel`,
so the caller decides what a submit means. Colors come from the theme maps
the editor already uses.
- **Pros:** one code path for window, editor and tests; no callbacks.
- **Cons:** a caller passes the `Rect` to both `render` and mouse-aware
  `on_event`.

### Option B — widgets draw only on a `Window`
- **Cons:** untestable headless; unusable by the editor's core.

### Option C — callbacks per action
- **Cons:** closures stored in structs; harder to follow than a returned value.

## Decision

Option A, with `TextField`, `ListView` (fuzzy filter), `Button`, `Dialog`,
`Pane` and `Tree`. The editor's file picker and command palette (a
`TextField` over a `ListView`) and its text pane (a `Pane`) now render
through them, character for character as before; the selection in a list
longer than its 12 rows now scrolls into view. The rename prompt stays on
the editor's status line.

## Rejected alternatives

- **Window-only widgets:** no headless tests, no editor.
- **Callbacks:** indirect.
