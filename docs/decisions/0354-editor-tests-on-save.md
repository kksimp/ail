# 0354. Editor tests on save (Ctrl-Shift-U)

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling — the editor ([ADR 0180](0180-editor-written-in-ail.md))
- **Spec rules:** none (tool behaviour); reads `ail test --json`

## Context

F6 runs a tab's tests into the pane, which replaces whatever the pane
showed. A quick loop of edit, save and see whether tests pass wants the
result without losing the pane.

## Decision

Ctrl-Shift-U toggles tests on save for the workspace. Turning it on runs
the active tab's tests at once; after that each save runs them. The tab's
status line adds `tests P/P pass`, `tests F FAILED of N`, or
`tests: didn't run` when `ail test --json` gives no summary (the program
doesn't check). The pane is left alone; F6 still shows the full output.
Turning it off clears the counts.

The brief suggested Ctrl-Shift-T, which already moves to the next theme
(ADR 0295), so the toggle is Ctrl-Shift-U ("unit tests").

## Rejected alternatives

- **Ctrl-Shift-T, moving the theme:** breaks a remembered key for no gain.
- **Show the results in the pane:** hides the problems or search list the
  person is working from; F6 already does that on demand.
- **Run the tests of every open tab:** slower per save and the status line
  is per tab; the saved tab is the one that changed.
