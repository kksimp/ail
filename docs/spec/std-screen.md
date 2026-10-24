# Standard library: `std.screen`

Chapter key: `std`

> **Status:** Drafted, slice 1 (ADR 0878). Items and queries are built; the
> pixel classifier (`std.screen.3`) and the `std.app` hook come in later slices.

What is on a screen, as text an agent can read without a vision model.

**std.screen.1** — `use std.screen` provides `Item`, `Screen`, `item`,
`color_name` and `lines`. A `Screen` is a size and a list of `Item`s in paint
order (later items are on top). Every function is pure and needs no
capability. *(ADR 0878)*

**std.screen.2** — The queries are `what_is_at(x, y)`, `find(role, color,
text, budget)` and `describe(x, y, w, h, budget)`. Each answers in text, one
line per item: `#7 button 'Submit', red, at 412,300 100x32, enabled, links to
/pay`. `find` ANDs its non-empty criteria; `text` matches as a
case-insensitive substring. `what_is_at` answers with the topmost item.

**std.screen.3** — An item read from the program's own tree has `conf` 1.0.
An item guessed from pixels has `conf` below 1.0 and a `why` naming the
evidence, and its line says `probably`.

**std.screen.4** — A `budget` of N tokens (about four characters each; 0 for
none) limits the answer. Items are listed focused first, then interactive,
then by area. The answer ends in `+N more` for the items left out; nothing is
cut silently.

**std.screen.5** — `Item.id` is stable across frames: the same thing keeps
its id.

**std.screen.7** — Colours are named from a fixed palette (`black`, `dark
gray`, `gray`, `light gray`, `white`, and `red`, `orange`, `yellow`, `green`,
`cyan`, `blue`, `purple`, `pink`, each also `dark X`; `brown`).

**std.screen.8** — `changes_since(before, budget)` lists items by id that
appeared (`+`), went (`-`) or changed (`~`), or says `unchanged`.

**std.screen.9** — Nothing is computed until a query is called: no
background work, no streaming.

Rejected: a vision model over screenshots (slow, token-heavy, guesses what the
program knows); streaming every frame (costs tokens whether or not asked).
