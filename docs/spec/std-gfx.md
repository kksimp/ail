# Standard library: `std.gfx` real-time frame loop

Chapter key: `std`

> **Status:** Drafted. The headless half (ADR 0956) and the CPU-drawn window
> loop on the macOS host (ADR 0957) are implemented; the GPU parts are planned
> (ADR 0958) and have no rule IDs until they ship. Design: [gpu-frame-loop.md](../design/gpu-frame-loop.md).

**std.gfx.1** — `use std.gfx` imports `trait Game`, `Input`, `Frame`, `Cmd`,
`Stats` and the functions below. `trait Game` has `update(mut self, input
Input, dt float)` and `draw(self, mut f Frame)`. `Frame` is an ordered list
of draw commands (`Cmd`: `Clear`, `Rect`, `Line`, `Text`), built with
`new_frame`, `clear`, `rect`, `line` and `text`. *(ADR 0956)*

**std.gfx.2** — `Input` is a value: `down` (every key held), `pressed` and
`released` (keys that changed since the previous frame), `text` typed, the
pointer (`x`, `y`, `buttons`) and `wheel`. `down(i, key)`, `pressed(i, key)`
and `released(i, key)` ask about one key; `idle()` is an input with nothing
held. A key is named as `std.ui.7` names it. Pure data, so tests build it by
hand. *(ADR 0956)*

**std.gfx.3** — `next_input(prev, events)` makes the next frame's `Input`
from the previous one and the `std.app` events since: `key` presses a key
(its auto-repeats do not press again), `keyup` releases it (`key_up(name)`
makes that event), `click` and `move` set the pointer, `scroll` adds to the
wheel. *(ADR 0956)*

**std.gfx.4** — `script(mut g T, w, h, dt, steps)` runs `update` and `draw`
for each step of events with no window, `dt` seconds apart, and returns
`dump(frame)` after each; `dump` is one line per command after a header
`frame WxH cmds=N`, and `cmd_text(c)` is one command's line. It has no
effects. *(ADR 0956)*

**std.gfx.5** — `stats(f)` gives `cmds`, `fills` (`Clear` and `Rect`),
`lines` and `texts` of a frame. *(ADR 0956)*

**std.gfx.6** — `use std.gfx.host` adds `run(mut g T, title str, w int, h int)
! GfxError` for any `T` that implements `Game`. It starts the app host
(`std.app.10`), shows a window with one canvas, and once per display refresh
runs `next_input`, `update` (with `dt` the seconds since the last shown frame,
at most 0.1), `draw`, and presents, until the window closes. The window is sized so that its canvas is exactly `w` by `h` points, one point per surface pixel (a Retina display shows each pixel on a 2 by 2 square). It has the effects
of `app.run`. A host that can't map the surface, or a missing host, fails with
`GfxError`. *(ADR 0957)*

**std.gfx.7** — The frame is drawn by the CPU into the back one of two RGBA8
buffers (`Clear`, `Rect` and `Line` in pixels; `Text` is drawn by the host over the surface, from the line `{"v":1,"t":"text","id":"gfx","items":[{"x","y","s","size","color"}]}` that `run` sends when the frame's text changes, so `pixels` shows none; anything
off the surface is dropped). The host shows a buffer as the whole canvas,
scaled. *(ADR 0957)*

**std.gfx.8** — The buffers live in one shared mapping that starts with `O` unused bytes (and ends with as many), so that no buffer begins at a page boundary where the runtime's block check would see the end of another block. The program makes a
file with `mkstemp` under `$TMPDIR` (mode 0600, random name), maps it
`MAP_SHARED`, and sends `{"v":1,"t":"surface","id":"gfx","w":W,"h":H,
"stride":S,"offset":O,"count":2,"path":P}` through `lines` (`std.app.21`); per frame it
sends `{"v":1,"t":"present","id":"gfx","buf":K}`. The host maps the file by
path, shows buffer K at its next draw, and answers each drawn frame with the
event `vsync` (`id`, `dx` the seconds since the previous one). The program
unlinks the file at the first `vsync`. A host skips a `surface` it can't map.
*(ADR 0957)*

**std.gfx.9** — `host.pixels(f Frame) List[int] ! GfxError` draws `f` the way
`run` does into a fresh surface and returns its pixels as `0xRRGGBB`, row by
row (`w * h` numbers), with no window: a test sees what would be shown. Alpha
is ignored.

**std.gfx.10** — `use std.gfx.watch` adds `run`, as `host.run` with the attach
listener on (`attach.wire.1`), for a game that implements `attach.Observe`.
`ail attach screen` then answers with the last frame's commands: `Clear` and
`Rect` as boxes, `Line` as lines, `Text` as text (`std.app.24`). `ail attach
observe` gives the game's observed values and `gfx.cmds`, `gfx.fills`,
`gfx.lines` and `gfx.texts` of the last frame. *(ADR 0979)*

