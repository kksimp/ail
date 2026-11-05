# Standard library: `std.gfx` real-time frame loop

Chapter key: `std`

> **Status:** Drafted. The headless half is implemented (ADR 0956); the
> window, surface and GPU parts are planned (ADRs 0957, 0958) and have no
> rule IDs until they ship. Design: [gpu-frame-loop.md](../design/gpu-frame-loop.md).

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
