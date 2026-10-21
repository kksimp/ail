# 0675. Round-4 syntax candidates: no new syntax, `par` over chunks already works

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** Syntax; token efficiency
- **Spec rules:** none changed (uses `expr.loop.4`, `expr.comp.*` and the `par` comprehension rules as they are)
- **Builds on:** [ADR 0255](0255-round-4-token-measurements.md), [program-measurements-agent-r4.md](../design/program-measurements-agent-r4.md)

## Context

Round 4 of the agent corpus (lane 203) and lane 192's std-parity twins named
syntax gaps: one handle per `spawn` (`task_sum`), no index-and-item loop,
no `zip`/`enumerate`/map `items` (they need pairs), and `fn main() !` plus
`?`/`else` plumbing. ADR 0076 decides by tokens first; under about 2% on
representative code, or confined to rare constructs, is a tie and the
language stays as it is. Measured with `tools/tokbench/run.sh --json`
(mean of five tokenizers) on the 40 twinned programs in
`tools/tokbench/corpus/programs/` (23,004 tokens in AIL).

## Decision

1. **`par` over a range of chunks needs no new syntax.** A `par` comprehension
   over the chunk starts already runs one call per chunk on the thread pool
   and keeps the results in order: `[sum_range(lo, lo + 250) par for lo in
   [0, 250, 500, 750]].sum()`. `examples/task_sum.ail` now uses it. The
   `spawn` handle form stays for tasks that differ (see `chan_pipe`).
2. **No index-and-item `for i, x in xs` and no `for k, v in m`.** Measured
   tie, rejected. `expr.loop.1` and `expr.loop.4` stay.
3. **Pairs, `zip`, `enumerate` and map `items` are not decided here.** They
   need a tuple (or pair) type, which is a core type-model decision for the
   designer: [OQ-124](../open-questions.md).
4. **`fn main() !` is not inferred** from a `?` in the body (rejected below).

## Measurements

Two-variable `for` (write `for i, x in xs`), applied by hand to every loop
in the corpus where it removes an index lookup or a `keys()` lookup (six
programs; the two loops in `hexdump` and `inventory` use forms AIL already
has, `for row in rows` and `for mut it in items`, and are not counted):

| Program | Before | After | Change |
|---|---|---|---|
| tail | 317.4 | 306.4 | −11.0 |
| sheet | 4,572.0 | 4,562.8 | −9.2 |
| json | 3,306.8 | 3,300.2 | −6.6 |
| csv2json | 187.8 | 180.4 | −7.4 |
| scan | 285.6 | 278.2 | −7.4 |
| interp | 5,419.0 | 5,414.6 | −4.4 |
| **Six programs** | **14,088.6** | **14,042.6** | **−46.0 (−0.33%)** |
| Whole corpus | 23,004.2 | 22,958.2 | −0.20% |

A tie (under 2%, and confined to loops that need both index and item; most
`keys()` loops are `keys().sorted()`, which the form does not cover). The
language card would also grow by an estimated 25 tokens (`expr.loop.4` and the
`for k in m` diagnostic change), against a saving of about 8 tokens a use.

`par` over chunks, `task_sum` (twin: Python 162.2):

| Form | Tokens | vs. Python |
|---|---|---|
| Four `spawn` handles and four `wait()?` | 157.2 | −3% |
| `[sum_range(lo, lo + 250) par for lo in [...]].sum()` | 111.2 | −31% |

Nothing is added to the card: its `par` comprehension line
(`ys := [f(x) par for x in xs]`) already covers it; the quick reference gets
one example line.

## Rejected alternatives

- **Two-variable `for` for lists and maps.** A 0.2% saving on the corpus
  (see above), a card cost, and a second shape for every loop that an agent
  must choose between. It also makes `for k, v in m` mean pairs while
  `for k in m` stays an error (`expr.loop.4`).
- **`for i, x in xs.enumerate()` and `zip`.** Need pairs; see OQ-124.
- **Inferring `!` on `main`.** Saves one token a program, and hides the
  effect from the signature (ADR 0037): worse for AI understanding.
- **A `par` range sugar such as `par sum(0..1000, 250)`.** More syntax for a
  shape the comprehension already writes in one line.
- **Doing nothing about `task_sum`.** The program was only the closest to
  Python because it was written with four named handles; the comprehension is
  not a new feature, so the example, not the language, was the gap.
