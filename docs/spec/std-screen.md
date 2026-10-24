# Standard library: `std.screen`

Chapter key: `std`

> **Status:** Drafted, slices 1-2 (ADR 0878). Items, queries and the pixel
> segmenter are built; wiring it to the editor and PDF viewer and the
> `std.app` hook come in later slices.

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

```ail rules=std.screen.1,std.screen.2,std.screen.3,std.screen.4,std.screen.5,std.screen.7,std.screen.8,std.screen.9
use std.screen

fn main()
    mut a := screen.item(1, "button", "Pay", 0xcc2222, 10.0, 10.0, 80.0, 30.0)
    a.href = "/pay"
    mut guess := screen.item(2, "button", "", 0x2255cc, 10.0, 60.0, 80.0, 30.0)
    guess.conf = 0.8
    guess.why = "rounded blue rectangle"
    s := screen.Screen(w: 200.0, h: 200.0, items: [a, guess])
    print(s.what_is_at(20.0, 20.0))
    print(s.find("button", "blue", "", 0))
    print(s.describe(0.0, 0.0, 200.0, 50.0, 0))
    print(s.find("", "", "", 1))
    print(s.changes_since(s, 0))
```

## The pixel classifier

**std.screen.10** — `use std.screen.pixels` provides `segment(w, h, px,
texts) List[Item]`. `px` is `w * h` packed `0xRRGGBB` values, row by row (any
other length gives no items). It returns one item per flat-colour region that is
at least 6 by 6, at least 80% of its bounding box, and under 60% of the raster
(the background). Ids are `5000000 + y * w + x` of the region's top left, so the
same box keeps its id between frames (`std.screen.5`). Every item has `conf`
below 1.0 and a `why` (`std.screen.3`).

**std.screen.11** — A region is a `panel` (`conf` 0.5) unless a text item in
`texts` has its centre inside it, its height is 16 to 60, its width is 1.5 to 12
times its height and its area is under 5% of the raster; then it is a `button`
whose text is the label, `conf` 0.9 with a cut corner (rounded) and 0.7 without.

```ail rules=std.screen.10,std.screen.11
use std.screen
use std.screen.pixels

fn main()
    w := 400
    h := 200
    mut px := List[int]()
    for _ in 0..w * h
        px.push(0xffffff)
    for y in 10..34
        for x in 10..60
            px[y * w + x] = 0xcc2222
    texts := [screen.item(1, "text", "Pay", 0xffffff, 25.0, 16.0, 20.0, 12.0)]
    print(pixels.segment(w, h, px, texts)[0].line())
```
