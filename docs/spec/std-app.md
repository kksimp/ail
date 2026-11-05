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
`mono`, `sans` or `serif`, ADR 0905; `(x, y)` is the text's top-left corner),
`Fill(x, y, w, h, color)`, `Stroke(x1, y1, x2, y2, color, width)`,
`Pic(x, y, w, h, src)` (an image file path or URL),
`Bitmap(x, y, w, h, id, pw, ph, data)` (a `pw` by `ph` image scaled to the
rectangle; `data` is its `R, G, B, A` bytes in standard base64, which
`bitmap_data(rgba List[int])` makes; the host keeps the image under `id`, so an
empty `data` draws the one already sent, and a host that has none under that
`id` draws a placeholder), `Clip(x, y, w, h)`
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
(default does nothing). `struct Measure` is `text`, `font` (`mono`, `sans` or `serif`),
`size`, `bold` and `italic`. After each `update`, while `measures()` is not
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
