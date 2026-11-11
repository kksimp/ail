# 1017. `std.app` reports the display's size and scale

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.31` ([std-app.md](../spec/std-app.md))
- **Amends:** [ADR 0838](0838-std-app-native-windows.md)
- **Requested by:** Garlic (`kksimp/garlic`): `screen.width`, `screen.height` and `devicePixelRatio` must match Chrome's, and a program cannot see the display.

## Context

An app learns only its canvas size. Garlic reported the window area for `screen.*` and `1` for `devicePixelRatio`; Chrome reports the display and its scale (2 on a Retina Mac), and sites such as browserleaks read them.

## Decision

`Event` gets `screen_w`, `screen_h` (points) and `scale` (device pixels per point), filled on `start` and every `resize` and `0` elsewhere. Hosts report the main display: macOS `NSScreen.main.frame` and `backingScaleFactor`, Linux the primary GDK monitor's geometry and scale factor, Windows `Screen.PrimaryScreen.Bounds` and the DPI over 96. `0` means unknown; programs treat a `scale` of `0` as `1`. Headless events carry `0`, so tests need no fixed display. Garlic's `Gui.update` passes them to `Vm.viewport`.

## Rejected alternatives

- **A new `screen` event.** Apps already handle `start` and `resize`, and a display change coincides with a resize; a third event adds a case for nothing.
- **The window's current monitor.** More precise on multi-display setups, but each host needs window-to-monitor lookup; the main display is what Chrome's default reports first. Revisit if a user needs it.
- **Fixed 1.0 scale in headless runs.** Zero is the existing convention for absent facts (insets, `clicks`), and a fixed value would hide the case where the host cannot say.

## Consequences

Three more `Event` fields, zero for every other kind. The Mac host's `Edge.insets` now also carries the screen members, so `start` and `resize` have them without a second code path.
