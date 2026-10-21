# 0838. `std.app`: native windows from a view tree, drawn by a generic host over JSON lines

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** Designer (option chosen on a decision card, 2026-10-21); details by AI (delegated, ADR 0076) within the integrator's constraints
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.1`–`12` ([std-app.md](../spec/std-app.md)); `eff.infer.1`, `eff.infer.4`
- **Builds:** [ADR 0177](0177-std-ui-text-grid-window.md), [ADR 0178](0178-window-host-on-main-thread.md), [ADR 0424](0424-streaming-child-processes.md), [ADR 0432](0432-c-calls-through-two-register-files.md)

## Context

The designer asked whether AIL should bind a SwiftUI front end to an AIL
back end on macOS, and wants GUI apps (first: the Garlic browser in a native
window). `std.ui` is a text-cell window (ADR 0177). Native builds can't call
C yet (`ffi.native.1`), and AIL has no callbacks or exported functions
(`ffi.decl.3`). The proposal is in the project's
`notes/swiftui-binding-options.md`.

## Decision

1. **An app is a value of a type implementing `app.App`:** `update(mut self,
   ev Event)` and `view(self) Node`, plus `title` and `focus` with
   defaults. The view tree (`Node`) and a canvas's display list (`Op`) are
   plain enums; `Event` is one flat struct with a `kind`. No closures or
   callbacks cross any boundary.
2. **A generic host draws the tree.** It is a separate program that knows no
   app: `tools/app-host/macos` is SwiftUI plus one AppKit view for `Canvas`,
   built by one self-setting-up `run.sh` (ADR 0071), with no third-party
   packages, ARM64 only (ADR 0004). It is never built by the Linux selftest.
3. **Transport v1 is newline-delimited JSON over the host's standard input
   and output**, with `"v":1` on the first line each way. The AIL program
   starts the host with `proc.start` (`std.proc.4`), from `AIL_APP_HOST` or
   the build under `AIL_ROOT` (which `./ail` exports). Events in, whole
   views out, sent only when the view's line changed. A missing or dying host
   is a typed `app.AppError`, never a trap.
4. **A headless mode is part of the API:** `outline` renders a tree as text
   and `script` drives `update` and `view` with a list of events, so `ail
   test` and agents check apps without a window. The selftest runs the real
   protocol against a fake host written in AIL
   (`bootstrap/tests/app/fake_host.ail`).
5. **A new capability `app`** (`std.app.12`), given by `app.run_host` and
   so `app.run`, deniable with `--deny app`. `std.ui`'s `ui` is not reused.
6. **`std.app` is AIL source** (`std.ui.19`), so the interpreter and the
   bytecode tier run it unchanged. Native builds wait for native
   `proc.start`, whose child streams are channels and need native tasks and
   channels (PR #194) first; until then a program that calls `run` runs on
   the interpreter.

### Delegated details

- Builders (`app.button(id, caption)`, `app.field`, `app.toggle`,
  `app.canvas`, `app.label`, `app.column`, `app.row`, `app.draw_text`,
  `app.draw_rect`) exist because a variant with two fields of one type
  can't be built by position (`type.struct.7`), and an enum variant needs
  its enum name where the type isn't known (`type.enum.3`). A builder call
  is shorter than the named form in every case.
- `Button` and `Toggle` name their text `caption`, and the op builders are
  `draw_*`, because derived `ToJson` code binds field names as locals, which
  may not shadow a module item (`expr.bind.5`).
- Commands from `update` (set the title, move the focus) are declarative
  trait methods read after each update, not a command list: `title()` and
  `focus()`. The host acts when the value changes.
- `resize` carries the canvas's monospace cell size, so a cell-based layout
  (Garlic's) needs no measuring round trip in v1.
- Whole views, not diffs: Garlic's 200-run screens are a few kilobytes, and
  unchanged views aren't sent. Keyed diffs are a compatible v2 change
  (reserved ADR 0839), as is host-side text measuring.

## Consequences

- An app is written in AIL only; an AI writes one language and tests the
  app headless. The same tree can be drawn by Windows and Linux hosts later
  with no change to apps.
- Controls are limited to the node set; custom views need new nodes.
- The pipe adds latency per event (sub-millisecond locally), fine for forms
  and a browser, not for 60 fps games, which need a separate in-process
  layer (proposal addendum, `std.gfx`, not decided).

## Rejected alternatives

- **(a) A Swift shell calling an AIL library through a C header.** Needs
  exported AIL functions, a dylib or static-library output with a Mach-O
  export table, and callbacks, none of which exist; every app is written in
  two languages, roughly double the tokens; each OS needs its own shell.
- **(b) AIL generating SwiftUI glue from type declarations.** All the costs
  of (a) plus a code generator, and generated Swift is hard for an agent to
  repair.
- **(c) AppKit through `objc_msgSend` behind `std.ui`.** Needs native FFI
  and callbacks first, and reaches AppKit only: SwiftUI has no C or
  Objective-C ABI.
- **(d) `std.ui` only.** No native controls; apps stay a text grid.
- **An in-process transport for v1.** Needs export and callbacks (`ffi.decl.3`);
  it remains the v2 option behind the same AIL API.
