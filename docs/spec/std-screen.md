# Standard library: `std.screen`

Chapter key: `std`

> **Status:** Drafted, slices 1-3 (ADR 0878). Items, queries, the pixel
> segmenter and the `std.app` view are built; reading the editor's and PDF
> viewer's rasters and live attach come in later slices.

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

## Any `std.app` program

**std.screen.12** — `use std.screen.view` provides `of_ops(ops, ox, oy)`,
which turns canvas ops into items moved by (`ox`, `oy`): `Fill`, `Shade`, `Round` and `Poly` are a `box`, `Str`
a `text`, `Pic` and `Bitmap` an `image`, `Stroke` and `Line` a `line`. These boxes are
exact (`conf` 1.0), except that a string's width is estimated.

**std.screen.13** — `of_node(node, w, h, focus)` makes the `Screen` of a view
tree: `Button` is a `button`, `Field` a `textbox`, `Toggle` a `checkbox`,
`Label` a `text` and `Canvas` a `canvas` followed by its ops. The host owns
the real layout, so controls are placed by nominal sizes and carry `conf` 0.9
with `why` "estimated layout"; `focus` (`App.focus`) marks the focused control.
For app `a`: `of_node(a.view(), w, h, a.focus())`. The model is built only when
this is called (`std.screen.9`).

```ail rules=std.screen.12,std.screen.13
use std.app
use std.screen
use std.screen.view

fn main()
    tree := app.column([app.label("Cart"), app.button("pay", "Pay")])
    s := view.of_node(tree, 200.0, 100.0, "pay")
    print(s.find("button", "", "", 0))
```

**std.screen.14** — `of_ops` also runs the pixel classifier (`std.screen.10`) over
each `Bitmap` that carries its pixels, and adds the regions it finds after the
`image` item, moved and scaled to the bitmap's rectangle (a PDF page, a
screenshot). A `Bitmap` with an empty `data` or a size that doesn't match
`pw`, `ph` gets the `image` item only. Nothing is computed until a query is
called (`std.screen.9`). *(ADR 0878)*

```ail rules=std.screen.14
use std.app
use std.screen.view

fn main()
    mut rgba := List[int]()
    for y in 0..20
        for x in 0..40
            c := if x >= 4 and x < 16 and y >= 4 and y < 12 then 0xcc else 0xff
            rgba.extend([c, c, c, 255])
    ops := [app.Op.Bitmap(x: 10.0, y: 10.0, w: 80.0, h: 40.0, id: "page", pw: 40, ph: 20, data: app.bitmap_data(rgba))]
    for it in view.of_ops(ops, 0.0, 0.0)
        print("{it.role} {it.x} {it.y}")
```

## Programs that draw a character grid

**std.screen.15** — `use std.screen.cells` provides `of_cells(cols, rows,
runs, cursor_col, cursor_row)`, the `Screen` of a grid of `std.ui.widgets`
runs, in cell units (a run at column 4 of row 2 is at x 4, y 2). Each run on a
background other than the page's (the background covering the most cells) is a
`box`; each run with visible text is a `text` with the run's colour; the caret is
a `caret` of one cell. All are exact (`conf` 1.0), so no pixel classifier is
needed. The editor (`--print --screen`) and the PDF viewer (`view --screen`)
print `describe` of it. *(ADR 0878)*

```ail rules=std.screen.15
use std.screen.cells
use std.ui.widgets

fn main()
    runs := [
        widgets.Run(col: 0, row: 0, text: "hello", fg: 0x202020, bg: 0xffffff, bold: false),
        widgets.Run(col: 0, row: 1, text: "status", fg: 0xffffff, bg: 0x007acc, bold: false),
    ]
    s := cells.of_cells(20, 2, runs, 5, 0)
    print(s.find("box", "", "", 0))
```
