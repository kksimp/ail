# Standard library: `std.app` native windows

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in the bootstrap interpreter
> (ADR 0838). Native builds wait for native `proc.start` (std.app.10). The
> macOS host is `tools/app-host/macos`, the Linux host (GTK 3, ADR 0887) `tools/app-host/linux`,
> Windows `tools/app-host/windows`.

An app is a value whose type implements `app.App`. It says what the window
shows as a *view tree* (`view`) and changes its state when something happens
(`update`). A separate *host* program draws the tree with the platform's own
controls and reports what the user does. The two talk over the host's
standard input and output, so no AIL code runs on the host's threads and no
callback crosses a language boundary (`ffi.decl.3`).

**std.app.1** — `use std.app` imports a standard module written in AIL
(`std.ui.19`). Its items are the types `Node`, `Op`, `Event`, `AppError` and
the trait `App`, and the functions below. *(ADR 0838)*

**std.app.2** — `enum Node` is the view tree: `Label(text str)`,
`Button(id str, caption str)`, `Field(id str, value str, hint str)` (one line
of text), `Toggle(id str, caption str, on bool)`, `Column(kids List[Node])`
(top to bottom), `Row(kids List[Node])` (left to right), `Spacer` and
`Canvas(id str, w float, h float, ops List[Op])`. A canvas whose `w` or `h`
is `0.0` fills the space left in that direction. The builders
`label(s)`, `column(kids)`, `row(kids)`, `button(id, caption)`,
`field(id, value, hint)`, `toggle(id, caption, on)` and
`canvas(id, w, h, ops)` give the same values without naming fields
(`type.struct.7`). *(ADR 0838)*

