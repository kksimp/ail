# 1163. Certain fix for `str ! str`, and a field line for a message that names one

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer's rule of no syntax change to look like other languages (fixes, diagnostics and the card only)
- **Topic:** 3 — Diagnostics; 8 — Error handling
- **Spec rules:** err.str.2, err.str.4, err.else.2, err.msg.3
- **Amends:** [ADR 1140](1140-standard-text-failure.md) (err.str.2 gets a certain fix), [ADR 1129](1129-str-errors.md)

## Context

The replay of failed agent versions on main 832e86a1 left two parser-task causes with no certain fix: `fn run_line(...) str ! str`
(err.str.2, 11 versions, the largest single cause on the calculator task) and `error struct Bad "{msg}"` written without its field
(agents copy the first line of the card's example and drop the field wording, so `msg` is an unknown name).

## Decision

1. **err.str.2 certain fix.** In `str ! str` the body's last expression is the result and a `return` of text is the failure,
   as in `T ! str` (err.str.1). The fix writes `str ! Fail` and `return Fail(msg: TEXT)` for each `return`, when the body ends
   in a `str` value, has no bare `return`, and the program declares no `Fail` of its own. A `return` of a literal or interpolation that starts with an error word (a short fixed list)
   is a failure (`"syntax"`, `"unknown {name}"`); `""` and an interpolation that starts with a shown value (`"{v}"`) are results and
   stay; any other `return` withholds the fix. The replay shows both kinds in one function (`return ""`, `return "{v}"`, `return "syntax"`).
   Otherwise help only, now naming the result as the last expression.
3. **`!` with no result, `return TEXT`** (expr.block.2 in 13 replayed versions, after an agent hand-edits err.str.4's help to a bare
   `!`). Reported as err.str.4 at the `!` instead of expr.block.2, with the same certain fix (`! Fail`, `return Fail(msg: TEXT)`).
4. **err.else.2** help names a fallback (`""`, `0`, `0.0`, `false`) for the `_ := f() else err` block that does not leave. Help only.
5. **err.msg.3.** An `error struct` whose message shows `{msg}`, `{message}`, `{why}`, `{reason}`, `{text}`, `{detail}` or `{what}`
   alone, with no field of that name, is reported once at the message. The certain fix writes `name str` lines under the
   declaration. The message is not spliced into `impl ToStr` then, so `expr.name.1` does not repeat the mistake.
   Other names stay `expr.name.1`: a constant could be meant.
6. **Card.** The error-type line puts the fields on lines of their own under the message.

## Rejected alternatives

- **Treat every `return` of a variable or any interpolation in `str ! str` as a failure too.** `return acc` is as likely an early success;
  a wrong guess compiles and changes behaviour.
- **Fix the signature only (`str ! Fail`, returns unchanged).** Every failure would become a success value.
- **Guess the type of any missing message name.** `n` may be `int`; only text-like names get a certain `str` field.
