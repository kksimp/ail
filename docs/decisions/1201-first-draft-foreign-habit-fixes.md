# 1201. First-draft fixes for four habits: list swap, spread, two keyword names, `+` after a fallback `else`

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); diagnostics and fixes only, no language change
- **Topic:** 22 — Tools
- **Spec rules:** `gram.foreign.1`, `lex.kw.1`, `trait.op.1`, `expr.prec.1`, fix.edit.2

## Context

The Opus trial's first drafts and the first-draft survey left four mistakes that cost a retry each, though the
answer is known to the compiler:

- `xs[i], xs[j] = xs[j], xs[i]` stopped at "expected end of line" and said nothing about swapping. The existing
  help for multiple assignment covered only `a, b = b, a`.
- `[...xs, x]` had help but no fix.
- `error, any := f()` reported the first keyword name only; the second showed up after the fix, one run later.
- `m.get(k) else "?" + k` kept `trait.op.1` with no rewrite: [ADR 1175](1175-more-first-draft-certain-fixes.md) and
  [ADR 1199](1199-public-error-set-fix.md) rewrote `+` after an `else` only when it closes a `then`.

## Decision

- A multiple assignment whose targets are index expressions reports `gram.foreign.1` with the help "swap two list
  elements with `xs.swap(i, j)`". When the line is exactly `xs[i], xs[j] = xs[j], xs[i]` it has the fix
  `xs.swap(i, j)` (anchor `tuple_swap`). Not a certain fix: `xs` may be a map.
- `[...xs, x]`, `[x, ...xs]` and `[...xs, ...ys]` get the fix `xs.concat([x])`, `[x].concat(xs)`, `xs.concat(ys)`
  (anchor `spread_list`), when the list is on one line and each spread is a plain postfix expression.
- A declaration `a, b := e` reports every keyword among its names, each with the rename fix it already had.
- The `+` text rewrite starts after any `else`. `else` is the loosest operator (expr.prec.1), so `a else b + c` is
  `a else (b + c)` and `b` is a whole operand; ADR 1175's caution about `a else b` was about the end of an operand,
  which is unchanged. ADR 1199's `closes_then` is no longer needed and is removed.

New anchors live in `compiler/fix/foreignfix.ail`. Tests: `tests/fix/tuple_swap`, `spread_list`,
`keyword_names_two`, `plus_fallback`.

## Options considered

- A certain fix for the swap: rejected, a map index has no `swap`.
- A fix for `a, b = b, a` with a temporary: rejected, it adds a line and invents a name.
- Allowing `...xs` in list literals: a syntax change, not proposed.

## Consequences

Untested at writing; the Mac job `first-draft-3/job.sh` is the first run.
