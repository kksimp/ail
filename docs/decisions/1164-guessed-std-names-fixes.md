# 1164. Certain fixes and help for guessed std names and foreign forms

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

6. **More guessed names** (v2 replay lines): `trim_left` and `trim_right` rename to `trim_start` and `trim_end`
   (certain); `trim_matches`, `trim_start_chars` and kin get help (blank trim takes no argument; a given prefix is
   `strip_prefix(p) else s`); `int.MIN` and `int.MAX` get help writing the two literals.
7. **Help wording**: err.use.1 names the two lines to replace (`r := f()` then `if t := r`); err.else.2 and err.else.4
   say a handler block ends with `continue`, `break` or `return` (the replay's six `_ := f() else err` versions
   stopped there).

The v2 diff of the versions made clean but still wrong (95) shows no case caused by a fix of this ADR: they are logic
(the smallest `int`, task rules). The only fixed class that appears with a source of this ADR is `trim_start(' ')`
in three versions, a `#` test where `trim_start()` is identical.

Not done here: `err.use.1`, `type.coll.3`, `lex.str.2` hoisting, `expr.if.4` and the
lane 6 items (`err.str.2`, `err.else.1`, `err.else.4`).

## Rejected alternatives

- A certain fix for `trim_start("-")` to `strip_prefix`: the result is `str?`, so the program's meaning changes.
- A fix for `a.min()`: no constant exists, and a guessed literal is the `overflow-min` trap.
- New syntax for `a if c else b`: the identity rule forbids it.
