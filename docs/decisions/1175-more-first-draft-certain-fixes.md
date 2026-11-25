# 1175. More certain fixes for first drafts: empty braces, `+` on text in more places, `unreachable`, `_`

- **Status:** Accepted
- **Date:** 2026-11-25
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer's rule of no syntax change, with fixes, diagnostics and the card only
- **Topic:** 3 — Diagnostics
- **Spec rules:** [fix.auto.1](../spec/22-tools.md), [lex.str.2](../spec/02-lexical.md), [expr.bind.4](../spec/05-expressions.md), [trait.op.1](../spec/10-traits.md), [err.model.2](../spec/08-errors.md)
- **Amends:** [ADR 1138](1138-string-slip-certain-fixes.md), [ADR 1172](1172-first-draft-build-fixes.md)

## Context

The 80-trial rerun on v0.2.0 had 24 AIL first drafts that were not green, 21 of them compile errors, against 10 for C
(1.35 versions a trial against 1.12). The causes were forms, not names, and the card already states the two largest.
Better help text did not matter: 23 of 24 trials fixed the error on the next try. A try is saved only when the build
rewrites the form itself. The optional-typing causes are ADR 1173; this ADR takes the rest.

## Decision

- **lex.str.2, an empty `{}` that nothing can fill** is literal braces, and its `\{}` fix is certain. Nothing can fill
  it when the string is not followed by `,` (an argument), `.` (`.format(x)`) or `%`. ADR 1138 left every empty pair a
  guess; in all 4 failing trials the string was a whole argument or value.
- **trait.op.1, `+` on text** is rewritten to an interpolation in more places: after the `{` of an interpolation, the
  `|` that ends a closure's parameters, a `:`, a comparison or `then`; and before `}`, a comprehension's `for` or `if`,
  `==`, `!=`, or the `else` of `if c then a + b else d`. Each neighbour binds looser than `+`. Not after `else`: the
  left operand may then be `a else b`.
  A string literal whose only braces are `\{` and `}` now joins as written, so `"\{" + body + "}"` becomes
  `"\{{body}}"` in the round after the build escapes its `{`.
- **`unreachable()`** and a bare **`unreachable`** (a match arm, a last line) get the certain fix
  `unreachable("unreachable")`, reported as expr.call.1.
- **expr.bind.4:** `_` is never a declared name, so `for _ in 0..n` inside another, and two `_` clauses in one
  comprehension, are accepted. `_` binds nothing (eff.use.1), so no name is shadowed.
- **Help only:** `s.find(x, start)` names `s.slice(start).find(x)`; `[x] + xs` names `extend` and `push`.

## Options considered

- **`unreachable()` with an optional message in std:** rejected; a second form of one call, and the fix teaches the
  one form. The added text is fixed, so the rewrite has one meaning.
- **`+` joining strings:** rejected by the designer's rule (no syntax to match other languages) and type.str.3.
- **Every empty `{}` certain:** rejected; `print("{}", x)` and `"{}".format(x)` are placeholders with their own fix.
- **A `+` rewrite after `else`:** rejected until the rewrite reads the syntax tree and not the line.
- **`find(x, start)` in std:** left to the std lane; help only here.

## Consequences

- `compiler/syntax/lex.ail`, `compiler/fix/rewrite.ail`, `compiler/check/{call,expr,body}.ail`.
- Mistakes pairs `string_brace_empty_pair`, `string_plus_comprehension`, `string_plus_if_value`,
  `string_plus_in_interpolation`, `string_plus_brace_text`, `unreachable_no_message`, `unreachable_bare_arm`; rule-only `list_plus_list`,
  `str_find_start`; case `underscore_nested`.
- Not built in the cloud. The runner builds, regenerates the fix, explain and api goldens and the cards, and replays
  the rerun's failed first drafts.
