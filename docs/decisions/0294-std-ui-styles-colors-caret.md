# 0294. `std.ui` text styles, named colors and caret shapes

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture
- **Spec rules:** [std.ui.16–18](../spec/17-std.md)
- **Extends:** [ADR 0177](0177-std-ui-text-grid-window.md), [ADR 0243](0243-std-ui-font-size.md)

## Context

Editor themes need bold or italic tokens, a palette programs can name, and
a caret the window draws in more than one shape. The deciding criteria are
**tokens** (calls are frequent in UI code) and **AI understanding** (every
argument checked by type, nothing stringly typed).

## Options considered

### Styles
- **A — `put_styled(..., ui.Style(bold, italic, underline))` beside `put` (chosen).**
  Named bools read unambiguously; `put` stays as short as before.
- **B — int flags (`ui.BOLD + ui.ITALIC`).** Fewer tokens per call, but
  adding a flag twice silently gives another style.
- **C — a style argument on `put`.** Breaks every existing call.

### Colors
- **A — module constants `ui.RED` (chosen).** Two tokens, typed `int`,
  usable wherever a color is; a misspelling is a compile error listing the
  colors (std.ui.17).
- **B — an enum `ui.Color.Red` with a conversion.** More tokens and a call
  per use.
- **C — `ui.color("red")`.** Errors only at run time.

### Caret
- **A — `set_cursor(col, row, shape)` with an enum `ui.Cursor` (chosen).**
  A bare variant (`Block`) is typed by the parameter; `cursor` stays as
  the `Bar` shorthand.
- **B — a string shape.** Unchecked.

## Decision

`put_styled` takes a `ui.Style`; ten named colors are `int` constants of
`std.ui`; `set_cursor` takes `ui.Cursor` (`Bar`, `Block`, `Underline`).
The bootstrap draws the styles from the one bitmap font (bold: strokes one
pixel wider; italic: upper rows shifted right; underline: the row below
the baseline) and caches each styled glyph.

## Consequences

- Programs that use only `put` and `cursor` are unchanged.
- A native layer draws real bold and italic faces; the API stays.

## Rejected alternatives

- **Int flags:** ambiguous sums.
- **Enum colors or string lookup:** more tokens, or errors at run time.
- **String caret shapes:** unchecked.
