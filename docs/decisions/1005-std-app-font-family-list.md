# 1005. `std.app` font-family lists: the host measures and draws in the first installed family

- **Status:** Accepted
- **Date:** 2026-11-08
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.28`, amends `std.app.13` ([std-app.md](../spec/std-app.md))
- **Amends:** [ADR 0839](0839-std-app-text-measure.md), [ADR 0984](0984-std-app-font-op.md)

## Context

`Measure.font` and `Str.font` name one of three faces (`mono`, `sans`, `serif`) or a web font (`web:` and an id, ADR 0984). A web page sets `font-family: Inter, "Helvetica Neue", Arial, sans-serif`: Garlic can only reduce that to a face, so text in an installed family (Georgia, Arial, Menlo) is measured and drawn in the wrong one, and font-fingerprinting tests (browserleaks `/fonts`) see every family measure the same. Resolving the list in Garlic would need the installed font list in the program; the host has it.

## Decision

1. `Measure.font` and `Str.font` may be a CSS `font-family` list. The host measures and draws in the first installed family (matched without regard to case); when none is installed, in the list's first generic (`serif`; `monospace` and `ui-monospace` are `mono`; `sans-serif`, `system-ui` and `ui-sans-serif` are `sans`); with no generic, `sans`. All three hosts (macOS, Linux, Windows) resolve the same way, so the spec can say what each returns.
2. The `measured` event may carry `fonts`: each run's chosen family in order, joined by commas. `Event` gets `fonts`, and `trait App` gets `fonts_used(mut self, runs, used)` (default nothing), called after `measured` when the host said; a host that omits `fonts` changes nothing for the app.
3. `std.app` has `family_list(font)` and `generic_of(font)`, the parsing and fallback the hosts use, so a program can predict the host and tests are deterministic without a host: a family certain to be missing forces the generic.
4. `Measure` itself is unchanged; the fake test host answers the generic face.

## Rejected alternatives

- **Send the installed font list to the program at start.** Hundreds of names in the pipe for every app, and the program would still need the host's matching rules.
- **A separate `fonts` request.** Another protocol round after each layout; the chosen family is already known when the width is measured.
- **Reuse `Event.id` for the families.** Fewer tokens by a hair (under 2%, a tie by ADR 0076), and ambiguous: `id` names a control everywhere else.
- **Resolve in the program with a bundled font list.** Wrong on every machine but the one it was made on.

## Consequences

A page's text measures in the right family when it is installed, and an app learns which. The hosts build an installed-family set once per process (`NSFontManager`, `pango_font_map_list_families`, `InstalledFontCollection`). The Swift, C and C# sides are checked on the gates of their own machines only.
