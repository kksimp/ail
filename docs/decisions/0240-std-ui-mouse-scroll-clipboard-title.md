# 0240. `std.ui` adds mouse, scroll, clipboard and title

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture
- **Spec rules:** [std.ui.2](../spec/17-std.md), [std.ui.9–13](../spec/17-std.md)
- **Extends:** [ADR 0177](0177-std-ui-text-grid-window.md)

## Context

The designer wants the AIL editor to have the usual comforts by morning:
click to place the caret, drag to select, wheel scrolling, and copy and
paste with other programs. `std.ui` (ADR 0177) had keys only. The deciding
criterion is **AI understanding**: events in cells, not pixels, so a program
maps them to text without knowing the font.

## Options considered

### Option A — `Mouse(col, row, button, kind)` and `Scroll(lines)` variants; clipboard through platform tools (chosen)
- **Pros:** cell coordinates match `put`; one variant per kind of input;
  the clipboard needs no new crate (`pbcopy`, `wl-copy`, `xclip`, `xsel`,
  PowerShell), and falls back to the program's own.
- **Cons:** two new variants break `match`es that listed every variant
  (the demo, a test and the spec example were updated); a Linux desktop
  without one of the tools keeps the clipboard inside the program.

### Option B — a clipboard crate (`arboard`)
- **Pros:** direct API.
- **Cons:** pulls in a dozen crates (X11 protocol, image handling) for two
  functions; ADR 0023 asks for justification of each crate.

### Option C — mouse movement events for every pixel
- **Pros:** hover effects.
- **Cons:** floods the event queue; an editor needs presses and drags only.

## Decision

`Event` gains `Mouse(col int, row int, button str, kind str)` (`left`,
`right`, `middle`; `down`, `up`, `drag`) and `Scroll(lines int)`
(positive towards the end, about 3 per notch). `ui.clipboard_set(text)` and
`ui.clipboard_get() str?` use the system clipboard under `ail run` and a
program-local one otherwise, and always under `ail test`, so tests never
touch the user's clipboard. `Window.set_title(text)` sets the title. All
give the capability `ui`.

## Rationale

The editor needed exactly these, and each is the smallest form that serves
it. Keeping tests away from the real clipboard keeps `ail test` hermetic.

## Consequences

- Programs that `match` every `Event` variant must list the two new ones;
  [gui-path.md](../design/gui-path.md) asks future variants to be batched.
- Shift-click selection needs modifier state on mouse events; a later
  additive change can add a method `w.modifiers()` instead of a field.

## Rejected alternatives

- **Clipboard crate:** too many transitive crates for two functions.
- **Pixel-level mouse motion:** floods events; not needed by a text UI.
- **Fields for modifiers on `Mouse`:** lane 12 had already coded against the
  four-field form; adding fields later breaks patterns, so it waits for a method.
