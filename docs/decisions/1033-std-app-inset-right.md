# 1033. `std.app` edge mode: `inset_right` for window controls on the right

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.33` ([std-app.md](../spec/std-app.md))
- **Amends:** [ADR 0981](0981-std-app-edge-mode.md)
- **Requested by:** the designer (2026-11-09): Garlic on Windows and Linux looks as good as on macOS, with the tab strip in the title bar.

## Context

ADR 0981 reports the window controls' room only at the top-left (`inset_left`, `inset_top`), which is where macOS puts them. On Windows and Linux the controls sit at the top-right, so an app that fills the strip would draw tabs under them.

## Decision

`start` and `resize` also carry `inset_right` (points), `0` when the host draws no controls at the top-right. `Event` gains `inset_right` (`0.0` by default, read by `event_from_json`). The macOS host sends none, which reads as `0`. The Windows host, in edge mode, hides the caption, keeps the frame (shadow, snap, resize edges, Windows 11 rounded corners) and draws minimise, maximise or restore, and close buttons of 46 by 32 points, so it reports `inset_right` 138, `inset_left` 0 and `inset_top` 32. `drag` and `zoom` (std.app.25) work as on macOS. The Linux host, in edge mode, drops the GTK title bar, overlays the same three buttons (46 by 32 points, themed icons) and 5-point resize edges on the left, right and bottom, and reports the same insets; `drag` begins a GTK move from the last press and `zoom` toggles maximise.

## Rejected alternatives

- **Mirroring the controls to the left on Windows and Linux.** Against each platform's convention.
- **The app drawing the controls itself.** Three look-alikes to maintain in every app, and none matches the system theme.
- **One signed inset.** A window may one day have controls on both sides (Linux `gtk-decoration-layout`); two fields say so plainly.

## Consequences

One more `Event` field, zero for every other kind. Apps that ignore it are unchanged on macOS and keep working elsewhere. Garlic's tab strip uses it in the Garlic G2 slice.
