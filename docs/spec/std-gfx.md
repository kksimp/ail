# Standard library: `std.gfx` real-time frame loop

Chapter key: `std`

> **Status:** Proposed (ADRs 0956-0958). Not implemented. Design:
> [gpu-frame-loop.md](../design/gpu-frame-loop.md).

**std.gfx.1** — `use std.gfx` imports `trait Game`, `Input`, `Frame`,
`Surface`, `GfxError` and `run`. `trait Game` has `update(mut self, input
Input, dt float)` and `draw(self, mut f Frame)`. *(ADR 0956)*

**std.gfx.2** — `run(mut g T, title str, w int, h int) ! GfxError` opens a
window and, once per display refresh, calls `update` with the latest `Input`
and the seconds since the last presented frame, then `draw`, then presents.
It has the effect `gfx`, a capability (`ail run --deny gfx`). *(ADR 0956)*

**std.gfx.3** — `Input` is a value: `down(key)`, `pressed(key)` and
`released(key)` (this frame), `text` typed, `pointer` (x, y, buttons),
`wheel`, `pad(n)` gamepad state. A key is named by character or physical
code (`KeyA`, `ArrowLeft`). Pure data, so tests build it by hand. *(ADR 0956)*

**std.gfx.4** — The host reports key release as event `keyup` and marks
auto-repeat on `key` with `repeat`; `std.app` receives both. *(ADR 0956)*

**std.gfx.5** — `script(mut g T, inputs List[Input], dt float) List[str]`
runs `update` and `draw` per input with no window and returns each frame's
`dump(f)`, the draw commands as text. It has no effects. *(ADR 0956)*

**std.gfx.6** — `stats(f Frame)` gives draw calls, vertices and upload bytes
of a frame. *(ADR 0956)*

**std.gfx.7** — `Frame.pixels() Surface` is the frame's RGBA8 buffer
(`w`, `h`, `stride`); writes show at the next present. *(ADR 0957)*

**std.gfx.8** — The program sends `{"v":1,"t":"surface","id":I,"w":W,
"h":H,"stride":S,"count":N}` and shares the buffers; per frame it sends
`{"t":"present","buf":K}`; the host answers `{"kind":"vsync","t":T,
"dt":D,"buf":K}`. *(ADR 0957)*

**std.gfx.9** — On macOS and Linux the region is a file under `$TMPDIR`,
mode 0600, random name, unlinked once both sides mapped it. On Windows it is
a `CreateFileMapping` handle with no path. *(ADR 0957)*

**std.gfx.10** — A host that cannot map the region fails `run` with
`GfxError` kind `no_host`. *(ADR 0957)*

**std.gfx.11** — A `par` loop or tensor op may run as a GPU compute kernel
only when the cost model says transfer plus launch beats the CPU; the result
equals the CPU's bit for bit for integers and within a stated tolerance for
floats. `ail explain` reports the choice. *(ADR 0958)*

**std.gfx.12** — Kernels lower to MSL on Metal and SPIR-V on Vulkan. *(ADR 0958)*

**std.gfx.13** — Render batches (`rect`, `sprite`, `text`, `mesh`) draw on the
GPU; Metal first, then Vulkan and Windows. *(ADR 0958)*

**std.gfx.14** — User shader functions are written in AIL and lowered by the
same backends. *(ADR 0958)*
