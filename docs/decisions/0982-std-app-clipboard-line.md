# 0982. `std.app` clipboard line: an app copies text to the system clipboard

- **Status:** Accepted
- **Date:** 2026-11-08
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.26` ([std-app.md](../spec/std-app.md))
- **Amends:** [ADR 0838](0838-std-app-native-windows.md); the inverse of [ADR 0978](0978-std-app-paste-event.md)
- **Requested by:** Garlic (`kksimp/garlic`): Command-C in the omnibox and the find bar, and a page menu's "Copy link address" and "Copy text".

## Context

A canvas gets the clipboard's text as `paste` (std.app.23), but nothing an app draws can be copied out: the host sees only drawing operations, and the app has no way to reach the clipboard.

## Decision

An app sends `{"v":1,"t":"clipboard","text":T}` through `lines()` (std.app.21); `app.clipboard(text)` builds the line. The host puts `T` on the system clipboard as given (UTF-8, no newline conversion): `NSPasteboard.general` on macOS, the `CLIPBOARD` selection through `GtkClipboard` on Linux, `Clipboard.SetText` on Windows. A host that cannot write the clipboard ignores the line, as it ignores any unknown line, so a program must not depend on it.

## Rejected alternatives

- **The host copies on Command-C itself.** It has no selection: the app knows what is selected in a drawn field, and whether the key means copy at all.
- **A `copy` event the app answers.** A round trip for nothing; the app already knows the text when the key arrives, and `lines()` exists.
- **A `trait App` method `clipboard(self) str?`** like `popup` or `pick`.** Copying is an action at an instant, not state the view carries; a line fits, as `drag` and `zoom` (std.app.25) do.

## Consequences

One line kind and one std function. Headless tests read the line from `lines()`, as the spec example does.
