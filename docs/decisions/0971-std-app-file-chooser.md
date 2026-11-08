# 0971. `std.app` native file and folder chooser

- **Status:** Accepted
- **Date:** 2026-11-06
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.22` ([std-app.md](../spec/std-app.md))
- **Amends:** [ADR 0838](0838-std-app-native-windows.md)

## Context

Husk needs the user to pick a folder to scan. A text field works, but a Mac user expects the system dialog, with its sidebar, recents and permission prompts. `std.app` has no way to ask the host for one.

## Options considered

### Option A — a request like `popup` (chosen)
`App.pick() Picker?`; `run` sends a `pick` line when it changes, the host answers `picked`.
- **Pros:** same shape as `popup` (ADR 0932); the app stays a pure function of its state; headless `script` tests it by feeding `picked`.
- **Cons:** the app clears its request itself.

### Option B — a blocking call, `app.choose_folder(...)`
- **Pros:** reads naturally.
- **Cons:** `update` would block the loop and the inbox; no headless story.

## Decision

`trait App` gets `pick(self) Picker?` (default `none`). `Picker` is `id`, `prompt`, `folders`, `start`. The host shows its native open dialog and answers `picked` (`id`, `text` the path, empty on cancel). The macOS host uses `NSOpenPanel` (a sheet on the key window). Linux and Windows hosts have no dialog yet, so they answer `pick_error` with a clear message at once rather than ignore the request; apps keep a text field as the fallback. GTK `GtkFileChooserNative` and the Windows folder dialog follow in lockstep work (ADR 0008) once those hosts' dialogs are written.

## Rejected alternatives

- **Option B.** Blocks the event loop.
- **Silently ignoring the request on other hosts.** The app would wait forever.
- **A `Button` kind that opens the dialog.** Ties the dialog to one control; apps want to open it from keys or menus too.

## Consequences

One method and one message pair. The Linux and Windows dialogs remain to do; until then the clear error stands.
