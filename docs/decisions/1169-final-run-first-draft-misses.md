# 1169. Fixes and help for the final run's first-draft compile misses

- **Status:** Accepted
- **Date:** 2026-11-19
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer's rule of no syntax change, with fixes, diagnostics and the card only
- **Topic:** 3 — Diagnostics
- **Spec rules:** [type.str.9](../spec/04-types.md), [expr.if.4](../spec/05-expressions.md)
- **Amends:** [ADR 1130](1130-certain-fixes-applied-by-check.md) (certain fixes applied by check)

## Context

The final paid run had 32 AIL first drafts that did not compile, against 4 for C. Ten printed an integer padded with
zeros. The compiler already had a certain fix for `n.pad_left(2, '0')`, but only when the receiver was a bare name;
for `(a % 100).pad_left(2, '0')` or `r.cents.pad_left(3, '0')` it printed help and failed the build. Other trials
guessed `n.to_string_pad()` (no help at all), declared the helper as `fn int.name(self)` (a bare trait.method.5), wrote
`b.to_int()` on a `BigInt` (a did-you-mean, not certain), or `xs.to_list()` on a list.

## Decision

- type.str.9: the certain fix for a pad method on an `int` works for any receiver that is one postfix expression
  (name, field chain, call, index, parenthesised expression): the receiver becomes `"{recv}"`. New edit anchor
  `int_text`, settled against the source line like `join_swap`.
- An `int` method whose name contains `pad` or `zfill` (and `rjust`, `ljust`) gets help naming `"{x:02}"` and
  `"{x}".pad_left(2, '0')`. No fix: the width and fill are not known.
- trait.method.5 (a method declared on a built-in type) gets help: write a free function.
- `BigInt.to_int()` has a certain fix to `as_int()`; `xs.to_list()` on a list has a certain fix that drops the call.
- expr.if.4 for branches that differ only by one being optional gets help: type the binding `x T? := if ...`.

## Options considered

- **Add `int.pad_left` to std**: rejected; padding is a `str` operation (type.str.9) and a second spelling widens the
  language for one guess. The fix teaches the existing form.
- **Certain fix for guessed pad names**: rejected; the width is not in the call.
- **Move `to_int` on `BigInt` to a std alias**: rejected; `as_int` gives `int?` and a second name would hide that.

## Consequences

- `compiler/check/call.ail`, `checker.ail`, `expr.ail`; `compiler/fix/{edit,resolve,rewrite}.ail`.
- Mistakes pairs `int_pad_left_paren`, `int_pad_left_field`, `bigint_to_int`, `list_to_list`; rule-only
  `int_to_string_pad`, `if_branch_plain_then_opt`.
- Not gated yet: nothing was built in the cloud. Remaining misses (one-line `if` in scripted edits, local names hiding
  modules, a missing `use`) need no change here: scripted edits are the agents' own, and those diagnostics already
  print their fix.
