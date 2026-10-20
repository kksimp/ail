# 0177. `std.ui` is a window of text cells, shimmed with two pinned crates

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture; 23 — Compiler architecture (bootstrap dependencies)
- **Spec rules:** [std.ui.1–9](../spec/17-std.md), [eff.infer.1](../spec/18-effects.md), [eff.infer.4](../spec/18-effects.md)

## Context

The designer's stretch goal is a code editor with a windowed GUI, written in
AIL. AIL has no FFI in the bootstrap yet (ADRs 0103–0105 are designed, not
implemented), so the interpreter must provide the window. The bootstrap has
had no dependencies (ADR 0023: each crate needs a stated justification).
The deciding criterion is **AI understanding**: a grid of text cells with a
handful of methods is the smallest surface an agent can drive correctly, and
it is exactly what a text editor or a dashboard needs. Tokens: an editor's
draw loop is `put` calls, about as short as a terminal UI's.

## Options considered

### Option A — a text-cell grid in a window, `minifb` + `embedded-graphics` (chosen)
- **Pros:** one small API (`open`, `size`, `clear`, `put`, `cursor`,
  `present`, `next_event`, `poll_event`, `close`); real window on macOS,
  Linux (X11, loaded at run time, so headless builds need no libraries) and
  Windows. `minifb` brings 4 crates on Linux and none that touch the
  network; `embedded-graphics` is pure Rust and carries MIT-licensed bitmap
  fonts (10x20 Latin-1).
- **Cons:** two crates and their transitive dependencies in the bootstrap;
  no Wayland-native window (XWayland works); bitmap glyphs only.

### Option B — pixels and shapes (a canvas API)
- **Pros:** general.
- **Cons:** much larger API; an editor would have to lay out text itself;
  more tokens per program.

### Option C — terminal UI (raw mode, ANSI escapes)
- **Pros:** no dependencies.
- **Cons:** the designer chose a windowed GUI.

### Option D — platform APIs directly (Cocoa, Win32, Xlib) from Rust
- **Pros:** no third-party crates.
- **Cons:** three backends' worth of unsafe code for scaffolding that the
  real toolchain replaces (see [gui-path.md](../design/gui-path.md)).

### Option E — a hand-made bitmap font instead of `embedded-graphics`
- **Pros:** one crate fewer.
- **Cons:** hours of glyph data to draw and review; a worse font.

## Decision

`use std.ui` provides `ui.open(title, cols, rows) Window ! UiError` and the
`Window` methods, `Size`, `Event` (`Key`, `Resize`, `Close`) and
`UiError` of spec std.ui.1–9. The bootstrap implements them with
`minifb =0.28.0` (default features off, `x11` on) and
`embedded-graphics =0.8.1` (default features off), pinned exactly. `ui.*`
gives a new capability `ui` and the effect `io`, so `--deny ui` works.
Without a display, `ui.open` fails with `kind: "no_display"`; it never
panics, and nothing else touches the window code, so `./ail selftest` runs
headless.

## Rationale

The grid matches what the editor needs and nothing more, so it is easy to
learn from one spec page and easy to test headless: `put` and key
translation are pure functions with unit tests. The two crates are the
smallest pair that gives a real window on all three first-class targets;
they are scaffolding of the Rust bootstrap only. ADR 0009 and ADR 0010 govern
the real compiler and runtime, which will reach the platform through
`extern c` instead.

## Consequences

- AIL programs can open a window today; lane 12's editor builds on it.
- The bootstrap's `Cargo.lock` now lists crates; `cargo build` needs them
  cached once (it works offline afterwards).
- Wayland-only Linux desktops without XWayland get `no_display`.
- Follow-up: mouse events, clipboard, and a font size option; the path to a
  native layer is in [gui-path.md](../design/gui-path.md).

## Rejected alternatives

- **Canvas API:** too large a surface for the goal; more tokens per program.
- **Terminal UI:** not what the designer asked for.
- **Direct platform APIs:** large unsafe scaffolding that will be thrown away.
- **Hand-made font:** slow to make, worse result, saves one pure-Rust crate.
- **Newer `minifb 0.29`:** released 17 days ago; 0.28 has a longer record.
