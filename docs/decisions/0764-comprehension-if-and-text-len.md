# 0764. `if` elements in comprehensions and `len` of text compile in the tier

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 13 — Standard library (the interpreter's speed)
- **Spec rules:** expr.comp.1, expr.if.1 (behaviour unchanged)
- **Builds on:** [ADR 0763](0763-comprehensions-compile-to-loops.md)

## Context

ADR 0763 left a comprehension to the walker whenever its element, filter or
range held an `if` or `match` expression, because the test for "can leave
early" (`has_try`) treats both as unknown. Separately, `s.len()` on a text,
map, set or bytes local ran the walker: the fast path for `len` and `push`
handled lists only, and every other receiver fell through to a hand-off.

## Decision

- `comp_part_plain` looks inside `if` and `match` expressions whose branches
  only compute values (expressions and `name := value` bindings), so
  `[if x > 3 then x * 2 else x for x in xs]` compiles to loops. A branch with a
  statement that can leave (`?`, `break`, `continue`, `return`, a leaving
  `else`) or any other statement keeps the comprehension in the walker.
- `ListFast` counts text (characters, as the walker does), map and set
  entries and bytes for `len` itself, then falls back to the walker for any
  other receiver as before.

- A trap inside a comprehension lists the comprehension's variables among the
  locals in scope (trace.snap.2), as a trap inside a `for` already lists its
  variable. The walker's `comp` popped them before the snapshot was taken, and
  the compiled loops (ADR 0763) leave them in place, so the tiers printed
  different locals. The walker now takes the snapshot before it pops.

## Measured (callgrind, Linux x86-64)

A micro-benchmark of 200,000 `w.len()` calls on text plus 20 comprehensions with
an `if` element over 10,000 items: 400.0 M to 226.8 M instructions (-43%). The
`tools/perfbench` programs don't use either form in a hot loop and are
unchanged within 1.3% (binarytrees +1.0%, fasta +1.3% from code layout; no
instruction on their path changed).

`./ail selftest` passes. `comp_loops.ail` gains `if` elements, text, map, set
and bytes lengths, and has no hand-offs; it agrees with `--no-bytecode`. The
bytecode fuzz generator (`tests/fuzz/bcprogs.rs`) gains comprehensions with
filters, `if` elements, two clauses, sets and maps. ADR 0763's fuzz claim was
wrong: the generator it extended (`tools/fuzz/gen/gen.ail`) is not the one the
bytecode differential uses. The first run with the right generator found the
locals disagreement above in 3 of 3,000 programs; after the fix, 8,000 programs
and a longer run show none.

## Rejected alternatives

- **Teach `has_try` about `if` and `match`.** It also guards the `else`
  compile path; widening it changes code generation outside comprehensions
  for no measured gain.
- **A `Str` branch in the method guard instead of `ListFast`.** The guard is
  the slow path (a user-method cache probe); `len` on text is common enough to
  take the one-instruction route.
