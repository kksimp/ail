# 1178. `List.get(i)`, `List.concat(ys)`, `str.find(sub, from)` and `int.pow(e)`

- **Status:** Accepted
- **Date:** 2026-11-25
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer's rule of no syntax change
- **Topic:** 4 — Type system
- **Spec rules:** [type.coll.34](../spec/04-types.md), [type.coll.35](../spec/04-types.md), [type.str.21](../spec/04-types.md), [type.num.12](../spec/04-types.md) (amended), [trait.op.1](../spec/10-traits.md)
- **Amends:** [ADR 1172](1172-first-draft-build-fixes.md) (`xs.get(i)` is a method, no longer rewritten), [ADR 1125](1125-methods-agents-guess.md)

## Context

Two list operations had no direct spelling, and agents wrote one anyway in every measured run.

- `xs.get(i)` for a read that may be past the end (`argv.get(1) else ""`): seen in E13, in the final run's failed
  first drafts, and in 3 builds of the $40 rerun. ADR 1172 rewrote it to `xs.skip(i).first()`, which builds a list to
  read one element and is not a form anyone would look for. `Map.get(k)` already gives a `V?`.
- A new list from two lists (`[x] + xs`, `xs + ys`): a failed draft in the rerun, with the bare error "`+` is not
  defined for List[int]" and no fix. The only std spelling was to copy one list and `extend` it, two statements.

- `s.find(x, start)` to search from a position: a failed draft in the rerun (expr.call.1). The std spelling was
  `s.slice(start).find(x)` plus `start`, which copies the rest of the text on every step of a scan.

- An integer power: `int` had no `pow` (the checker lane's probes). The ways around it were a loop or
  `n.to_float().pow(e).to_int()`, which is wrong above 2^53 without saying so.

Each rewrite and each failed draft is a build round an agent pays for (the measure of [ADR 1077](1077-decision-order-ai-understanding-first.md)
criterion 1 in the agent runs).

## Decision

- **`xs.get(i)` gives a `T?`** (type.coll.34): the element when `0 <= i < xs.len()`, `none` otherwise, a negative
  index included. It never traps. Native lowering is two compares and the element read.
- **`xs.concat(ys)` gives a new `List[T]`** (type.coll.35) and leaves both lists as they are. `bytes` already has
  `concat` with this meaning.
- **`s.find(sub, from)` gives an `int?`** (type.str.21): the index, counted from the start of `s`, of the first `sub`
  at character `from` or later; `none` when there is none or `from` is outside `0..=s.len()`. One runtime function,
  `str_find_from_opt`, with no copy.
- **`b.pow(e)` on `int` gives an `int`** (type.num.12), by squaring in the runtime (`int_pow`). It traps
  (expr.arith.1) when `e` is negative or the result does not fit, as `*` does.
- **`xs + ys` on two lists stays an error** (trait.op.1), now with the certain fix that writes `xs.concat(ys)` when
  both operands are simple enough to move (fix.edit.2), and help naming `concat` and `extend` otherwise.
- **`xs.at(i)`, `xs.nth(i)` and `xs.remove(v)` get help only.** `at` traps in some languages and gives nothing in
  others, and `Deque.at(i)` traps, so no single rewrite is right. `remove` on a list would be read as `remove_at` with
  a value in place of an index.

Token cost: none on code that does not use the names; `xs.get(i) else d` is shorter than a length test.

## Rejected alternatives

- **Keep the `skip(i).first()` rewrite.** It builds, but the agent's next program writes `get` again, and a reader
  must work out what the rewritten line means.
- **`+` joins lists.** `bytes` has it, and it is the habit. Rejected for now: `+` on `str` is refused by design, and a
  `+` that joins lists and not text would make that harder to learn; it also copies the left operand on every use,
  which `a + b + c` in a loop hides. A method keeps the copy visible. This can be revisited by the designer.
- **`get` with a negative index counting from the end.** Rejected: slices refuse negative indexes (type.coll.4), and
  a wrong computed index should read as absent, not as some other element.
- **`find` that traps on a `from` outside the text**, as `slice` does. Rejected: `find` already answers "not there"
  with `none`, and a scan loop ends on it either way.
- **`pow` that gives `int?` on overflow.** Rejected: `*` traps, and a power is repeated `*`; an optional would put an
  `else` on every use. A `checked_pow` can follow if a program needs it.
- **A `**` operator.** A syntax change; not in this lane.
- **`xs.remove(v)` as a method.** Deferred: it needs equality lowering for every element type; the help writes the
  two-step form.

## Consequences

- `tests/mistakes/list_get_guess` and `list_get_else` are removed (the code is now valid); `tests/cases/list_get_concat`
  covers both methods and `tests/cases/str_find_from` the search, `tests/cases/int_pow` the power; `tests/mistakes/list_plus`, `list_at_guess` and `list_remove_value` cover the diagnostics.
- `tools/native-matrix/matrix.py` gains `get` and `concat` snippets; `tests/native_matrix/` is refrozen with the batch.
