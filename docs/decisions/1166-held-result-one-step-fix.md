# 1166. A certain fix for `r := f(x)` then `if t := r`

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer's rule of no syntax change to look like other languages (fixes, diagnostics and the card only)
- **Topic:** 3 — Diagnostics; 8 — Error handling
- **Spec rules:** err.use.1
- **Amends:** [ADR 1130](1130-certain-fixes-applied-by-check.md) (one more certain fix), [ADR 1164](1164-guessed-std-names-fixes.md)

## Context

The replay of failed versions on main 832e86a1 has 14 versions with err.use.1, the largest leftover cause on the
ledger task: `r := parse_line(l)` and then `if t := r` with `else why`. The checker already knows the shape (it holds
the local's name while checking the `let`), and the help names the one-step form, but no fix existed.

## Decision

When a `let` of a `T ! E` value is the line before an `if` whose only condition is `t := r`, the fix writes
`if t := f(x)` in place of the `let` and removes the old `if` line. It is certain only when: the `let` is not `mut`
and has no `else`; the `if` is on the next line at the same indent with one condition; `r` is named nowhere else (the
`if`'s blocks, its `else`, the rest of the block; an `if` or `match` or a loop in those places counts as naming it,
since nested blocks are not read); and `t` is not `r`. Otherwise the help stays, and now says to delete the line.

## Rejected alternatives

- A fix that keeps `r` and rewrites only the `if` (`if t := r` stays): `r` still holds an unhandled failure.
- Rewriting when `r` is used later, by keeping both forms: it would call `f(x)` twice.
