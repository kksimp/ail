# GUI path: from the `std.ui` shim to a native layer

> Non-normative design note, 2026-10-20 (lane 11). The normative rules are
> `std.ui.1–9` in [spec 17-std](../spec/17-std.md); the decisions are
> [ADR 0177](../decisions/0177-std-ui-text-grid-window.md),
> [ADR 0178](../decisions/0178-window-host-on-main-thread.md) and
> [ADR 0179](../decisions/0179-key-events-ctrl-chords.md).

## Where it stands

The Rust bootstrap implements `std.ui` with two crates: `minifb` for the
window, framebuffer and keyboard, and `embedded-graphics` for a 10x20
bitmap font. The main thread hosts the window and the interpreter thread
keeps the cell grid (ADR 0178). This is scaffolding: the real toolchain has
no Rust in it, links no third-party libraries it doesn't need (ADR 0009),
and has no external backend (ADR 0010).

What the AIL program sees is the part that lasts: `ui.open`, the eight
`Window` methods, `Event` and `UiError`. Programs written against it (the
editor, dashboards) keep working when the layer underneath changes.

## The real toolchain

`std.ui` becomes an ordinary AIL module in the standard library, written in
AIL, that reaches the platform through `extern c` (ADRs 0103–0105) inside
`unsafe` blocks (ADR 0102, ADR 0106), with a safe API on top. Per target:

| Target | Window and events | Drawing | Notes |
|---|---|---|---|
| macOS ARM64 | AppKit through the Objective-C runtime (`objc_msgSend`, `NSApplication`, `NSWindow`, `NSView`) | Core Graphics bitmap context, or Core Text for real fonts | main thread only; the runtime's task scheduler must keep the UI on thread 0 |
| Windows x86-64/ARM64 | Win32 (`CreateWindowExW`, a message loop, `WM_CHAR`, `WM_KEYDOWN`) | GDI `StretchDIBits`, later DirectWrite | `WM_CHAR` gives layout-correct text, as minifb's callback does |
| Linux | Wayland (`wl_surface`, `xdg_shell`, `wl_shm`) first, X11 (`libX11` or raw protocol over a socket) as fallback | software framebuffer in shared memory; xkbcommon for keymaps | libc is linked only when C is used (ADR 0105), which a GUI program does |

The cell grid, dirty-cell diffing and key translation are already pure
logic (`bootstrap/src/ui.rs`) and port directly to AIL. The font becomes an
AIL constant table generated from an MIT-licensed bitmap font, then real
outline fonts through the platform text APIs.

## What the shim leaves behind

- **Threads.** The host thread and channel exist only because the bootstrap
  interpreter runs off the main thread (ADR 0178). In the real runtime the
  program's first task runs on the main thread, and `std.ui` calls are
  direct. The rule that one window is open at a time can then be lifted.
- **`Window` as a handle int.** In the real language `Window` is a unique
  resource (a non-copyable value with a destructor that closes it); today it
  is a struct with an `int` that can be copied or forged. Programs that
  treat it as unique (one variable, passed as `mut`) keep working.
- **Effects.** The `ui` capability and `--deny ui` stay as they are.
- **Fonts and glyphs.** Latin-1 only, 10x20 pixels, no size option, `?` for
  everything else. A native layer adds a font size, full Unicode via the
  platform's text system, and wide-character cells.
- **Clipboard.** Today it shells out to `pbcopy`, `wl-copy`, `xclip` or
  PowerShell (ADR 0240); a native layer uses `NSPasteboard`, the Win32
  clipboard API and the Wayland/X11 selection protocols.
- **Missing pieces to add additively:** modifier state on mouse events
  (for Shift-click), HiDPI scaling, a `meta` key distinct from Ctrl on
  macOS (ADR 0179), and IME composition. Mouse and scroll arrived as new
  `Event` variants (ADR 0240), which broke `match`es that listed every
  variant; batch any further variants into one change.

## Risks

- A new `Event` variant is a breaking change for programs that match every
  variant without `_`. Batch future variants into one change.
- Wayland has no server-side decorations by default; a Linux native layer
  needs client-side decorations or `xdg-decoration`.
