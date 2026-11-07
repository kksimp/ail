# 0973. `std.ui.widgets`: layout on `Rect`, Label, Checkbox, RadioGroup, ProgressBar, Tabs *(delegated)*

- **Status:** Accepted
- **Date:** 2026-11-07
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.ui.25`–`28` ([17-std.md](../spec/17-std.md))
- **Builds:** [ADR 0314](0314-standard-modules-in-ail.md)–[0315](0315-widget-contract.md), [ADR 0863](0863-native-std-ui-through-the-app-host.md)

## Context

`std.ui.widgets` had TextField, ListView, Button, Dialog, Pane and Tree. Programs
laid screens out by hand arithmetic and had no label, checkbox, radio, progress or
tab widget.

## Decision

Same contract as `std.ui.21` (pure `render`, `draw`, `on_event`), no new
capability, no runtime slots. Layout is plain `Rect` methods: `left`, `top`,
`inset`, `has`, and `cols_of`/`rows_of` taking sizes where -1 shares the rest.
New widgets: `Label`, `Checkbox`, `RadioGroup`, `ProgressBar` (integer percent),
`Tabs`.

## Rationale

Measured criteria: one list of sizes replaces several lines of subtraction per
split (tokens), and pure rectangles keep each widget locally checkable (AI
understanding). Integer-only state keeps screens testable without a window.

## Rejected alternatives

- A layout tree object (`VBox`/`HBox` widgets): more types and an event routing
  layer for no token gain over rectangle lists.
- Float progress values: needs rounding rules; integer `done`/`total` suffices.
- Focus management inside the module: left to the program, as for Button.
