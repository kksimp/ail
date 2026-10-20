# 0302. Parallel map: `[f(x) par for x in xs]`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 11 Concurrency
- **Spec rules:** `conc.par.12`, `expr.comp.2`

## Context

Lane 49 runs `par for` loops on a thread pool (ADRs 0272–0275), but a
parallel map had to be spelled as a preallocated list plus an indexed loop.
The designer wants multi-core use to be the easy path. The brief asked for
`xs.par_map(f)` or `[f(x) par for x in xs]`.

Measured with `tools/tokbench/run.sh` (mean of five tokenizers) on a
four-line `main` that maps `steps` over a list:

| Form | Tokens |
|---|---|
| `mut lens := [0 for _ in xs]` + `par for i in 0..xs.len()` loop | 61.6 |
| `xs.par_map(\|x\| steps(x))` | 42.2 |
| `[steps(x) par for x in xs]` (chosen) | 42.0 |
| Python `ProcessPoolExecutor().map` | 59.4 |

## Options considered

### Option A — `[f(x) par for x in xs]` (chosen)
- **Pros:** −32% against the loop; reuses `par for` and the comprehension
  (filters with `if` work); no closures beyond key functions; one keyword
  says where the cores are used.
- **Cons:** only one clause, only a list.

### Option B — `xs.par_map(|x| f(x))`
- **Pros:** reads like other languages' map.
- **Cons:** a tie on tokens; widens `|x|` closures beyond key functions
  (ADR 0250); no filter; another method to learn beside comprehensions.

## Decision

A list comprehension with one clause written `par for` computes its
elements on lane 49's pool when that clause would run there, in order and
with the sequential result; the element and filter must have no ordering
effect, no handle use and no outside writes, and one that provably breaks
that is a compile-time error.

## Rationale

Tokens tie (0.5%), so AI understanding (criterion 2) decides: the form
reuses two constructs agents already know, and its checks are the
`conc.par.2` checks they already get from `par for`.

## Consequences

- Each element is computed by a worker with its own interpreter; captured
  values are copied, as for `par for` (ADR 0274).
- A trap, a handle or a `ref` makes the comprehension recompute in order,
  so the trap is the sequential one.
- Several clauses or set and map comprehensions can follow if programs need them.

## Rejected alternatives

- `xs.par_map(f)`: same tokens, a wider closure feature, no filter.
- Only the indexed `par for` loop: 47% more tokens for the commonest parallel pattern.
