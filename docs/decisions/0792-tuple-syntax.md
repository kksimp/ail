# 0792. Tuple syntax: `(A, B)` types, `t.0` elements, `a, b := t`, `for k, v in m`

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076), under the designer's ADR 0791
- **Topic:** 04 — Types
- **Spec rules:** `type.tuple.1-5`, `type.map.5` ([04-types](../spec/04-types.md))

## Context

ADR 0791 adds tuples and leaves the spelling of the type, the element access,
destructuring and the `for` forms to the implementing lane.

## Decision

- **Type:** `(int, str)`, two or more elements. `(int)` and `()` are errors
  (`type.tuple.1`); a return type may start with `(`.
- **Literal:** `(a, b)`; `(a)` stays a parenthesised expression and `(a,)` is
  an error (`type.tuple.2`). Each element is checked against the expected
  element type, so `(1, none)` is an `(int, str?)` where one is expected.
- **Elements:** `t.0`, `t.1`, ..., read and assigned like fields. The lexer
  reads the digits after a single `.` as an index, so `t.0.1` is two accesses,
  not `t` and the float `0.1` (`type.tuple.3`).
- **Destructuring:** `a, b := e` with no parentheses, `_` to skip, and `mut`
  per name: `a, mut b := e` (`type.tuple.4`). A statement starting
  `name, name ... :=` is a destructuring; nothing else starts that way.
- **Loops:** `for a, b in xs` and the same clause in a comprehension. The
  parser binds a hidden loop variable, `(a, b)`, which no program can name,
  and takes it apart as the first statement of the body, so every later pass
  sees an ordinary loop plus a destructuring (`type.tuple.5`). `for mut` keeps
  one name.
- **Maps:** `for k, v in m` iterates a map's entries in insertion order.
  `for k in m`, one name, stays the `type.map.5` error, whose help now offers
  `for k, v in m` first.

## Token measurement

Tokbench mean of five tokenizers, on snippets of each form:

| form | chosen | alternative |
|---|---|---|
| sort and print by element (`p.0`, `p.1` / `p[0]`, `p[1]`) | 59.6 | 60.6 |
| `q, r := divmod(...)` and `for name, age in m` / with parentheses | 64.0 | 68.0 (+6%) |
| signatures `(int, int)`, `List[(str, int)]` / `Tuple[...]` | 29.8 | 31.8 (+7%) |

Twin t12 (`sort_ties`) rewritten with pairs falls from 91.4 to 57.6; t03 from
38.8 to 35.2 (tools/tokbench/README.md).

## Rejected alternatives

- **`t[0]`:** a tie on tokens, but an element's type depends on the index, so
  only a literal could be allowed, and `[i]` reads as list indexing an agent
  would try with a variable. `t.0` is a field read with a numeric name.
- **`(a, b) := t` and `for (k, v) in m`:** 4 tokens more on the snippet, and
  the parentheses add nothing the comma doesn't say.
- **`Tuple[int, str]`:** longer, and the literal is already `(1, "a")`.
- **`for k in m` yielding the pair:** one name would silently change meaning
  from keys (in Python) to entries; the error with two-name help keeps the
  reading explicit.
- **Destructuring as a pattern statement (`let`-style):** AIL has no `let`;
  the comma form fits `:=`.
