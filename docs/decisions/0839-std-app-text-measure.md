# 0839. `std.app` text measure: the host reports advance widths for the app's text runs

- **Status:** Accepted
- **Date:** 2026-10-22
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.13` ([std-app.md](../spec/std-app.md))
- **Amends:** [ADR 0838](0838-std-app-native-windows.md)

## Context

Garlic's pixel layout estimates text widths from a Helvetica advance table with a fudge factor (`gfx.text_width`). The host draws in the system font, so wrapping, shrink-to-fit items and flex rows are laid out with the wrong widths: on Wikipedia the tab row overlaps.

## Decision

1. `trait App` gets `measures(self) List[Measure]` and `measured(mut self, runs, widths)`, both with defaults, so no existing app changes.
2. After each `update`, `app.run` asks the host for `measures()` (`{"t":"measure","runs":[...]}`), waits for one `measured` event whose `text` is the widths joined by commas, and calls `measured`. The loop repeats up to four rounds until `measures()` is empty. Events that arrive meanwhile are queued and handled after.
3. The host measures with the font a `Str` op of the same family, size and weight is drawn in, so measure and draw agree.
4. Garlic's `gfx` records each run it had to estimate (`Scene.probes`), `Gui` keeps the measured widths by run, and lays out again with them. A run not yet measured, and the headless mode, use the estimate.

## Rejected alternatives

- **A synchronous measure call from inside `view`.** `view` is pure and AIL has no module state to hold the host's pipe.
- **Measure the drawn strings only.** Layout needs word widths, and drawn strings join several words.
- **Raise the estimate's fudge factor.** Wrong per font, per size, and still overlaps.
- **A width table shipped with Garlic.** Fonts differ between hosts and users.

## Consequences

The first layout of a page is estimated and replaced a round later, so the first frame can reflow once. A page with thousands of distinct words sends them in chunks of 5000. The Swift side is checked on the Mac gate only.
