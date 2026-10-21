# 0758. `s.split(sep, limit)` makes at most `limit` pieces

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076); the integrator chose it over `str.partition`
- **Topic:** 04 — Types (strings)
- **Spec rules:** type.str.4 ([04-types](../spec/04-types.md))
- **Amends:** [ADR 0124](0124-string-split-and-to-int.md)

## Context

Lane 192's twin t19 reads `key=value` lines and must split at the first `=`
(a value may hold one). Python writes `k, v = line.split("=", 1)`; AIL needed
`l.find("=")`, two `slice` calls and an `else continue`. ADR 0750 skipped
`str.partition` (0.9% across the twins, and its three-part result needs
tuples, OQ-124).

## Decision

`s.split(sep, limit)` returns at most `limit` pieces and the last keeps the rest
of `s`. `limit` counts pieces, not cuts: `"a=b=c".split("=", 2)` is
`["a", "b=c"]`. `limit` must be at least 1, otherwise the call traps
(`type.str.4`); 1 returns `[s]`. `sep` is a `str` or a `char` as for `split`
(`type.str.6`). No new method name: one optional argument on a method agents
already know, and no tuples.

## Token measurement

Twin t19 (`key=value` lines), tokbench mean of five tokenizers: the `find`
and `slice` form 96.2, with `split("=", 2)` 89.6 (-6.6, -6.9%), Python 63.6.
Language card: a clause on the existing `split` line, +15 tokens on the trimmed card (6,771 to 6,786; budget 8,000).

## Rejected alternatives

- **`str.partition`** (ADR 0750): returns three parts, which needs tuples.
- **`splitn` as a second method:** one more name for the same job.
- **Python's `maxsplit` (cuts, not pieces):** `split("=", 1)` would give two
  pieces, which reads as "one piece" to an agent that has not met Python.
  Rust's `splitn` and this ADR count pieces.
