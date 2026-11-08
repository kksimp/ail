# 0979. `std.gfx` windows publish the attach socket

- **Status:** Accepted
- **Date:** 2026-11-07
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI; 20 — AI semantic API
- **Spec rules:** `std.gfx.10`, `std.app.24`, `attach.run.4`
- **Amends:** [ADR 0884](0884-observe-field-and-attach-protocol.md), [ADR 0957](0957-shared-surface-transport.md)
- **Requested by:** the designer: an agent should see what a running AIL window shows without screenshots.

## Context

Headless, `host.pixels`, `gfx.stats` and `gfx.dump` show a frame as text. A live window opened by `std.gfx.host.run` published no attach socket, because the attach shim (ADR 0884) rewrites only `app.run(` calls, so `ail attach screen` found nothing.

## Decision

- `std.gfx.watch.run` is `host.run` with the attach listener on, for a game that implements `attach.Observe`.
- The attach shim also rewrites `host.run(` (after `use std.gfx.host` on a line of its own) to `std.gfx.watch.run(` and adds `impl attach.Observe` for each non-generic `impl gfx.Game for T` (`attach.run.4`).
- `trait App` gets `shown(self) Node` (default `view`, `std.app.24`). The attach screen model and `input` ids are built from `shown`. The game window's `shown` is a canvas of the last frame's commands as ops (`Clear` and `Rect` fills, `Line` strokes, `Text` strings), so `ail attach screen` lists boxes, lines and text without a vision model. The commands are kept in the program; none go over the host pipe.
- `observe` reports the game's `@observe` values and `gfx.cmds`, `gfx.fills`, `gfx.lines`, `gfx.texts` of the last frame.
- `host.advance` runs one frame with no window, so a test builds a loop, advances it and asks the screen model and `attach.answer` what an agent would see.

## Rejected alternatives

- **Put the last frame's ops in `view`.** The host would receive them as JSON every frame, undoing ADR 0957.
- **Have `host.run` always serve the socket.** Every game would link the listener and the network module; attach is off by default for `ail build` (ADR 0884).
- **A second protocol for games.** The same `screen` and `observe` queries already say what an agent needs.

## Consequences

Text is reported as a string at its position but is not yet drawn by the CPU surface (`std.gfx.7`). Windows has no native sockets yet, so a Windows game window runs without the listener (as `std.attach.serve.run` already does).
