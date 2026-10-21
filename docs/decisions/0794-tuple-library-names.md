# 0794. `xs.enumerate()` and `zip(a, b)`; no `m.items()` or `m.pairs()`

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076), under the designer's ADR 0791
- **Topic:** 04 — Types (library)
- **Spec rules:** `type.tuple.9`, `type.tuple.5` ([04-types](../spec/04-types.md))

## Context

ADR 0791 names `xs.enumerate()`, `zip(a, b)` and, only if measured to pay,
`m.pairs()`.

## Decision

- `xs.enumerate()` is a list method returning `List[(int, T)]`.
- `zip(a, b)` is a built-in function of two lists returning `List[(A, B)]`,
  as long as the shorter. It is a free function because neither list is the
  receiver; a program's own `fn zip` replaces it, so no existing program
  changes meaning. Three lists zip as `zip(zip(a, b), c)`.
- **No `m.items()` or `m.pairs()`:** a map iterates as its entries
  (`for k, v in m`), and `ail fix` deletes `.items()` (`expr.name.1` help and
  fix). `enumerate(xs)` gets the fix `xs.enumerate()`.

## Token measurement

Tokbench mean of five tokenizers:

| snippet | tokens |
|---|---|
| `for i, x in xs.enumerate()` and `zip(xs, ys)` (chosen) | 42.6 |
| free `enumerate(xs)` | 42.2 |
| method `xs.zip(ys)` | 42.4 |

All three are ties (under 2%). The method keeps `enumerate` with the other
list methods and needs no new global name; `zip` stays free as ADR 0791 says.

`m.pairs()` was measured on the one twin that sorts map entries, t24: 79.0
tokens now (`keys().sorted()` and `m[k]`), 74.6 with `m.pairs().sorted()`,
82.6 with `[(k, v) for k, v in m].sorted()`. The saving is 4.4 tokens on one
of 25 twins (0.4% of the set), a tie under ADR 0076, against a card line, so
`m.pairs()` is not added.

## Rejected alternatives

- **`m.items()`:** Python's name for the same pairs; iterating the map
  already gives them.
- **`m.pairs()`:** measured above; revisit if more programs need a list of
  entries.
- **`zip` as a method:** a tie on tokens, and ADR 0791 names the function.
