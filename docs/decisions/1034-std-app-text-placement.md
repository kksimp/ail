# 1034. `std.app` text placement and `sans_line`

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076); number confirmed by the integrator
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.34` ([std-app.md](../spec/std-app.md))
- **Amends:** [ADR 0838](0838-std-app-native-windows.md)

## Context

A `Str` puts the top of its text at `y`, but each host's line box differs, so text centred by constants tuned on the Mac (Garlic's tabs, omnibox and menu rows) sits high or low on Windows and Linux. The Windows host also drew text with GDI+ `ClearTypeGridFit`, which spaces glyphs unevenly at small sizes.

## Decision

1. For a `Str` at `y`, a host places the ascent-to-descent box of the glyphs in the centre of the line box whose top is `y` (`line` is the font's natural line height at that size). On a font with no line gap this is today's placement.
2. `start` and `resize` carry `sans_line`, the natural line height of `sans` at 13 points, rounded up to whole points like `cell_h`. An app centres text in a row with `row_top + (row_h - sans_line * size / 13) / 2`. `0` means the host does not say.
3. The Windows host takes its families from a table: `sans` is Segoe UI Variable Text when it is installed and its bold and italic are drawn at the widths they are measured at (checked once, by drawing "Hamburgefonstiv" and comparing the advance-minus-ink gap of bold and italic with regular, within 1 px), else Segoe UI; `serif` is Georgia; `mono` is Cascadia Mono when installed, else Consolas.
4. While the renderer is chosen, the Windows host reads `AIL_HOST_TEXT` (a: GDI+ ClearTypeGridFit, the old behaviour and the default; b: AntiAliasGridFit; c: AntiAlias with `TextContrast` 0; d: GDI `TextRenderer`). The switch is removed once captures pick the variant.

## Rejected alternatives

- **A baseline-anchored `Str`.** It changes every existing app.
- **Per-run ascent and descent in `measured`.** It doubles the reply for a need one number covers.
- **Direct2D and DirectWrite now.** About 500 lines of COM interop; only worth it if the GDI variants are still clearly worse than the browser (a separate slice).

## Consequences

The Mac host sends `sans_line` too; Garlic's G3 slice replaces its constants. The Linux host (L3) sends it and applies the same placement.
