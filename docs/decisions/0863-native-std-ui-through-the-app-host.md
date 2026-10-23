# 0863. Native `std.ui` and widgets render through the `std.app` host: one window path for edit, pdf and Garlic

- **Status:** Accepted
- **Date:** 2026-10-23
- **Decided by:** Kaleb (option 1 on the decision card, 2026-10-23 11:33 UTC)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.ui.1`–`24` ([17-std.md](../spec/17-std.md)); `std.app.9`, `std.app.10` ([std-app.md](../spec/std-app.md)); `ffi.native.1`
- **Builds:** [ADR 0177](0177-std-ui-text-grid-window.md), [ADR 0178](0178-window-host-on-main-thread.md), [ADR 0838](0838-std-app-native-windows.md), [ADR 0839](0839-std-app-text-measure.md)

## Context

Native builds cannot call C (`ffi.native.1`), so the interpreter's built-in
`ui.Window` (ADR 0177, drawn by the interpreter's own window thread) has no
native counterpart. `tools/edit` and `tools/pdf` use `std.ui` and
`std.ui.widgets`; Garlic already draws through `std.app` (ADR 0838). Native
`proc.start` and, since the RR-3 concurrency work, tasks, channels,
`TaskGroup` and cancellation compile natively, so `app.run_host` and an
inbox of host events run in a native program.

`std.ui.widgets` is plain AIL: its `render` methods return `Run`s and need no
window. Only `draw`, `draw_runs` and the programs' own loops touch `ui.Window`.
The `Window` API is pull-based (`next_event` blocks), while `app.App` is
push-based (`update` is called per event).

## Options considered

### Option A — `ui.Window` becomes an AIL module over `std.app` (chosen)
- **Pros:** one host (`tools/app-host/macos`), one protocol, one place for
  fonts, clipboard, resize and focus; edit, pdf and Garlic share it; no new C
  or Swift surface; `outline` and `script` keep working headless; native and
  interpreter run the same AIL.
- **Cons:** a bridge from push to pull is needed; the grid is drawn as a
  canvas of `Str` and `Fill` ops, so a full-grid redraw costs JSON bandwidth
  (mitigated by sending only when the view line changes, std.app.9, and by
  run-length merging of cells with equal style).

### Option B — a second native window host for the text grid
- **Pros:** the cell grid maps one to one onto a native text view.
- **Cons:** two hosts and two protocols to keep in step on three platforms;
  Garlic and the editors would not share fonts or input handling.

### Option C — native C calls into the platform toolkit
- **Pros:** no helper process.
- **Cons:** blocked by `ffi.native.1`; needs callbacks that AIL does not have
  (`ffi.decl.3`).

## Decision

Native `std.ui` and `std.ui.widgets` render through the `std.app` host. A
`ui.Window` is a cell grid held in AIL. `present` turns the grid into one
`Canvas` of `Str` and `Fill` ops in `mono`, and `next_event` and `poll_event`
read host events from an inbox channel fed by an `app.run_host` loop running
in a task of the window's `TaskGroup`. Host events map to `ui.Event`:
`key` to `Key`, `click` to `Mouse`, `scroll` to `Scroll`, `resize` to
`Resize` (cells from `cell_w`, `cell_h`), `close` to `Close`. `clipboard_*`,
`set_title`, `set_font_size` and `set_cursor` become host messages or app
state. One window path serves `tools/edit`, `tools/pdf` and Garlic. macOS
first; Windows and Linux follow with their hosts.

## Rationale

The designer's priorities are native self-hosting first, then Garlic parity.
Reusing the host Garlic already needs removes a whole backend from the
native work, and the tasks and channels needed for the pull-to-push bridge
already exist natively.

## Consequences

- `tools/edit` and `tools/pdf` become buildable natively once the bridge
  lands; their native coverage is tracked in the roadmap.
- `std.ui.1`–`18` keep their meaning. Spec text that says the interpreter
  owns the window thread changes to "the host draws" in the same change as
  the code.
- Pixel-exact control of a text grid is the host's; the interpreter's own
  window thread is retired once the native path is complete.
- Follow-ups: a native `std.ui` module in AIL (own `compiler/mir/lower_*.ail`
  only if the compiler needs a hook, ADR 0823); coverage of `tools/edit` and
  `tools/pdf`; a Windows and Linux host.

## Rejected alternatives

- **Option B:** two hosts double the maintenance and diverge in input and fonts.
- **Option C:** impossible under `ffi.native.1` and `ffi.decl.3`.
- **Keeping `std.ui` interpreter-only:** `tools/edit` and `tools/pdf` would
  block native coverage and the removal of the interpreter (ADR 0855).
