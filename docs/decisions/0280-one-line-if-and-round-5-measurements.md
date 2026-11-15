# 0280. One-line `if c then a else b`, `{}` typed by `m[k] op= n`, and the round-5 measurements

- **Status:** Accepted; amended by [ADR 1120](1120-one-line-if-elif.md) (the one-line form takes `elif` arms)
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Core syntax; collections; token efficiency
- **Spec rules:** expr.if.8, type.coll.13

## Context

Lane 51 took the agent corpus from +6.9% over Python to parity. Besides ADRs
0277–0279, two smaller costs remained: choosing between two values takes a
four-line `if`/`else` block where Python writes `a if c else b`, and a counter
map declared from `{}` had to be written `Map[str, int]()` because `m[k] += 1`
(ADR 0251) didn't fix its type. The brief also asked to measure return-type
inference, `print` with several arguments and string `+`. All numbers are in
[program-measurements-r5.md](../design/program-measurements-r5.md).

## Options considered

### One-line conditional
- **`if c then a else b` (chosen):** −11.4 tokens on the agent corpus
  (−0.66%), −42.4 on the examples (−0.23%), reads left to right in the order
  the multi-line `if` has, and `then` is contextual, so it stays a name.
- **Python's `a if c else b`:** reads the value before the condition, and
  clashes with comprehension `if` filters.
- **Keep only the block form:** four lines for a two-way choice.

### `{}` with `m[k] op= n`
- **Fix the type from the first `op=` with a number (chosen):** −24.0 tokens on
  the five programs (−1.0%); it is the rule `m[k] = v` already follows.
- **Keep `Map[K, V]()`:** spells out what the first use already says.

## Decision

- `if c then a else b` is an expression on one line (`expr.if.8`). `a` and `b`
  are plain expressions; the `else` belongs to the `if`. It has no `elif`.
- A local declared from `{}` becomes a `Map` at its first `x[k] op= v` with a
  numeric `v`, as at `x[k] = v` (`type.coll.13`).
- Rejected after measuring: return-type inference for one-expression private
  functions (−0.53% of the agent corpus, a tie that loses signatures as the
  thing an agent reads first, ADR 0184), `print` with several arguments (no
  saving: interpolation is as short), and string `+` (+0.37%: interpolation is
  shorter, so it stays rejected).

## Rationale

Both adopted changes are ties on tokens (under 2%), so criterion 2 decided:
`if c then a else b` keeps the condition first, like every other `if`, and
puts a two-way value on the line that uses it; `{}` typed by its first use is
the existing rule applied to one more first use. Each rejected candidate either
costs tokens or trades away explicit signatures for under 1%.

## Consequences

- The examples use the one-line form where a value is chosen between two
  short expressions; `sheet` renamed its variable `then` to `yes` for
  readability, though `then` remains a valid name.
- The three-program agent corpus is 0.25% below Python; the five-program
  corpus is 0.9% above.

## Rejected alternatives

- `a if c else b`, return-type inference, multi-argument `print`, string `+`:
  see above.
