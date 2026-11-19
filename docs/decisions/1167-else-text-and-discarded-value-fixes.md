# 1167. Certain fixes for `else "text"` on a failing function and for a discarded value with a block

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer's rule of no syntax change (fixes and help only)
- **Topic:** 3 — Diagnostics; 8 — Error handling
- **Spec rules:** err.else.1, err.else.2, err.else.5 (reported as err.else.4)
- **Amends:** [ADR 1130](1130-certain-fixes-applied-by-check.md), [ADR 1135](1135-self-read-store-and-text-failure-fixes.md), [ADR 1164](1164-guessed-std-names-fixes.md)

## Context

The v2 replay of failed calculator versions left three shapes with no certain fix: `return a.checked_add(b) else "overflow"`
in a function that fails with text (err.else.1), `v.checked_sub(0) else return "overflow"` as a statement on an optional
(err.else.4, only `T ! E` had the `_ := ` fix), and `run_line(...) else err` with a block that reports and carries on
(err.else.4, then err.else.2 after the agent wrote `_ :=`).

## Decision

1. **err.else.1.** `x else TEXT` where `x` is not text, inside a function declared `T ! str` (or with the standard `Fail`, and a
   literal or interpolation the fix can wrap): the text is the failure. Certain fix `else return TEXT` (`return Fail(msg: TEXT)`).
   Any other enclosing function gets the report only.
2. **err.else.4 on `T?`.** The `_ := ` fix applies as it does for `T ! E`, when the `else` leaves or gives a fallback.
3. **err.else.4 with a carrying-on block, and err.else.2 after `_ :=`.** The value is thrown away, so the fallback is never read:
   the fix writes `_ := ` and ends the block with `""`, `0`, `0.0` or `false` for a `str`, `int`, `float` or `bool` value, when the
   block's last statement is an assignment, a binding or a call. Any other value type or last statement: help only.
4. **own.param.1** needs no change: the replay shows the one remaining version clean after intake since ADR 1147.

## Rejected alternatives

- **Fallback for `x := f() else err` with a block that carries on.** The agent may want `x` to be 0, or may want to leave; only a
  discarded value makes the literal certain.
- **`else return TEXT` in any function.** Outside a function that fails with text, `return TEXT` is a different mistake.
