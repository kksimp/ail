# 0930. `std.app` progress events for long jobs

- **Status:** Accepted
- **Date:** 2026-11-04
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.15` ([std-app.md](../spec/std-app.md))
- **Amends:** [ADR 0838](0838-std-app-native-windows.md)
- **Requested by:** Husk (the Mac disk-space app, `kksimp/husk`), whose GUI needs it. Not implemented yet.

## Context

A scan or copy runs in a task and tells the window how far it has got. [ADR 0861](0861-std-app-inbox.md) lets a task post `external` events, but each app invents the text format (`pct|files|path` in Husk), so an agent or test cannot read progress the same way across apps.

## Decision

`struct Progress` has `done int`, `total int` (`0` when unknown) and `note str`. `app.progress(source, p Progress) Event` makes an `external` event whose `text` is `progress|done|total|note`; `app.progress_of(ev Event) Progress?` reads it back (`none` for any other event). Pure AIL: no host change, so every host works the same. `app.outline` is unchanged.

## Rejected alternatives

- **A new `progress` event kind from the host.** The host does not know about tasks; only the program does.
- **A `Node.Progress` bar.** Useful, but a canvas already draws one, and a node needs a change in every host. Reopen when a second app wants a native bar.
- **Float percent only.** Loses "N files" and a note, and cannot say "unknown total".

## Consequences

Husk's scan, Clove's archive progress and a download can share one reader. The format is part of the rule, so it never changes without a new ADR.
