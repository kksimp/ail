# 1032. `std.app` vector ops: `Round`, `Poly`, `Line`

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.32` ([std-app.md](../spec/std-app.md))
- **Amends:** [ADR 0838](0838-std-app-native-windows.md), [ADR 0933](0933-std-app-alpha-fill.md)

## Context

A canvas can draw rectangles (`Fill`, `Shade`), single line segments (`Stroke`) and images. A disc, a rounded tab or an icon is built from 20 to 60 one-pixel bands or short strokes. No host can anti-aliase those at 1x, and the display list (and the JSON per frame) grows with every shape. Garlic's chrome looks different on each platform for this reason.

## Decision

Three ops in `enum Op`, all anti-aliased, all in points:

- `Round(x, y, w, h, r, color, alpha, stroke float)`: a rounded rectangle; `r` is clamped to half the shorter side (a circle is `r = w / 2`); `stroke` 0 fills, above 0 outlines with that width centred on the edge; `alpha` 0 to 1.
- `Poly(points, color, alpha)`: a filled polygon, `points` is `x0, y0, x1, y1, ...`, non-zero winding; fewer than 3 points draw nothing.
- `Line(points, color, width)`: an open polyline with round caps and joins; fewer than 2 points draw nothing.

JSON is `{"kind":"Round",...}`, `{"kind":"Poly","points":[...],...}` and `{"kind":"Line",...}`, written by hand with the other ops (std.app.7). The three hosts draw them at device resolution: AppKit `NSBezierPath`, GDI+ `GraphicsPath` with `SmoothingMode.AntiAlias`, cairo arcs with round caps. The display list is produced by the program, so it is the same bytes on every host; a host that does not know an op skips it (std.app.19). `app.outline` and `std.screen` show `Round` and `Poly` as a `box` and `Line` as a `line` over the bounding box.

## Rejected alternatives

- **A general path op with curves.** More code in three hosts, and nothing in Garlic's chrome needs curves once arcs are polylines with round joins.
- **Host-side supersampling of the old ops.** Keeps the op count and cannot fix band steps.
- **A `Circle` op.** It is `Round` with `r = w / 2`.

## Consequences

An old host skips the new ops, so shapes are missing there. Garlic's move to the ops (G1) waits for a pinned ail with the hosts updated.