**std.gfx.11** — `Input.pads` holds the connected gamepads (`pad(i, n)` is
gamepad `n`, or `no_pad()`). A `Pad` has `buttons`, `pressed` and `released`
(bit sets of `pad_bit(name)`: `a` 1, `b` 2, `x` 4, `y` 8, `lb` 16, `rb` 32,
`back` 64, `start` 128, `l3` 256, `r3` 512, `up` 1024, `down` 2048, `left`
4096, `right` 8192), sticks `lx`, `ly`, `rx`, `ry` from -1.0 to 1.0 (right and
up are positive) and triggers `lt`, `rt` from 0.0 to 1.0. `pad_down`,
`pad_pressed` and `pad_released` ask about one button. The host sends the
event `pad` (`id` the gamepad's number, `text` the state `buttons,lx,ly,rx,ry,
lt,rt`, empty when disconnected) when a gamepad's state changes, and
`pad_event(n, state)` makes one for a test. The macOS host reads gamepads with
GameController and the Linux host with evdev; a host that can't sends no
`pad` event, so `Input.pads` stays empty and every pad reads as `no_pad()`: a game
must not need a gamepad. *(ADR 0985)*

## Examples

```ail rules=std.gfx.11
use std.gfx

fn main()
    i := gfx.next_input(gfx.idle(), [gfx.pad_event(0, "1,0.5,-0.5,0,0,0,1.0")])
    print(gfx.pad_down(i, 0, "a"))
    print(gfx.pad(i, 0).lx)
    print(gfx.pad(i, 3).buttons == gfx.no_pad().buttons)
```

```ail rules=std.gfx.1,std.gfx.2,std.gfx.3,std.gfx.4,std.gfx.5
use std.app
use std.gfx

struct Dot
    x float

impl gfx.Game for Dot
    fn update(mut self, input gfx.Input, dt float)
        if gfx.down(input, "right")
            x += 10.0 * dt

    fn draw(self, mut f gfx.Frame)
        gfx.clear(mut f, 0)
        gfx.rect(mut f, self.x, 0.0, 2.0, 2.0, 0xff0000)

fn main()
    mut d := Dot(x: 0.0)
    steps := [[app.key("right")], [gfx.key_up("right")]]
    frames := gfx.script(mut d, 16, 8, 1.0, steps)
    print(frames[1])
    i := gfx.next_input(gfx.idle(), [app.key("a")])
    print(gfx.pressed(i, "a"))
    f := gfx.new_frame(4, 4)
    print(gfx.stats(f).cmds)
```

```ail norun rules=std.gfx.6,std.gfx.7,std.gfx.8
use std.gfx
use std.gfx.host

struct Ball
    x float
    v float

impl gfx.Game for Ball
    fn update(mut self, input gfx.Input, dt float)
        x += v * dt
        if x > 300.0 or x < 0.0
            v = 0.0 - v

    fn draw(self, mut f gfx.Frame)
        gfx.clear(mut f, 0x000000)
        gfx.rect(mut f, x, 100.0, 20.0, 20.0, 0xff8800)

fn main()
    mut b := Ball(x: 10.0, v: 120.0)
    host.run(mut b, "ball", 320, 240) else err
        print("{err.why}")
```

```ail rules=std.gfx.7,std.gfx.9
use std.gfx
use std.gfx.host

fn main()
    mut f := gfx.new_frame(4, 3)
    gfx.clear(mut f, 0x112233)
    gfx.rect(mut f, 1.0, 1.0, 2.0, 1.0, 0xff0000)
    gfx.line(mut f, 0.0, 0.0, 3.0, 0.0, 0x00ff00)
    px := host.pixels(f) else err
        print(err.why)
        return
    print(px.len())
    print(px[0])
    print(px[5])
    print(px[11])
```

```ail rules=std.gfx.10,std.app.24
use std.attach
use std.gfx
use std.gfx.host

struct Dot
    n int

impl gfx.Game for Dot
    fn update(mut self, input gfx.Input, dt float)
        n += 1

    fn draw(self, mut f gfx.Frame)
        gfx.rect(mut f, 1.0, 1.0, 2.0, 2.0, 0xff0000)

impl attach.Observe for Dot
    fn observed(self) List[attach.Observed]
        [attach.observed("n", "int", "{n}")]

fn main()
    mut l := host.new_loop(Dot(n: 0), "dot", 8, 8)
    _ := host.advance(mut l, 0.1)
    mut tr := attach.Tracker.new()
    tr.update(l.observed())
    print(tr.observe(["gfx.cmds"], 0))
```

```ail rules=std.gfx.7
use std.gfx
use std.gfx.host

fn main()
    mut f := gfx.new_frame(40, 20)
    gfx.text(mut f, 2.0, 3.0, "hi", 12.0, 0xff0000)
    print(host.text_line(f))
    print(host.text_line(gfx.new_frame(4, 4)))
```
