# 0978. `std.app` canvas paste event

- **Status:** Accepted
- **Date:** 2026-11-07
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.23` ([std-app.md](../spec/std-app.md))
- **Amends:** [ADR 0838](0838-std-app-native-windows.md)
- **Requested by:** Garlic (`kksimp/garlic`), whose Chromium-style omnibox is drawn on a canvas and edited in AIL (the designer asked for a Chromium look, 2026-11-07).

## Context

A canvas reports keys, but `std.ui.clipboard_get` (std.ui.12) is the program's own clipboard, not the system's, so a drawn text field cannot take a paste. Command-V in a canvas arrives as the key `v` with `meta`, which tells the program nothing about what to insert.

## Decision

A new event kind for a canvas: `paste` (`id`, `text` is the system clipboard's text). The host sends it instead of the `key` event when the paste key is pressed in a canvas: Command-V on macOS, Control-V on Linux and Windows. All three hosts read their clipboard (`NSPasteboard`, `GtkClipboard`, `Clipboard.GetText`). A host that cannot read it keeps sending the `key` event, so a program treats `paste` as a convenience, never the only way in.

## Rejected alternatives

- **Route `std.ui.clipboard_get` through the host.** A request and reply over the pipe inside a synchronous call blocks the app's event loop, and the runtime would need a channel to the host it does not have.
- **Send the clipboard with every `key` event.** Leaks the clipboard to any program on every keystroke.
- **A native `Field` node styled by the app.** Three hosts, three looks, and no way to draw a dimmed path or a suggestion dropdown (Garlic's need).

## Consequences

One more event kind for hosts; headless `script` passes `paste` like any other. Copy (Command-C) from a canvas is not covered; a `clipboard` protocol line from the app can follow when a program needs it.
