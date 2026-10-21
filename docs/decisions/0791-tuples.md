# 0791. Tuples: anonymous pairs and triples as values

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** the designer (option "Full tuples" on the OQ-124 card)
- **Topic:** 04 — Types
- **Spec rules:** `type.tuple.*` ([04-types](../spec/04-types.md)), written by the implementing lane
- **Resolves:** OQ-124

## Context

AIL had no pair type. Agent code reaches for `zip(a, b)`, `enumerate(xs)`
and `m.items()`, and returning two results from one call needs a struct.
Day-2 measurements on the 25 std twins: tuples save 53.8 tokens
(AIL +14.6% over Python to +9.1%), against 57.2 for the three no-core
features together (`json.load[T]`, derived `args.parse`, `split` limit), so
the two are close to additive, not alternatives. Pairs change the type model
(value semantics, patterns, `==`, map keys), so the designer decided.

## Decision

A tuple is an anonymous value of two or more elements with their own types,
written `(a, b)` and typed `(int, str)`. It is a value like a struct: copied
on assignment, `mut` where its holder is `mut`, never shared.

- **Elements:** `t.0`, `t.1`, ... read and (on a `mut` holder) assign.
- **Destructuring:** `a, b := t` declares both; `for i, x in xs.enumerate()`,
  `for k, v in m` (iterating a map yields `(key, value)` pairs, so there is
  no `items()`), and `(0, y)` or `(x, _)` as a `match` pattern.
- **Library:** `xs.enumerate() List[(int, T)]`, `zip(a, b) List[(A, B)]`
  (stops at the shorter), `m.pairs()` only if a program needs a list of pairs
  (measure first). `sorted`, `==`, `<`, `hash` and `to_str` are elementwise
  when every element has them, so a tuple is a map key and prints as
  `(1, "a")`. JSON: a tuple is an array of its elements both ways.
- **Not included:** one-element tuples, the empty tuple (there is `unit`),
  named tuple fields (that is a struct), tuple splat into arguments.

The lane that implements this writes the `type.tuple.*` rules with runnable
examples (ADR 0498), ports the checker to `compiler/check` in the same PR
(ADR 0740), covers the bytecode tier, `ail fmt`, `ail fix` where a Python
habit maps (`m.items()` → `m`, `enumerate(xs)` → `xs.enumerate()`), and
measures the card growth and the twins before and after with tokbench. Native
lowering of tuples goes through the lower.ail queue as a second PR.

## Rejected alternatives

- **No pairs (OQ-124 a):** keeps the type model small but leaves the
  measured gap on every pair-shaped program.
- **Loop syntax only (OQ-124 c):** pays syntax for the smallest slice and
  still cannot express `zip` or two results from one call.
- **Named tuple fields:** duplicates structs.
