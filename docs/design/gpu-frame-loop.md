# Real-time rendering: frame loop, pixels, GPU (design note)

> **Status:** Proposed, non-normative. No ADR yet (numbers come from the
> integrator). Goal from the designer (2026-11-05): 60 fps and above, using
> modern GPUs and many cores well, enough for games, AI-first.

## Why `std.app` cannot do it

`std.app` is a retained view tree sent as JSON lines over a pipe
(std.app.7). A canvas redraw re-serialises its whole display list and the
host re-parses it. A 1080p frame as `Bitmap` base64 is about 11 MB of text.
There is also no frame pacing (the program does not know when the display
refreshes), no key-up event (`key` is key-down only, std.app.5), no gamepad,
and the Linux and Windows hosts lag the Mac one. The roadmap sketch in
`swiftui-binding-options.md` already says games need a separate in-process
layer, `std.gfx`, sharing input types with `std.app`.

## Shape: `std.gfx`, one trait, three tiers

Same trait pattern as `std.app`, so an agent learns one shape:

```ail skip=context
trait Game
    fn update(mut self, input gfx.Input, dt float)   # fixed or variable step
    fn draw(self, mut f gfx.Frame)                    # record this frame
fn main()
    gfx.run(MyGame(), title: "demo", size: (1280, 720))
```

- `Input` is a value snapshot per frame: keys down, `pressed` and `released`
  this frame (so key-up is a first-class fact, not an event to miss),
  pointer, wheel, gamepads, text typed. Pure data, so `ail test` can feed
  scripted `Input` values and assert on `Frame` contents headlessly (the
  AI-first part: a game is testable and a frame is dumpable as text with no
  window, like `app.outline`).
- `Frame` is a draw-command list, never pixels in the AIL heap, with three
  tiers behind one API:
  1. **Surface** (step 1): a CPU pixel surface `Frame.pixels() Surface`
     (width, height, stride, RGBA8). AIL writes bytes in place; the host
     shows it. Needs no GPU, runs everywhere.
  2. **Batches** (step 2): `rect`, `sprite`, `text`, `line`, `mesh` batches
     recorded as flat arrays and drawn by the GPU (Metal on macOS,
     Vulkan on Linux and Windows, D3D12 later).
  3. **Shaders** (step 3): user shader functions written in AIL and lowered
     to MSL and SPIR-V (already decided: SPIR-V/MSL; no external compiler).
- Rendering never blocks the update: `draw` records, a render thread submits.
  Recording uses the parallel runtime, so independent draw lists (per tile,
  per entity group) fill on many cores and merge in submission order. The
  scheduler rule is the language's: use as many cores as help, never more.

## Transport (the key change)

The pipe stays for control, but pixels and draw data never go through JSON.

- The program creates a **shared memory region** (file-backed, in
  `$TMPDIR`, mapped by both processes) holding two or three frame buffers.
  The control line `{"v":1,"t":"surface","id":I,"w":W,"h":H,"stride":S,
  "path":P,"count":N}` tells the host to map it.
- Per frame the program writes into the buffer not being shown, then sends a
  tiny `{"t":"present","buf":K}` line. The host swaps and replies with the
  next vsync tick `{"kind":"vsync","t":T,"dt":D,"buf":K}` (display link),
  which is also the pacing signal: `gfx.run` blocks on it, so frames are
  vsync-locked and `dt` is the real presentation delta.
- In-process mode (host as a library loaded through native FFI, no pipe at
  all) is the later target; the surface and batch layouts stay identical so
  nothing in user code changes.

Runtime gap: the native runtime only has anonymous `sys_mmap`
(`compiler/rt/native/core.ail`). Needs a file-backed, shared `mmap`
primitive (`open`, `ftruncate`, `mmap MAP_SHARED`) on macOS, Linux and
Windows (`CreateFileMapping`), exposed as an effect-checked `std.gfx`
internal, not general user code.

## Input

Add to the host protocol: `keyup` (`NSEvent.keyUp`, Win32 `WM_KEYUP`, GTK
key-release), `keydown` gets a `repeat` flag, and keys are named by
physical code as well as by character (games bind WASD by position).
`gamepad` events come from GameController.framework (macOS), XInput and
GameInput (Windows), evdev (Linux). `std.app` gets `keyup` too, since the
event type is shared.

## Plan (smallest first)

1. **Mac frame loop and surface.** Host: `keyup`, `repeat`, vsync tick via
   `CADisplayLink` / `CVDisplayLink`, a surface view backed by a shared
   buffer shown through a `CALayer` (zero copy where the layer can wrap the
   buffer). Runtime: file-backed shared mmap. Library: `std.gfx` with
   `Game`, `Input`, `Surface`, `run`, headless `script`. Demo: a bouncing
   sprite field that measures frame time.
2. **Metal batches** on the same API; instanced sprites, textures, text
   atlas, then `mesh`. Target 144 fps with 100k sprites on an M-series Mac.
3. **Vulkan and the Windows host** with the same batch layout; D3D12 only if
   Vulkan on Windows proves a problem.
4. **Shader functions in AIL** lowered to MSL and SPIR-V, then compute for
   offload (shares work with the GPU compute decision already made).
5. **Audio and gamepad** (`play`, `stream`), asset loading.

Each step ships with a headless test (scripted `Input` in, frame dump out),
a frame-time benchmark in `tools/bench`, and rule IDs in a new
`docs/spec/std-gfx.md`.

## AI-first properties

- Everything observable as text: `gfx.dump(frame)` lists the draw commands,
  `gfx.stats(frame)` gives draw calls, vertices, upload bytes, so an agent
  can reason about performance without a GPU capture.
- `ail check` can flag draw calls inside loops that should be batched, with
  a fix suggestion, the way it flags other cost problems.
- Frame time budget is data: `Frame.budget()` returns the ms left, and a
  debug build reports which phase (update, record, submit, present) missed
  the budget.
- No hidden global state: the window, input and surface are values passed
  in; an agent can fake all of them.

## Rejected alternatives

- **Faster JSON for `std.app`.** Even binary-in-JSON copies and parses every
  frame; it cannot be vsync paced and keeps the host in the hot loop.
- **Embed SDL or minifb.** Violates custom tooling by default (ADR 0009) and
  adds a C dependency to every game.
- **Write the Metal and Vulkan renderers in Swift/C++.** The AIL promise is
  that apps are written in AIL; only the thin window and device glue lives
  outside, and the renderer's command encoding is AIL.
- **Retained scene graph as the base API.** Hides cost; the immediate
  command list is simpler for an agent to reason about and still allows a
  retained layer on top.

## Open points for the designer

- File-backed shared memory in `$TMPDIR` vs a new `shm` capability: the
  former is simplest; confirm.
- Fixed-step `update` default (60 Hz accumulator) vs variable `dt`. Proposal:
  fixed step by default, `dt` always passed.
