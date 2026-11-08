# 0981. `std.app` edge mode: the app draws into the title bar

- **Status:** Accepted
- **Date:** 2026-11-07
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.25` ([std-app.md](../spec/std-app.md))
- **Amends:** [ADR 0838](0838-std-app-native-windows.md)
- **Requested by:** Garlic (`kksimp/garlic`): the designer wants it to look like Chromium (2026-11-07), whose tab strip sits in the title bar on macOS.

## Context

A `std.app` window has the host's title bar above the app's tree. A browser with a drawn tab strip then shows two bars: the title and the tabs. Chromium, Safari and Firefox put the tabs in the title bar, beside the window controls.

## Decision

`trait App` gets `edge(self) bool` (default `false`); the `view` line carries `"edge":true` when it is. The host then hides its title bar and lets the tree reach the window's top edge, keeps its own window controls where they are, and tells the app the room they take: `inset_left` and `inset_top` (points) on `start` and on every `resize`, `0` when the host draws no controls there. With no title bar the user has nothing to drag, so the app sends `{"v":1,"t":"drag"}` through `lines()` (std.app.21) on a mouse-down in its empty strip and `{"v":1,"t":"zoom"}` on a double-click; the macOS host runs `performDrag` with that mouse-down and `zoom`. Hosts that cannot hide their title bar (the Windows and Linux hosts today) keep their decorations and report both insets as `0`, so a program never depends on edge mode.

## Rejected alternatives

- **`.hiddenTitleBar` at window creation for every app.** Every app would lose its title and have to draw its own strip.
- **One `inset` value.** A strip needs the controls' width to start right of them and the control band's height to centre on it.
- **The host drags on any empty background.** The host cannot tell the app's strip from a page; the app knows.

## Consequences

Two more `Event` fields, zero for every other kind. Headless `script` passes `start` and `resize` with insets like any other event.
