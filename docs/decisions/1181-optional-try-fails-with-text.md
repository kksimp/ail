# 1181. `?` on an optional fails with text in a `T ! str` or `Fail` function

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** Designer (chose it on the 2026-11-26 decision card, "All three"; proposal by the checker rough-edges lane). The failure text was proposed as `absent`; the AI chose `a value is absent`, the text `Absent` already shows (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 8 — Error handling
- **Spec rules:** [err.prop.6](../spec/08-errors.md), [err.prop.2](../spec/08-errors.md), [err.prop.5](../spec/08-errors.md)
- **Amends:** [ADR 0279](0279-question-mark-on-optionals-fails-with-absent.md), [ADR 1129](1129-str-errors.md), [ADR 1140](1140-standard-text-failure.md)

## Context

`n := s.to_int()?` worked in a function declared `!int` (the absence becomes the error `Absent`, ADR 0279) and was
err.prop.2 in one declared `int ! str` or `int ! Fail`. Those two are the forms the card teaches first for a function
that fails with a message, and their whole point is that the author does not design an error type. The author's fix
was `else return "..."` on every optional, which is right when there is something to say and noise when there isn't.

## Decision

- **err.prop.6:** in a function declared `T ! str`, or whose stated errors include the standard `Fail`, and that
  doesn't return an optional, `e?` on an optional gives the value or fails with a text: the context `m` of `e? m`
  when written, else `a value is absent`. In `T ! str` the text is the failure itself (what `return TEXT` does);
  with `Fail` the failure is `Fail(msg: TEXT)`.
- The context is the text, not a note beside it: a text failure has no other content, and the caller's `else e`
  then reads what the author wrote (`bad port: http`).
- Functions that state other error types keep err.prop.2: the function has to say which of its errors an absence is.
- A function literal or local function inside such a function keeps err.prop.2; the rule reads the declaration.
- Native: `compiler/mir/lower_absenttext.ail`, one call from the absent path of `lower_try`. No `Absent` tag is set,
  so a caller that infers its errors sees a text failure or a `Fail`, never an `Absent`.

## Options considered

- **Keep err.prop.2 with a fix that writes `else return "..."`:** rejected; the fix would have to invent the message.
- **Fail with `Absent` in every function that can fail:** rejected. A function that states its error types would
  fail with a type it doesn't state (err.prop.3 exists to prevent that).
- **Text `absent`:** rejected for `a value is absent`, which `Absent` already prints, so both forms read alike.
- **Attach the context as context (err.ctx.2) and keep the generic text:** rejected; a handler that prints `e` would
  show `a value is absent` and lose the one thing the author wrote.

## Consequences

- `compiler/check/{expr,textfail}.ail`, `compiler/mir/lower_absenttext.ail`, three lines in `lower_err.ail`.
- Case `absent_text_fail`. err.prop.2's message names the new form. The explain output needs regenerating.
- Programs accepted before mean the same; more programs are accepted.
- Not built in the cloud (ADR 0842); gated on the Mac.
