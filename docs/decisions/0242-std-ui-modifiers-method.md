# 0242. `Window.modifiers()` reports the modifiers of the last event

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture
- **Spec rules:** [std.ui.2](../spec/17-std.md), [std.ui.14](../spec/17-std.md)
- **Extends:** [ADR 0240](0240-std-ui-mouse-scroll-clipboard-title.md)

## Context

The editor needs Shift-click to extend a selection, but `Mouse(col, row,
button, kind)` carries no modifiers (ADR 0240), and adding a field would
break every pattern that matches it. The deciding criterion is **AI
understanding** with no breaking change: existing programs keep compiling.

## Options considered

### Option A — a method, `w.modifiers() ui.Modifiers`, for the last event returned (chosen)
- **Pros:** additive; exact, because the host records the modifiers when it
  queues each event, not when the program asks; works for every event kind.
- **Cons:** a second call per event when a program cares.

### Option B — a `shift` field on `Mouse`
- **Pros:** one call.
- **Cons:** breaks every `Mouse(c, r, b, k)` pattern in existing programs.

### Option C — the modifiers held now, when the program asks
- **Pros:** simplest.
- **Cons:** racy: the key may be released before the program reads the click.

## Decision

`Window.modifiers(self) ui.Modifiers` (`ctrl`, `alt`, `shift`) returns the
modifiers held when the last event returned by `next_event` or `poll_event`
happened; all false before any event and on a closed window.

## Rationale

It is the only option that is both additive and exact.

## Consequences

- The editor's Shift-click extends the selection (ADR 0241's front end).
- Future per-event data can follow the same pattern instead of new fields.

## Rejected alternatives

- **Field on `Mouse`:** breaks existing patterns.
- **Current state at the call:** racy.
