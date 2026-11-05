# 0931. `std.app` canvas hover events

- **Status:** Accepted
- **Date:** 2026-11-04
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.16` ([std-app.md](../spec/std-app.md))
- **Amends:** [ADR 0838](0838-std-app-native-windows.md)
- **Requested by:** Husk (the Mac disk-space app, `kksimp/husk`), whose GUI needs it. Not implemented yet.

## Context

A treemap wants a tooltip and a hover highlight. A canvas only reports `click`, `scroll` and `key`, so the app cannot know where the pointer is.

## Decision

Two new event kinds, for a canvas: `move` (`id`, `x`, `y`, in the canvas's points) and `leave` (`id`). Hosts send them only when the app asks: `trait App` gets `hover(self) bool` (default `false`), and the `view` line carries `"hover":true` when it is. Hosts send at most one `move` per 16 ms, the last position. The macOS SwiftUI host (ADR 0838) uses a tracking area on the canvas view. Hosts that cannot track the pointer never send either kind, so an app must work without them (hover only adds polish, never the only way to learn something).

## Rejected alternatives

- **Always send `move`.** Floods the pipe and wakes the app on every pointer twitch for apps that do not care.
- **A tooltip node drawn by the host.** The host would need app data (what to say per tile); the app can draw its own tooltip from `move`.
- **A per-canvas flag in `Node.Canvas`.** Changes a published variant in every host and every app.

## Consequences

Costs one `bool` per frame line. Headless `script` can pass `move` events like any other.
