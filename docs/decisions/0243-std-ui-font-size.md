# 0243. `Window.set_font_size` picks from fixed bitmap sizes and keeps the window's pixels

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture
- **Spec rules:** [std.ui.15](../spec/17-std.md), [std.ui.7](../spec/17-std.md)
- **Extends:** [ADR 0177](0177-std-ui-text-grid-window.md), [ADR 0240](0240-std-ui-mouse-scroll-clipboard-title.md)

## Context

The editor needs a font size option (`--font N`, Ctrl-+ and Ctrl--).
`std.ui` drew one 10x20 bitmap font. The bootstrap has bitmap fonts only
(`embedded-graphics`), and no outline renderer (ADR 0009 rules out adding
one just for this). Deciding criteria: **AI understanding** (a size means
one thing, and what happens to the grid is predictable), then **human
readability** (large sizes must stay crisp).

## Options considered

### Option A — a cell height in pixels, rounded to the nearest available size; the window keeps its pixels (chosen)
- **Pros:** one integer, the same as editors' "font size"; every size is a
  real bitmap font or an exact 2x of one, so glyphs stay sharp; zooming
  changes how many cells fit, which programs already handle as `Resize`.
- **Cons:** sizes are a fixed list (13 of them, 10 to 40), and widths vary
  with the font (6 to 20 pixels).

### Option B — keep the grid's cells and resize the window
- **Pros:** the grid never changes.
- **Cons:** a zoom in can push the window off the screen; it is not what
  editors do.

### Option C — a scale factor (1x, 2x, 3x) of the one font
- **Pros:** trivial.
- **Cons:** 20, 40 and 60 pixels only; no smaller sizes.

### Option D — a font size on `ui.open`
- **Pros:** no redraw.
- **Cons:** can't zoom while running; a method covers opening too.

## Decision

`Window.set_font_size(mut self, px int)` picks the available size nearest
`px` (the smaller on a tie) from 10, 12, 13, 14, 15, 18, 20, 24, 26, 28, 30,
36 and 40; `Window.font_size(self) int` returns it. A window opens at 20.
The window keeps its pixel size; when the number of cells changes, a
`Resize` event follows. Sizes up to 20 are Latin-1 bitmap fonts (6x10 to
10x20); larger ones draw a smaller font at twice the size.

## Rationale

Programs already handle `Resize`, so a font change needs no new event and
no new pattern. Rounding means any integer is valid, so `--font 16` works
without the program knowing the list.

## Consequences

- The editor takes `--font N`, and Ctrl-+ (or Ctrl-=), Ctrl-- and Ctrl-0
  step through the sizes and reset to 20. Numpad + and - work too.
- A native layer (see [gui-path.md](../design/gui-path.md)) replaces the
  list with any size from the platform's outline fonts; the rounding rule
  stays.

## Rejected alternatives

- **Resizing the window:** can overflow the screen; not editor behaviour.
- **Scale factor only:** no sizes below 20.
- **A parameter of `ui.open`:** can't zoom at run time.
