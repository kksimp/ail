# 0887. `std.app` host for Linux: a GTK 3 program, linked dynamically from the system

- **Status:** Accepted
- **Date:** 2026-10-24
- **Decided by:** AI (delegated, ADR 0076); integrator confirmed GTK 3 on condition 2 below (item 308)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.7`–`10` ([std-app.md](../spec/std-app.md))
- **Builds:** [ADR 0838](0838-std-app-native-windows.md), [ADR 0839](0839-std-app-text-measure.md), [ADR 0009](0009-custom-tooling-policy.md)

## Context

ADR 0838 deferred the Linux host. Apps are written once against the view tree, so a Linux host only has to draw the tree and speak the same newline-delimited JSON as `tools/app-host/macos`.

## Decision

1. `tools/app-host/linux/host.c` is one C file against GTK 3 (with Cairo and Pango, which GTK brings in). It draws `Label`, `Button`, `Field`, `Toggle`, `Column`, `Row`, `Spacer` as GTK widgets and `Canvas` as a `GtkDrawingArea` drawn with Cairo and Pango. It answers `measure` (ADR 0839) and sends every event of std.app.5. It has its own small JSON reader, so the only dependency is GTK 3.
2. GTK is LGPL: it is **linked dynamically from the system and never vendored**. `run.sh` builds `build/ail-app-host` with `gcc` and `pkg-config` (installing `libgtk-3-dev` with apt if missing). It is not built by the selftest or the Mac gate.
3. `app.host_path` (std.app.10) uses `AIL_APP_HOST`, else the macOS build, else `tools/app-host/linux/build/ail-app-host`.
4. A new view with the same shape as the last one updates the widgets in place (so typing, focus and scroll survive); a different shape rebuilds them and restores the focused control.

## Rejected alternatives

- **Raw X11 or Wayland plus our own widgets:** matches ADR 0009 most strictly but means writing text entry, IME, clipboard, accessibility and both display protocols; weeks of work for a worse result.
- **Qt:** a larger dependency, C++ only, and the LGPL/commercial licence mix is harder to follow than GTK's.
- **GTK 4:** not on older distributions; GTK 3 is on every current desktop and the API we use is the same.
- **A Python or other scripting host:** adds a runtime that users may not have.

## Consequences

Linux runs `todo` and Garlic with the same programs as the Mac. Wayland-only sessions use XWayland or GTK's own Wayland backend, whichever GTK picks. Windows has its own host (separate lane).
