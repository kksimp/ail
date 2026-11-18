# 1147. Certain fixes after the calculator rerun: a stored-into parameter, `pass`, `Fail` gaps, `% -1`

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer's rule of no syntax change to look like other languages (fixes, diagnostics and the card only)
- **Topic:** 3 — Diagnostics; 8 — Error handling
- **Spec rules:** own.param.1, gram.foreign.1, err.str.2, err.str.4, type.num.12
- **Amends:** [ADR 1130](1130-certain-fixes-applied-by-check.md) (own.param.1's certain fix is no longer limited to functions with no result; `pass` is no longer rewritten to `todo`), [ADR 1140](1140-standard-text-failure.md) (two gaps in err.str.4's fix)

## Context

The calculator-only rerun (36 trials, main 0384e848) left these causes of failed AIL versions: a parameter stored
into without `mut` (7 of 25, always in functions with a result, where the fix was withheld to protect a function
that changes its own copy), a certain fix that turned `pass` into `todo` (a program that worked now stopped),
err.str.4's fix withheld in two shapes, an err.str.2 that also reported err.prop.1 with a wrong fix line, and
`checked_rem(-1)` taught only by the full card.

## Decision

1. **own.param.1 (supersedes the no-result limit).** `mut` on the parameter is certain when the function has no
   result, or when the parameter is stored into and is not read after the first store in source order, and no store sits in a loop that
   also reads it (a compound store counts as a read). A store nothing reads again can only have been meant for the
   caller. Otherwise the help names both forms as before.
2. **gram.foreign.1 `pass`.** A certain fix must not add a stop. The fix deletes the line when `pass` is alone on its
   line and the block has another statement; a `pass` that is the only statement, or shares its line, gets help only.
3. **err.str.4.** A `?` that ends a function err.str.4 rewrites, or a branch of its tail, uses no `str` result when that
   caller is rewritten too. A string inside an interpolation of a returned text no longer withholds the fix: the end of
   the text is found past the interpolation.
4. **err.str.2.** A function that reports err.str.2 reports no err.prop.1 on its `?`s.
5. **`% -1`.** The core card says `a % -1` is `0` and `checked_rem(-1)` is `some(0)`, never `none`; the `ail ask` line and
   type.num.12 say the same and name the test for overflow.

## Rejected alternatives

- **Make `checked_rem(-1)` give `none`.** Matches habit, but `checked_rem` would fail where `%` never does. Designer's call; not taken.
- **Certain fix for every store into a parameter.** A function that stores then returns the parameter wants its own copy.
- **`todo` for a lone `pass`.** It stops the program when reached.
