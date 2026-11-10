# 0985. `std.gfx` gamepad input

- **Status:** Accepted
- **Date:** 2026-11-08
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI; GPU
- **Spec rules:** `std.gfx.11` ([std-gfx.md](../spec/std-gfx.md))
- **Amends:** [ADR 0956](0956-std-gfx-frame-loop.md)
- **Requested by:** the designer, who wants AIL games to take a gamepad as well as keys.

## Context

`Input` (ADR 0956) holds keys and the pointer. Games need gamepads, and the frame loop must stay testable with no window.

## Decision

- `Input.pads` is a list of `Pad` values: a button bit set (`buttons`) with `pressed` and `released` edges since the previous frame, two sticks (-1.0 to 1.0, right and up positive) and two triggers (0.0 to 1.0). Buttons have fixed bits (`pad_bit`: a, b, x, y, lb, rb, back, start, l3, r3, up, down, left, right), the layout of an extended gamepad.
- The host sends a `pad` event (`id` the pad's number, `text` the state `buttons,lx,ly,rx,ry,lt,rt`, empty when disconnected) when a pad's state changes; `next_input` folds it into the next frame's `Input`, so edges last exactly one frame.
- The macOS host polls `GameController` with each drawn frame. The Linux and Windows hosts send no `pad` event today: pads read as not connected, which the spec says; a game must not need one. They get evdev and XInput/GameInput later under this ADR.
- `pad_event(n, state)` makes the event for a test; `tests/cases/std_gfx_pad` feeds pad state through `next_input` and through the window loop (`host.new_loop`, `advance`) with no window.

## Rejected alternatives

- **One event per button.** More lines per frame and no stable view of sticks and buttons together.
- **Pad state in the `vsync` event.** Couples gamepads to the surface transport and sends state when nothing changed.
- **Stubbing gamepads on Linux and Windows with fake pads.** Hides the missing support from games; an empty list is honest.
- **Dead zones in the library.** Hardware differs; the game applies its own.

## Consequences

Eight pads at most. A pad that disconnects reads as `no_pad()` the frame it goes. Rumble and gyro are not covered.
