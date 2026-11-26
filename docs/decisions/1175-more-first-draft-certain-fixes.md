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
- **lex.str.2, `"{" + x + "}"`.** A `{` followed by `"` was read as an interpolation of the string literal `" + x + "`
  whenever a `}` came later on the line, and built with the wrong text and no error. When the braces would hold only
  one literal with no interpolation of its own, or two literals with nothing between them, the brace is text and gets
  the certain `\{` fix.
- **trait.op.1, `+` on text** is rewritten to an interpolation in more places: after the `{` of an interpolation, the
  `|` that ends a closure's parameters, a `:`, a comparison or `then`; and before `}`, a comprehension's `for` or `if`,
  `==`, `!=`, or the `else` of `if c then a + b else d`. Each neighbour binds looser than `+`. Not after `else`: the
  left operand may then be `a else b`.
  A string literal whose only braces are `\{` and `}` now joins as written, so `"\{" + body + "}"` becomes
  `"\{{body}}"` in the round after the build escapes its `{`.
- **`unreachable()`** and a bare **`unreachable`** (a match arm, a last line) get the certain fix
  `unreachable("unreachable")`, reported as expr.call.1.
- **expr.loop.3, `(a..b).rev()`** (also `reversed`, `reverse`) is written `[k for k in a..b].reversed()`, with a
  variable the line does not use. **expr.call.2, `xs.count(v)`** with a whole number or a name of the element type is
  written `xs.count(|x| x == v)`. Both were help only (mistakes `rs_range_rev`, `count_value`).
- **lex.stmt.1, `return a, b`** is written `return (a, b)` when the values end on that line.
- **lex.kw.1:** `defer := e`, `as := e` and `requires := e` are reported as a keyword used as a name (they were
  gram.syntax.1, bootstrap.unsupported and contract.decl.1). `Set()`, `List()` and `Map()` get help naming `Set[int]()`.
- **expr.bind.4:** `_` is never a declared name, so `for _ in 0..n` inside another, and two `_` clauses in one
  comprehension, are accepted. `_` binds nothing (eff.use.1), so no name is shadowed.

## Options considered

- **`unreachable()` with an optional message in std:** rejected; a second form of one call, and the fix teaches the
  one form. The added text is fixed, so the rewrite has one meaning.
- **`+` joining strings:** rejected by the designer's rule (no syntax to match other languages) and type.str.3.
- **Every empty `{}` certain:** rejected; `print("{}", x)` and `"{}".format(x)` are placeholders with their own fix.
- **A `+` rewrite after `else`:** rejected until the rewrite reads the syntax tree and not the line.
- **A descending range form (`b..a` by -1):** a language change; not done. The list costs an allocation, and the
  help names it.
- **`[...xs, x]`:** no certain fix; the joined form depends on the std lane's choice for `xs + ys`.
- **`xs[i], xs[j] = xs[j], xs[i]` as `xs.swap(i, j)`, and two keyword locals on one line:** not done here; both need
  the statement parsed further than recovery does now.
- **`s.find(x, start)` and `+` on lists:** left to the std lane (its PRs carry them).

## Consequences

- `compiler/syntax/{lex,parse,recover}.ail`, `compiler/fix/{rewrite,literal,edit,sure}.ail`,
  `compiler/check/{call,expr,body,vocab,foreign}.ail`.
- Spec type.str.9 no longer says AIL has no default arguments (expr.call.4 has them).
- Mistakes pairs `string_brace_empty_pair`, `string_plus_comprehension`, `string_plus_if_value`,
  `string_plus_in_interpolation`, `string_plus_brace_text`, `unreachable_no_message`, `unreachable_bare_arm`; case `underscore_nested`.
- Not built in the cloud. The runner builds, regenerates the fix, explain and api goldens and the cards, and replays
  the rerun's failed first drafts.
