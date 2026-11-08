# 0932. `std.app` click count and context menus

- **Status:** Accepted
- **Date:** 2026-11-04
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.17`, `std.app.18` ([std-app.md](../spec/std-app.md))
- **Amends:** [ADR 0838](0838-std-app-native-windows.md)
- **Requested by:** Husk (the Mac disk-space app, `kksimp/husk`), whose GUI needs it. Not implemented yet.

## Context

Husk wants double-click to open a folder and a right-click menu (Reveal in Finder, Move to Trash). `click` already says `left`, `right` or `other` in `text` (`std.app.5`), but not how many clicks, and an app cannot ask for a native menu.

## Decision

1. `struct Event` gets `clicks int`: 1, 2 or 3 for a `click`, `0` for other kinds. The host fills it from the platform's own multi-click timing.
2. `trait App` gets `popup(self) Popup?` (default `none`); `struct Popup` is `id str`, `x float`, `y float` and `items List[str]`. Like `measures`, `run` checks it after each `update`; when it differs from the last one sent it sends `{"v":1,"t":"popup","id":I,"x":X,"y":Y,"items":[...]}`. The host shows a native menu there and answers `{"kind":"popup","id":I,"text":LABEL}` (`text` empty if dismissed). The app clears its `Popup` when it sees that event.
3. SwiftUI host: `NSMenu.popUp`. Other hosts show their toolkit's menu; a host with none answers dismissed at once, so apps keep a keyboard or button path for each menu item.

## Rejected alternatives

- **The app draws its own menu on the canvas.** Looks wrong, ignores accessibility and the system's keyboard handling.
- **A `Node.Menu` in the view tree.** A menu belongs to a moment (a right-click), not a screen; the view would carry state the user never sees.
- **A `double` click kind.** Triple-click and later gestures would need more kinds; a count covers them.

## Consequences

One field and one pair of messages. Menu labels are matched by text, so apps should not show two equal labels.
