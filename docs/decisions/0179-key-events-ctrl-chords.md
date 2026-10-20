# 0179. Keys are typed characters, named keys, or Ctrl chords; Command is Ctrl on macOS

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture
- **Spec rules:** [std.ui.7](../spec/17-std.md)

## Context

`Event.Key(key str, ctrl bool, alt bool, shift bool)` must serve both text
entry (layouts, Shift, accents) and shortcuts (Ctrl-S). Platforms report a
typed character and a physical key separately, and with Ctrl held the typed
character is a control code or nothing. The deciding criterion is **AI
understanding**: one obvious way to match a shortcut (`Key("s", true, _, _)`)
and no duplicate events.

## Options considered

### Option A — text from character input; Ctrl chords and named keys from key codes (chosen)
- **Pros:** typed text follows the layout; a shortcut is the lowercase key
  with `ctrl`, the same on every layout-independent shortcut; exactly one
  event per key press.
- **Cons:** Ctrl chords use the US position of punctuation keys.

### Option B — every key press as a key code, text separately
- **Pros:** complete.
- **Cons:** two events per letter; every program must filter.

## Decision

Without Ctrl, a text key gives the typed character; with Ctrl, it gives the
key's unshifted character (`"s"`, `"."`) and no typed character. Named keys
(`enter`, arrows, `f1`–`f12`, …) always come from key codes. Modifier keys
alone give nothing. On macOS, Command reports as `ctrl`, so the same program
answers Cmd-S on a Mac and Ctrl-S elsewhere. Key presses are read from the
platform's input callback in order, so a press and release between two pumps
is not lost.

## Rationale

It gives one event per press and one spelling per shortcut, which an agent
can match without knowing the platform.

## Consequences

- A program can't tell Command from Control on macOS. Telling them apart
  later needs a new variant or method, since a new field would break
  existing patterns.
- Alt with a letter gives the typed character (`å` for Alt-A on macOS) with
  `alt` true.

## Rejected alternatives

- **All key codes plus text:** duplicate events; every program filters.
- **Command as its own flag now:** would change the API fixed for lane 12.
