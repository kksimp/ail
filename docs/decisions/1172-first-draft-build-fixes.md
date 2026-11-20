# 1172. Certain fixes that make a failed first draft build

- **Status:** Accepted
- **Date:** 2026-11-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer's rule of no syntax change, with fixes, diagnostics and the card only
- **Topic:** 3 — Diagnostics
- **Spec rules:** [trait.method.5](../spec/10-traits.md), [expr.name.1](../spec/05-expressions.md), [expr.call.2](../spec/05-expressions.md), [expr.bind.1](../spec/05-expressions.md), [type.coll.4](../spec/04-types.md)
- **Amends:** [ADR 1130](1130-certain-fixes-applied-by-check.md), [ADR 1169](1169-final-run-first-draft-misses.md)

## Context

A free replay of the final run's 29 failed first drafts on the frozen tree built only 6. A try is saved only when the
first draft builds, and much of ADR 1169 was help, which makes the second draft right but saves no try. This ADR adds
only certain rewrites, each with one reading, so the first draft builds after repair on intake.

## Decision

- **trait.method.5, a helper on a built-in type.** `fn int.pad2(self) str` (owner `int`, `float`, `str`, `bool` or
  `char`, or a foreign spelling of one) is rewritten to `fn pad2(x int) str`, with `self` written `x` through the
  body (in code and in interpolations; `v` or `value` when `x` is taken). Only for a plain `self`, no generics, and a
  name no item, prelude function or std method of the type has. Edit anchor `method_decl`.
- **A function called as a method.** `x.f(a)` where `f` is a function of the module whose first parameter has the
  receiver's type is written `f(x, a)` (expr.name.1). Edit anchor `method_call`. With the rule above this makes the
  helper habit a two-round repair: the declaration, then its calls.
- **Std spellings with one meaning.** `abs_val` and `absolute` on a number or `BigInt` are `abs`; `xs.get(i)` on a
  list is `xs.skip(i).first()`; `c.to_int() - 48` on a `char` is `c.code() - 48` (only when `- 48` follows, since then
  the code is meant; a bare `c.to_int()` stays help, as it may mean `to_digit`). Anchor `char_code`.
- **expr.call.2, a list of one.** `m.push(k, [x])` on a list-valued map, or any call where `[x]` is given and `x` is
  wanted, loses the brackets. Anchor `unbracket_one`; dropped for a comprehension or a list that is not the whole
  argument.
- **expr.bind.1.** `(a, b) := t` is `a, b := t`. Anchor `unparen`.
- **type.coll.4.** `xs[a else d]` is `xs[(a else d)]`. Anchor `paren_index`.

All are certain fixes (ADR 1130) and are settled against the source: an edit that does not match its anchor drops its
fix and the diagnostic is as before.

## Options considered

- **Help only** (ADR 1169): rejected for these cases; it does not save the try.
- **Rename every unknown std name by nearest match**: rejected (ADR 1130); a nearest name is a guess.
- **`to_int` on a `char` always `code()`**: rejected; `to_digit(10)` is as likely. Only the `- 48` reading is certain.
- **`xs.get(i)` as `xs[i]`**: rejected; it traps where `get` gives `none`. `skip(i).first()` has the same meaning and
  may copy the tail; the fix is for the first draft to build, and the help names the cheaper `i < xs.len()` test.
- **Local `out` hiding module `out` (expr.bind.5), `T ! str` (err.str.4), `if c then a else b else d` (expr.if.4)**:
  not done; each has more than one reading or needs edits at every return site.

## Consequences

- `compiler/fix/freefn.ail` (new), `edit.ail`, `resolve.ail`; `compiler/check/{checker,call,relate,help_opt}.ail`;
  `compiler/syntax/parse.ail`.
- Mistakes pairs `int_helper_method`, `tuple_paren_bind`, `map_push_list_of_one`, `index_else`, `char_to_int_digit`,
  `list_get_else`, `bigint_abs_val`.
- Not built in the cloud. The runner builds, regenerates goldens and cards, and replays the 29 drafts (bar 13 building)
  and the 802 finals (no regression).