**std.app.3** — `enum Op` is a canvas's display list, drawn in order, in
points from the canvas's top-left corner, colors `0xRRGGBB`:
`Str(x, y, text, font, size, fg, bold, italic, underline)` (`font` is
`mono`, `sans`, `serif` (ADR 0905) or `web:` and the `id` of a `Font`, ADR 0984; `(x, y)` is the text's top-left corner),
`Fill(x, y, w, h, color)`, `Stroke(x1, y1, x2, y2, color, width)`,
`Round`, `Poly` and `Line` (anti-aliased shapes, std.app.32),
`Pic(x, y, w, h, src)` (an image file path or URL),
`Bitmap(x, y, w, h, id, pw, ph, data)` (a `pw` by `ph` image scaled to the
rectangle; `data` is its `R, G, B, A` bytes in standard base64, which
`bitmap_data(rgba List[int])` makes; the host keeps the image under `id`, so an
empty `data` draws the one already sent, and a host that has none under that
`id` draws a placeholder), `Font(id, data)` (a web font: `data` is the bytes of a
TrueType or OpenType file in standard base64; the host keeps it under `id`, and
a `Str` whose `font` is `web:` and that `id` is drawn and measured in it; an
empty `data` keeps the font already sent; a host without the font, or that
cannot read it, draws in `sans`; the op draws nothing itself, ADR 0984), `Clip(x, y, w, h)`
(limits later steps) and `Unclip` (ends the latest `Clip`).
`draw_text(x, y, s, fg)` is a `Str` in `mono` at size 13, and
`draw_rect(x, y, w, h, color)` a `Fill`. *(ADR 0838)*

**std.app.4** — `trait App` has `update(mut self, ev Event)` and
`view(self) Node`, and two methods with defaults: `title(self) str`
(`"AIL"`), the window title, and `focus(self) str` (`""`), the `id` of the
control that should have the keyboard focus; the host moves the focus when
the value changes. *(ADR 0838)*

**std.app.5** — `struct Event` has the fields `kind`, `id`, `text` (all
`str`), `x`, `y`, `dx`, `dy`, `cell_w`, `cell_h` (all `float`) and `ctrl`,
`alt`, `shift`, `meta` (all `bool`). Fields that don't apply to a kind are
zero or empty. The kinds are: `start` (the host is ready); `press` (a
button, `id`); `edit` (a field changed: `id`, `text`); `submit` (Return in a
field, `id`); `toggle` (`id`, `text` is `on` or `off`); `click` (in a canvas:
`id`, `x`, `y`, `text` is `left`, `right` or `other`); `scroll` (`id`, `dx`,
`dy` in points, positive `dy` when the content should move up); `key` (in a
canvas: `id`, `text` is a name or character as in `std.ui.7`, with `meta`
for Command); `resize` (a canvas's size: `id`, `dx` wide, `dy` high, and
`cell_w`, `cell_h`, the size of one character of `mono` at 13 points); and
`close` (the window closed). `event(kind)` makes an event with every other
field empty; `press(id)`, `edit(id, value)` and `key(name)` fill in one.
*(ADR 0838)*

**std.app.6** — `outline(n Node) str` is the tree as indented text, one
node per line: `"text"` for a label, `[caption] #id` for a button,
`<value> #id (hint)` for a field, `[x] caption #id` or `[ ] caption #id` for
a toggle, `column`, `row`, `~` for a spacer, and `canvas #id WxH ops=N`
followed by one line per step. It is how a test or an agent sees a screen
without a window. *(ADR 0838)*

**std.app.7** — The host and the program exchange newline-delimited JSON,
one object per line. The program sends `{"v":1,"t":"view","title":T,
"focus":F,"tree":N}`, where `N` is the `ToJson` form of `Node`
(`json.derive.1`: `{"kind":"Column","kids":[...]}`); `frame(a)` is that
line for app `a`. Every line the program sends carries `"v":1`. *(ADR 0838)*

**std.app.8** — The host sends events as `{"kind":K, ...}` with the fields
of `Event` (std.app.5) that apply; missing members are zero or empty
(`event_from_json(j)`). Its first line also carries `"v":1`. A line that
isn't a JSON object, or has no `kind`, is skipped. *(ADR 0838)*

**std.app.9** — `run(mut a T) ! AppError` and
`run_host(mut a T, cmd str, args List[str]) ! AppError`, for any `T` that
implements `App`, start the
host, send the first view, and wait for the host's first line. Then each
event goes to `a.update`, and the view is sent again when its line differs
from the last one sent. A `close` event is passed to `update`, then the
program closes the host's input, copies the host's standard error to its
own, waits for the host, and returns. The program's own standard output is
free for `print`. `AppError` is `error struct AppError { kind str, message
str }` with the kinds `no_host` (no host program), `start` (it couldn't
start), `version` (its first line lacks `"v":1`) and `host_exited` (it ended
before a `close`). *(ADR 0838)*

**std.app.10** — `run` finds the host with `host_path() str ! AppError`:
the environment variable `AIL_APP_HOST`, else `AIL_ROOT/tools/app-host/
macos/build/AIL App Host.app/Contents/MacOS/AILAppHost`, `AIL_ROOT/tools/app-host/linux/build/ail-app-host`, or `AIL_ROOT/tools/app-host/windows/build/AILAppHost.exe` when that exists,
where the `ail` command sets `AIL_ROOT` to its own directory. The host is started with
`proc.start` (`std.proc.4`), which native builds don't compile yet, so a
program that uses `run` runs on the interpreter. *(ADR 0838)*

**std.app.11** — `script(mut a T, events List[Event]) List[str]`
passes each event to `update` in order, with no host, and returns
`outline(a.view())` after each one, so `ail test` covers an app's `update`
and `view` without a window. *(ADR 0838)*

**std.app.12** — `app.run_host`, and so `app.run`, has the effect `app`
(`eff.infer.4`) as well as `proc` and `io`; `app` is a capability, so
`ail run --deny app` rejects a program that opens a window
(`eff.cap.1`). `script`, `outline` and the builders have no effects.
*(ADR 0838)*

**std.app.13** — `trait App` also has `measures(self) List[Measure]` (default
empty) and `measured(mut self, runs List[Measure], widths List[float])`
(default does nothing). `struct Measure` is `text`, `font` (`mono`, `sans`, `serif`,
`web:` and a `Font` id, or a `font-family` list, std.app.28), `size`, `bold` and `italic`. After each `update`, while `measures()` is not
empty (at most 4 rounds), `run` sends `{"v":1,"t":"measure","runs":[...]}` with
the same fields per run, and waits for the host's `{"kind":"measured","text":W}`
event, where `W` is each run's advance width in points in the font a `Str` op
of that family, size and weight is drawn in, joined by commas; events that
arrive meanwhile are handled afterwards. If the count is wrong the answer is
dropped. `measured` gets the runs and widths, so the app lays out again. The
headless `script` never measures. *(ADR 0839)*

**std.app.14** — `run_host` reads the host's lines in a task and passes them,
with events other tasks post, through one channel, in arrival order.
`struct Inbox` has `post(ev Event)`; `external(source, data) Event` makes an
event with `kind` `external`, `id` the source and `text` the data. `new_inbox()`
makes an inbox; `run_with(mut a, cmd, args, inbox)` is `run_host` with it, so
the program starts the tasks that post (a server loop, a timer) before the
run and stops them after, in its own block, where their handles live.
`trait App` has `attach(mut self, inbox Inbox)` (default: nothing), called
once before the first `update`. `exited` (the host's pipe closed) fails `run`
with `host_exited`. Headless `script` has no inbox. *(ADR 0861)*

**std.app.15** — `struct Progress` has `done int`, `total int` (`0` when unknown) and `note str`. `progress(source, p) Event` makes an `external` event (`std.app.14`) whose `text` is `progress|done|total|note`; `progress_of(ev) Progress?` reads it back and gives `none` for any other event. *(ADR 0930)*

**std.app.16** — `trait App` has `hover(self) bool` (default `false`). While it is `true` (the `view` line carries `"hover":true`), a host sends `move` (a canvas's `id`, `x`, `y`) at most once per 16 ms with the last position, and `leave` (`id`). A host that can't track the pointer sends neither, so no app may need them. *(ADR 0931)*

**std.app.17** — `Event` has `clicks int`: 1, 2 or 3 for a `click`, `0` for other kinds, from the platform's multi-click timing. *(ADR 0932)*

**std.app.18** — `trait App` has `popup(self) Popup?` (default `none`); `struct Popup` is `id`, `x`, `y` and `items List[str]`. When it differs from the last one sent, `run` sends `{"v":1,"t":"popup","id":I,"x":X,"y":Y,"items":[...]}`; the host shows its native menu there and answers `{"kind":"popup","id":I,"text":LABEL}`, `text` empty if dismissed. A host with no menu answers dismissed. *(ADR 0932)*

**std.app.19** — `Op.Shade(x, y, w, h, color, alpha float)` fills a rectangle with opacity `alpha` (0 to 1, clamped) over what is drawn. A host skips an op it doesn't know. *(ADR 0933)*

**std.app.20** — A host sends `keyup` (a canvas's `id`, `text`, modifiers) when a key is released, and `key` carries `repeat` (auto-repeat) and `code` (the platform key code). A program that does not read them is unaffected; a host may omit both. *(ADR 0956)*

**std.app.21** — `trait App` has `lines(mut self) List[str]` (default empty).
After each `update`, `run` sends every returned line to the host unchanged and
the app's next call returns only new ones; it is how an app sends protocol
lines the tree cannot express, such as `surface` and `present`
(`std.gfx.6`). A host skips a line it doesn't know. *(ADR 0957)*

**std.app.22** — `trait App` has `pick(self) Picker?` (default `none`); `struct Picker` is `id`, `prompt`, `folders bool` (a folder, else a file) and `start` (the folder to open in, `""` for the host's choice). When it differs from the last one sent, `run` sends `{"v":1,"t":"pick","id":I,"prompt":P,"folders":F,"start":S}`; the host shows its native open dialog and answers `{"kind":"picked","id":I,"text":PATH}`, `text` empty if the user cancelled. A host with no native dialog answers `{"kind":"pick_error","id":I,"text":WHY}` at once, so an app can fall back to a text field. The app clears its `Picker` when it sees either event. *(ADR 0971)*

**std.app.25** — `trait App` has `edge(self) bool` (default `false`). While
it is `true` (the `view` line carries `"edge":true`), a host shows no title
bar and lets the app draw up to the window's top edge, keeping its own
window controls, and `start` and every `resize` carry `inset_left` and
`inset_top` (floats, points): the room those controls take at the window's
top-left, `0` when the host draws none there. The app moves the window by
sending the line `{"v":1,"t":"drag"}` through `lines()` (std.app.21) on a
mouse-down in its own strip, and `{"v":1,"t":"zoom"}` on a double-click
there. A host that cannot hide its title bar keeps its decorations and
reports both insets as `0`, so a program must not depend on edge mode.
*(ADR 0981)*

**std.app.33** — While `edge` is `true` (std.app.25), `start` and every
`resize` also carry `inset_right` (float, points): the room the host's
window controls take at the window's top-right, `0` when it draws none
there. `Event.inset_right` is `0.0` for every other kind and when the host
omits it. A host with its controls on the right (Windows, Linux) reports
`inset_right` and an `inset_left` of `0`; the macOS host reports
`inset_left` and `0`. An app keeps its strip between `inset_left` and
`width - inset_right`. *(ADR 1033, amends ADR 0981)*

**std.app.26** — An app puts text on the system clipboard by sending the
line `{"v":1,"t":"clipboard","text":T}` through `lines()` (std.app.21);
`app.clipboard(text)` is that line. The host copies `T` exactly as given
(UTF-8, no newline conversion). A host that cannot write the clipboard
ignores the line, as any unknown line, so a program must not depend on it.
*(ADR 0982)*

**std.app.27** — A host sends `down` when a mouse button is pressed in a
canvas and `up` when it is released, each with the canvas's `id`, `x`, `y`,
`text` (the button, as `click`), the modifiers and `clicks` (std.app.17;
`0` on `up`); `click` is sent as before, after `down`. The press captures
the pointer: until the release, the same canvas gets `move` (whether or not
hover is on, std.app.16) and the one `up` that follows every `down`, even
when the pointer leaves the canvas or the window, with coordinates relative
to that canvas (so possibly negative or past its size). A host may omit
`down` and `up` (then it sends neither), so a program must not depend on
them. *(ADR 0983)*

**std.app.31** — `start` and every `resize` carry `screen_w` and `screen_h`
(floats, points: the main display's size) and `scale` (float: device pixels
per point, `2` on a Retina display). A host reports the display it draws on
by default (macOS `NSScreen.main`, Linux the primary GDK monitor, Windows the
primary screen with its DPI); a headless event has all three `0`, and
`app.event_from_json` reads them like any member. A host that cannot tell
reports `0`, so a program must treat `scale == 0` as `1`. *(ADR 1017)*

**std.app.23** — A host sends `paste` (a canvas's `id`, `text` is the system
clipboard's text) when the paste key (Command-V on macOS, Control-V
elsewhere) is pressed in a canvas, instead of the `key` event for it; a
host whose clipboard it cannot read sends the `key` event as before, so a
program must not depend on `paste`. *(ADR 0978)*

**std.app.24** — `trait App` has `shown(self) Node` (default `view`). An attached
agent's screen model and `input` ids (`attach.wire.4`, `attach.input.2`) are built
from `shown`, so an app that draws somewhere its tree doesn't show, as a
`std.gfx` game does, can say what is on screen. *(ADR 0979)*

**std.app.28** — `Measure.font` and `Str.font` may be a CSS `font-family`
list: family names separated by commas, each optionally quoted. The host
measures and draws the text in the first family of the list that is
installed (matched without regard to case); when none is, in the list's
first generic family (`serif` is `serif`; `monospace` and `ui-monospace`
are `mono`; `sans-serif`, `system-ui` and `ui-sans-serif` are `sans`), and
with no generic either, in `sans`. All three hosts choose this way. The
`measured` event may carry `"fonts":F`, each run's chosen family in the same
order, joined by commas (a host that does not say omits it); `Event` has
`fonts` and `trait App` has `fonts_used(mut self, runs List[Measure], used List[str])`
(default does nothing), called after `measured` when the host said.
`family_list(font) List[str]` is the list's families, unquoted and in order,
and `generic_of(font) str` the face the list falls back to. *(ADR 1005)*

**std.app.30** — A host sends `appearance` (`text` is `dark` or `light`) once right after `start`, and again whenever the system's dark or light setting changes. An app may answer with a new view, as for any event. A host that cannot read the setting sends none, so a program must not depend on it and keeps a default theme. *(ADR 1008)*

**std.app.32** — Three ops draw anti-aliased vector shapes, in points, colors
`0xRRGGBB`. `Round(x, y, w, h, r, color, alpha, stroke float)` is a rounded
rectangle: `r` is clamped to 0 up to half the shorter side (so a circle is
`r = w / 2`), `stroke` 0 fills it and `stroke` above 0 outlines it with that
width, centred on its edge; `alpha` (0 to 1, clamped) is its opacity.
`Poly(points, color, alpha)` fills the polygon through `points`
(`x0, y0, x1, y1, ...`) with the non-zero winding rule; fewer than 3 points
draw nothing and a trailing odd value is ignored. `Line(points, color, width)`
strokes an open polyline with round caps and round joins; fewer than 2 points
draw nothing, and a `width` of 0 or less is 1. A host draws all three
with its platform's anti-aliasing at device resolution, and the display list
(the `view` JSON and `outline`) is the same on every host. A host that does not
know an op skips it (std.app.19). `app.outline` prints
`round X,Y WxH rR COLOR ALPHA STROKE`, `poly N COLOR ALPHA` and
`line N COLOR WIDTH` (`N` is the point count), and `std.screen` shows `Round`
and `Poly` as a `box` and `Line` as a `line` over their bounding boxes
(std.screen.12). *(ADR 1032)*

## Examples

```ail rules=std.app.1,std.app.2,std.app.4,std.app.5,std.app.6,std.app.11
use std.app

struct Counter
    n int

impl app.App for Counter
    fn update(mut self, ev app.Event)
        if ev.kind == "press" and ev.id == "up"
            n += 1

    fn view(self) app.Node
        app.column([app.label("count {n}"), app.button("up", "Up")])

fn main()
    mut c := Counter(n: 0)
    screens := app.script(mut c, [app.press("up"), app.press("up")])
    print(screens[1])
```

```ail rules=std.app.3,std.app.7,std.app.8
use std.app

struct Pad
    clicks int

impl app.App for Pad
    fn update(mut self, ev app.Event)
        if ev.kind == "click"
            clicks += 1

    fn view(self) app.Node
        app.canvas("pad", 0.0, 0.0, [app.draw_rect(0.0, 0.0, 20.0, 20.0, 0xff0000), app.draw_text(4.0, 4.0, "{clicks}", 0xffffff)])

    fn title(self) str
        "Pad"

fn main()
    mut p := Pad(clicks: 0)
    ev := app.event_from_json(parse_json("\{\"kind\":\"click\",\"id\":\"pad\",\"x\":3}") else Json.Null)
    p.update(ev)
    print(ev.x)
    print(app.frame(p).starts_with("\{\"v\":1,\"t\":\"view\",\"title\":\"Pad\""))
    print(app.outline(p.view()))
```

```ail rules=std.app.21
use std.app

struct Sender
    queue List[str]

impl app.App for Sender
    fn update(mut self, ev app.Event)
        queue.push("\{\"v\":1,\"t\":\"present\"}")

    fn view(self) app.Node
        app.label("sender")

    fn lines(mut self) List[str]
        sent := queue
        queue = []
        sent

fn main()
    mut s := Sender(queue: [])
    s.update(app.event("start"))
    print(s.lines().len())
    print(s.lines().len())
```

```ail rules=std.app.25
use std.app

## A tab strip that starts right of the window controls and moves the window.
struct Strip
    left float
    queue List[str]

impl app.App for Strip
    fn update(mut self, ev app.Event)
        if ev.kind == "start" or ev.kind == "resize"
            left = ev.inset_left
        elif ev.kind == "click" and ev.x >= left
            queue.push(if ev.clicks == 2 then "\{\"v\":1,\"t\":\"zoom\"}" else "\{\"v\":1,\"t\":\"drag\"}")

    fn view(self) app.Node
        app.canvas("strip", 0.0, 0.0, [app.draw_rect(left, 0.0, 200.0, 40.0, 0x35363a)])

    fn edge(self) bool
        true

    fn lines(mut self) List[str]
        sent := queue
        queue = []
        sent

fn main()
    mut s := Strip(left: 0.0, queue: [])
    mut start := app.event("start")
    start.inset_left = 78.0
    start.inset_top = 28.0
    s.update(start)
    print(app.frame(s).contains("\"edge\":true"))
    mut c := app.event("click")
    c.x = 100.0
    c.clicks = 1
    s.update(c)
    print(s.lines())
    print(app.outline(s.view()))
```

```ail rules=std.app.31
use std.app

## A page's `screen` and `devicePixelRatio`, from the display the host reports.
struct Page
    w float
    h float
    ratio float

impl app.App for Page
    fn update(mut self, ev app.Event)
        if ev.kind == "start" or ev.kind == "resize"
            w = ev.screen_w
            h = ev.screen_h
            ratio = if ev.scale == 0.0 then 1.0 else ev.scale

    fn view(self) app.Node
        app.label("{w}x{h} @{ratio}")

fn main()
    mut p := Page(w: 0.0, h: 0.0, ratio: 1.0)
    mut start := app.event("start")
    start.screen_w = 1512.0
    start.screen_h = 982.0
    start.scale = 2.0
    print(app.step(mut p, start))
```ail rules=std.app.30
## A view that follows the system theme: `appearance` picks the background.
struct Themed
    dark bool
impl app.App for Themed
        if ev.kind == "appearance"
            dark = ev.text == "dark"
        bg := if dark then 0x1f1f1f else 0xffffff
        app.canvas("c", 0.0, 0.0, [app.draw_rect(0.0, 0.0, 10.0, 10.0, bg)])
    mut t := Themed(dark: false)
    mut ev := app.event("appearance")
    ev.text = "dark"
    print(app.step(mut t, ev))
    ev.text = "light"
    print(app.step(mut t, ev))
```

```ail rules=std.app.33
use std.app

## A tab strip that stops left of the window controls on the right.
struct Tabs
    room float

impl app.App for Tabs
    fn update(mut self, ev app.Event)
        if ev.kind == "start" or ev.kind == "resize"
            room = ev.dx - ev.inset_right

    fn view(self) app.Node
        app.canvas("tabs", 0.0, 0.0, [app.draw_rect(0.0, 0.0, room, 32.0, 0x202124)])

    fn edge(self) bool
        true

fn main()
    mut t := Tabs(room: 0.0)
    mut r := app.event("resize")
    r.dx = 1000.0
    r.inset_right = 138.0
    r.inset_top = 32.0
    t.update(r)
    print(app.outline(t.view()))
    j := parse_json("\{\"kind\":\"resize\",\"dx\":900,\"inset_right\":138}") else Json.Null
    print(app.event_from_json(j).inset_right)
    print(app.event("start").inset_right)
```

```ail rules=std.app.27
use std.app

## A canvas that selects a span: the press starts it, moves stretch it, the
## release ends it.
struct Select
    from float
    to float
    held bool

impl app.App for Select
    fn update(mut self, ev app.Event)
        match ev.kind
            "down" =>
                from = ev.x
                to = ev.x
                held = true
            "move" => if held
                to = ev.x
            "up" => held = false
            _ => held = held

    fn view(self) app.Node
        app.canvas("s", 0.0, 0.0, [app.draw_rect(from.min(to), 0.0, (to - from).abs(), 20.0, 0x8ab4f8)])

fn main()
    mut s := Select(from: 0.0, to: 0.0, held: false)
    mut d := app.event("down")
    d.id = "s"
    d.x = 10.0
    d.text = "left"
    s.update(d)
    mut m := app.event("move")
    m.id = "s"
    m.x = 50.0
    s.update(m)
    mut u := app.event("up")
    u.id = "s"
    u.x = 50.0
    s.update(u)
    print((s.from, s.to, s.held))

```ail rules=std.app.28
use std.app

## A page's heading font: the host draws it in Georgia when that is
## installed, else in its serif face.
fn main()
    heading := "\"Nope Display 9z\", Georgia, serif"
    print(app.family_list(heading).join("|"))
    print(app.generic_of(heading))
    print(app.generic_of("ui-monospace, Menlo"))
    print(app.generic_of("Inter"))
```

```ail rules=std.app.26
use std.app

## A drawn field whose text Command-C (Control-C elsewhere) copies.
struct Field
    text str
    out List[str]

impl app.App for Field
    fn update(mut self, ev app.Event)
        if ev.kind == "key" and ev.meta and ev.text == "c"
            out.push(app.clipboard(text))

    fn view(self) app.Node
        app.canvas("f", 0.0, 0.0, [app.draw_text(4.0, 4.0, text, 0xffffff)])

    fn lines(mut self) List[str]
        sent := out
        out = []
        sent

fn main()
    mut f := Field(text: "ail.dev", out: [])
    mut c := app.key("c")
    c.meta = true
    f.update(c)
    print(f.lines())
    print(f.lines())
```

```ail rules=std.app.23
use std.app

## A one-line drawn field: typed keys add characters, a paste adds its text.
struct Omnibox
    text str

impl app.App for Omnibox
    fn update(mut self, ev app.Event)
        if ev.kind == "paste"
            text = "{text}{ev.text}"
        elif ev.kind == "key" and ev.text.chars().len() == 1
            text = "{text}{ev.text}"

    fn view(self) app.Node
        app.canvas("f", 0.0, 0.0, [app.draw_text(4.0, 4.0, text, 0xffffff)])

fn main()
    mut f := Omnibox(text: "")
    f.update(app.key("a"))
    mut p := app.event("paste")
    p.id = "f"
    p.text = "il.dev"
    f.update(p)
    print(f.text)
```

```ail rules=std.app.20
use std.app

struct Held
    keys int

impl app.App for Held
    fn update(mut self, ev app.Event)
        if ev.kind == "key"
            keys += 1
        elif ev.kind == "keyup"
            keys -= 1

    fn view(self) app.Node
        app.label("held {keys}")

fn main()
    mut h := Held(keys: 0)
    mut up := app.event("keyup")
    up.text = "a"
    screens := app.script(mut h, [app.key("a"), up])
    print(screens[0])
    print(screens[1])
```

```ail norun rules=std.app.13
use std.app

struct Ruler
    widths Map[str, float]

impl app.App for Ruler
    fn update(mut self, ev app.Event)
        return

    fn view(self) app.Node
        Label("{widths.get("hello") else 0.0}")

    fn measures(self) List[app.Measure]
        if widths.contains("hello")
            return []
        [app.Measure(text: "hello", font: "sans", size: 13.0, bold: false, italic: false)]

    fn measured(mut self, runs List[app.Measure], ws List[float])
        widths[runs[0].text] = ws[0]

fn main()
    print(Ruler(widths: {}).measures().len())
```

```ail norun rules=std.app.9,std.app.10
use std.app

struct Hello
    name str

impl app.App for Hello
    fn update(mut self, ev app.Event)
        if ev.kind == "edit"
            name = ev.text

    fn view(self) app.Node
        app.column([app.field("name", name, "your name"), app.label("Hello, {name}")])

fn main()
    mut h := Hello(name: "")
    app.run(mut h) else err
        print("no window: {err.kind}: {err.message}")
```

```ail rules=std.app.14
use std.app

fn main()
    ev := app.external("timer", "tick")
    print("{ev.kind} {ev.id} {ev.text}")
```

```ail tool=effects,--json,FILE expect="fn:main":["app" rules=std.app.12
use std.app

struct Quiet
    n int

impl app.App for Quiet
    fn update(mut self, ev app.Event)
        n += 1

    fn view(self) app.Node
        app.label("{n}")

fn main()
    mut q := Quiet(n: 0)
    app.run(mut q) else return
```

```ail rules=std.app.22
use std.app

struct Where
    path str
    asking bool

impl app.App for Where
    fn update(mut self, ev app.Event)
        if ev.kind == "press" and ev.id == "choose"
            asking = true
        if ev.kind == "picked"
            path = ev.text
            asking = false
        if ev.kind == "pick_error"
            asking = false

    fn view(self) app.Node
        app.column([app.label("scan {path}"), app.button("choose", "Choose folder")])

    fn pick(self) app.Picker?
        if asking
            return app.Picker(id: "folder", prompt: "Folder to scan", folders: true, start: "")
        none

fn main()
    mut w := Where(path: "", asking: false)
    app.script(mut w, [app.press("choose")])
    print(w.pick() != none)
    mut done := app.event("picked")
    done.id = "folder"
    done.text = "/tmp"
    app.script(mut w, [done])
    print("{w.path} {w.pick() != none}")
```

```ail rules=std.app.15,std.app.16,std.app.17,std.app.18,std.app.19
use std.app

struct Pad
    menu app.Popup?

impl app.App for Pad
    fn update(mut self, ev app.Event)
        if ev.kind == "click" and ev.text == "right"
            menu = app.Popup(id: "pad", x: ev.x, y: ev.y, items: ["Reveal", "Trash"])
        if ev.kind == "popup"
            menu = none

    fn view(self) app.Node
        app.canvas("pad", 0.0, 0.0, [app.Op.Shade(x: 0.0, y: 0.0, w: 4.0, h: 4.0, color: 0, alpha: 0.5)])

    fn hover(self) bool
        true

    fn popup(self) app.Popup?
        menu

fn main()
    ev := app.progress("scan", app.Progress(done: 3, total: 10, note: "a|b"))
    p := app.progress_of(ev) else app.Progress(done: -1, total: -1, note: "")
    print("{p.done}/{p.total} {p.note}")
    print(app.progress_of(app.external("scan", "other")) == none)
    mut pad := Pad(menu: none)
    print(app.outline(pad.view()))
    print(app.frame(pad).contains("\"hover\":true"))
    j := parse_json("\{\"kind\":\"click\",\"id\":\"pad\",\"text\":\"right\",\"clicks\":2}") else Json.Null
    c := app.event_from_json(j)
    print(c.clicks)
    pad.update(c)
    print(pad.popup() != none)
```

```ail rules=std.app.32
use std.app

fn main()
    ops := [
        app.Op.Round(x: 0.0, y: 0.0, w: 20.0, h: 20.0, r: 10.0, color: 0xff0000, alpha: 1.0, stroke: 0.0),
        app.Op.Poly(points: [0.0, 0.0, 10.0, 0.0, 5.0, 8.0], color: 0x00ff00, alpha: 0.5),
        app.Op.Line(points: [0.0, 0.0, 5.0, 5.0, 10.0, 0.0], color: 0, width: 2.0),
    ]
    print(app.outline(app.canvas("c", 20.0, 20.0, ops)))
```ail rules=std.app.3
use std.app

struct Face
    n int

impl app.App for Face
    fn update(mut self, ev app.Event)
        n += 1

    fn view(self) app.Node
        data := if n == 0 then "AAEAAAAA" else ""
        app.canvas("c", 0.0, 0.0, [app.Op.Font(id: "f", data: data), app.Op.Str(x: 0.0, y: 0.0, text: "Hi", font: "web:f", size: 20.0, fg: 0xffffff, bold: false, italic: false, underline: false)])

fn main()
    mut p := Face(n: 0)
    print(app.outline(p.view()))
```
