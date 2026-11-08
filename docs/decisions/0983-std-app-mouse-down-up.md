# 0983. `std.app` `down` and `up` events with pointer capture

- **Status:** Accepted
- **Date:** 2026-11-08
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.27` ([std-app.md](../spec/std-app.md))
- **Amends:** [ADR 0838](0838-std-app-native-windows.md), [ADR 0931](0931-std-app-canvas-hover.md)
- **Requested by:** Garlic (`kksimp/garlic`): selecting text on a page by dragging, as every browser does.

## Context

A canvas gets `click` on a press and, with hover on, `move` with the pointer's position. Nothing says when a button is released, so a program cannot tell a drag from a hover, and a drag that leaves the canvas never ends.

## Decision

A host sends `down` on a press and `up` on the release, with the same fields as `click`; `click` stays as it is, sent after `down`. A press captures the pointer: until the release the same canvas gets `move` (whether or not hover is on) and the matching `up`, even when the pointer leaves the canvas or the window, with coordinates relative to that canvas. Every `down` is followed by exactly one `up`. The hosts use what the toolkit gives: AppKit delivers `mouseDragged` and `mouseUp` to the view that got `mouseDown`; GTK holds an implicit grab for the button; WinForms sets `Capture`. A host may omit both events, so a program must not depend on them.

## Rejected alternatives

- **A `drag` event with `dx`, `dy`.** The program needs absolute positions over its own layout, not deltas.
- **A `buttons` field on `move`.** `down` and `up` serve selection, drag and drop and scrollbars alike with one mechanism, and the release matters on its own.
- **`click` on the release.** It would change what every program sees today; `click` stays on the press.

## Consequences

Two more event kinds and no new `Event` field. Headless `script` feeds `down`, `move` and `up` like any other event.
