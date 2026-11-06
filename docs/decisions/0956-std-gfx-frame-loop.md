# 0956. `std.gfx`: the frame loop for real-time programs

- **Status:** Accepted
- **Date:** 2026-11-05
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI; GPU
- **Spec rules:** `std.gfx.1`-`std.gfx.5` ([std-gfx.md](../spec/std-gfx.md))
- **Requested by:** the designer, who wants AIL to replace C, C++ and Rust, with real-time rendering that uses modern hardware well. Design: [gpu-frame-loop.md](../design/gpu-frame-loop.md). The headless half (`Game`, `Input`, `Frame`, `script`, `dump`) and the macOS host's `keyup` ship with this ADR; the window frame loop and vsync tick follow under the same ADR.

## Context

`std.app` is a retained tree over a JSON pipe (ADR 0838). It has no key-up, no vsync pacing, no gamepad, and cannot carry pixels at frame rate.

## Decision

New module `std.gfx` with `trait Game` (`update(mut self, input Input, dt float)`, `draw(self, mut f Frame)`) and `gfx.run`. `Input` is a per-frame value snapshot: keys down, pressed and released this frame, pointer, wheel, gamepads, text typed. Keys are named by physical code and by character. `gfx.script` runs scripted `Input`s with no window and returns frame dumps, so games are testable under `ail test`. The host sends `keyup` and a `repeat` flag on `key` (also to `std.app`), and a `vsync` tick that paces `run`.

## Rejected alternatives

- **Faster JSON in `std.app`.** Still parses every frame and cannot be vsync paced.
- **SDL or minifb.** ADR 0009 (custom tooling).
- **Retained scene graph as base API.** Hides cost from agents.

## Consequences

One more host protocol surface per platform. Mac first, then Windows and Linux hosts.
