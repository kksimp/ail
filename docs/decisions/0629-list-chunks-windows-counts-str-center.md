# 0629. List `chunks`, `windows`, `counts`, `flatten`; string `center`, `words`, `trim_start`, `trim_end`, `count`, `rfind`

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 4 — Types
- **Spec rules:** type.coll.17-19, type.str.16-18 ([04-types](../spec/04-types.md))

## Context

Lane 192's parity survey: Python agents write `Counter(xs)`, batches of `n`
(`xs[i:i+n]` loops), sliding windows and `s.center(w)`. In AIL each costs a
loop and a mutable map or list (about 40-60 tokens), and the loops are where
agents' bugs are.

## Decision

- `xs.chunks(n)` and `xs.windows(n)`: `List[List[T]]`; trap for `n < 1`.
- `xs.counts()`: `Map[T, int]`, first-seen order; `T` hashable.
- `xss.flatten()`: `List[List[T]] -> List[T]`.
- `s.words()` (split on whitespace, no empties; Python's `s.split()`),
  `trim_start`, `trim_end`, `s.count(sub)` (traps on empty `sub`), `s.rfind(sub)`.
- `s.center(width, fill)`: extra padding on the right (consistent with `pad_right`).

Not added: `any`, `all`, `count` (`any` is a keyword, and `for` with an early
`return` is short), `partition`/`rsplit` (need tuples or a result struct;
`find` plus `slice` covers them), `zip` (needs tuples).

## Token measurement

Counting words: `mut c := {}` + loop with `c[w] = c.get(w) else 0 + 1` is
about 38 tokens; `ws.counts()` is 7. Batching 8 items at a time: a 6-line
loop (~45 tokens) against `xs.chunks(8)` (8).

## Rejected alternatives

- **Closure forms (`group_by(|x| key)`).** The checker allows closures only
  as `by:` of four methods (`expr.key.1`); widening that is a language
  change, out of this lane's scope. `counts` and `sorted(by:)` cover most uses.
- **Default argument for `center`'s fill.** AIL has no default arguments
  (type.str.9).
