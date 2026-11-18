# 1163. Certain fixes and help for guessed std names and foreign forms

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer's rule of no syntax change to look like other languages (fixes, diagnostics and the card only)
- **Topic:** 3 — Diagnostics
- **Spec rules:** expr.name.1, expr.call.1, lex.stmt.1
- **Amends:** [ADR 1130](1130-certain-fixes-applied-by-check.md) (more certain fixes, same tiers)

## Context

The replay of the E14 first versions left causes with three or more versions and no certain fix: `some(x)` written as
a constructor (`line.char_at(k) == some(' ')`), `str` methods from other languages (`"0123".index_of(c)`),
`trim_start(' ')` with an argument, `a == a.min()` for the smallest `int`, and `x if c else y`.

## Decision

1. **`some(x)`** is a call hint like `Some(x)`: certain fix to the bare `x` (an optional is its value, type.opt.5);
   `opt == x` compares a `T?` with a `T` by expr.cmp.1 and type.opt.5.
2. **`str.index_of`, `str.position`** are certain renames to `find` (same receiver and argument, both `int?`);
   `list.position` renames to `index_of`.
3. **`s.trim_start(' ')`** (also `trim`, `trim_end`, and `" "`, `'\t'`): certain fix drops the blank-literal argument,
   which `trim_start()` already means. Any other argument is help only, naming `strip_prefix(p) else s` (type changes).
4. **`a.min()` / `a.max()` on an `int` with no argument** is help only: it names `a.min(b)` and writes the smallest and
   largest `int` literals, since there is no constant.
5. **`return A if C else B`** (lex.stmt.1) gets help writing `if C then A else B`; no fix, because the parser has no
   expression span at that point.

Not done here, held for the v2 replay lines: `err.use.1`, `type.coll.3`, `lex.str.2` hoisting, `expr.if.4` and the
lane 6 items (`err.str.2`, `err.else.1`, `err.else.4`).

## Rejected alternatives

- A certain fix for `trim_start("-")` to `strip_prefix`: the result is `str?`, so the program's meaning changes.
- A fix for `a.min()`: no constant exists, and a guessed literal is the `overflow-min` trap.
- New syntax for `a if c else b`: the identity rule forbids it.
