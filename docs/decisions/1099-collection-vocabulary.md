# 1099. One collection vocabulary

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 4 — Types and collections
- **Spec rules:** type.coll.28-32, type.deque.6, expr.iter.4 ([04-types](../spec/04-types.md), [05-expressions](../spec/05-expressions.md))

## Context

The AI-first review (item 6) found that the names an agent reaches for on a
list (`count`, `fold`, `flat_map`, `skip`, `take_while`) did not exist, and that
`Set`, `Map`, `Deque` and `Seq` each had their own subset. A method that works
on one collection and is missing on its sibling costs an agent a failed build
and a guess (ADR 1077, criterion 1: AI understanding).

## Decision

One list of names, one argument order, one meaning:
`map filter any all find count fold flat_map take skip take_while skip_while zip
enumerate sum min max sorted group_by contains is_empty len`, on `List`, `Set`,
`Map` (a function takes `|k, v|`), `Deque` and `Seq`, wherever it has a meaning.
`fold(init, f)` takes the starting value first and `f(acc, x)`; `count(f)` takes
a predicate (so `Seq.count` changed from no argument to a predicate). A set and a
map follow insertion order (type.map.5). `Set.filter` gives a `Set`,
`Map.filter` a `Map`, `Map.map_values(f)` a `Map`; other results are lists
(a `Seq` stays a `Seq`, a `Deque` stays a `Deque` for the shape-keeping ones).

What a collection lacks is written, with the reason, in
`tools/collection-vocab-exceptions.txt`; `tools/collection-vocab-audit.py` fails
the docs gate on a gap with no line, and on a line whose gap closed.

Implementation: `compiler/check/vocab.ail` types the calls; `compiler/mir/lower_vocab.ail`
rewrites each into the `for` loops and comprehensions the language already has
(one dispatch line in `lower.ail`), reading a set as a list of its elements and
a map as `for k, v in m`. `Seq` and `Deque` are prelude code in AIL.

## Rejected alternatives

- **Free functions (`fold(xs, 0, f)`).** Cost: a second spelling beside the
  method form every other method uses; worse for completion and for reading left to right.
- **`fold(f, init)` or `reduce`.** Cost: a second argument order than the
  `fold(init, |acc, x| e)` that Rust, Kotlin and Swift agents already write; `reduce` without an `init` has an empty-collection case.
- **`Set.map` returning a `Set`.** Cost: the result silently shrinks when two
  elements map to one value; a list says what happened.
- **Runtime calls for each method.** Cost: one native routine per collection and
  element type, against loops that already get bounds-check removal and inlining.
- **Keeping `Seq.count()` with no argument.** Cost: `count` would take a
  predicate everywhere but one collection.

## Measurements

None: the change adds names; no existing program's code changes except
`Seq.count()` callers (one test).
