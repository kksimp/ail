# 0427. `task.select(chans)` waits on several channels and returns an index

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 15 — Concurrency
- **Spec rules:** [conc.select.1](../spec/11-concurrency.md#communication)
- **Builds on:** [ADRs 0186–0189](0186-deterministic-task-scheduler.md)

## Context

A program that reads a child's `out` and `err`, or a worker's results and a
stop signal, must wait for whichever has data. The brief asked to choose the
shape with tokbench: a `select` block, or a `chan.any([a, b])` function.

## Options considered

### Option A — `task.select(chans List[Channel[T]]) int?` (chosen for this slice)
Returns the index of a channel with a value waiting (lowest first), `none`
when all are closed and empty; the caller then does `chans[i].recv()`.
- **Pros:** a library function: no syntax, parser, formatter, interpreter
  tier or self-hosted front-end change; typed by one checker function.
- **Cons:** a loop that dispatches on the index is longer than a block.

### Option B — a `select` block with an arm per channel
```
select
    line := c.out.recv() => print("out {line}")
    line := c.err.recv() => eprint("err {line}")
    closed => open = false
```
- **Pros:** shorter (measured below), no index, no second `recv`.
- **Cons:** a new statement form: parser, checker, formatter, both
  interpreter tiers and their AIL ports.

### Option C — `select` returning a value and its index as a pair
- **Cons:** AIL has no tuples, and the value types of two channels may differ.

## Measurement (`tools/tokbench/run.sh --json`, a make-output loop)

| Form | cl100k | o200k | claude-legacy | mistral-tekken | mistral-spm-v1 | mean |
|---|---|---|---|---|---|---|
| A: `task.select` + index dispatch | 98 | 98 | 105 | 102 | 118 | 104.2 |
| B: `select` block | 79 | 79 | 85 | 83 | 94 | 84.0 |

The block is 19% fewer tokens, well over the 2% threshold of ADR 0076.

## Decision

Option A, final. The block is 19% shorter in the one construct it covers,
but ADR 0076 counts a difference "confined to rare constructs" as a tie:
3 of the 65 corpus and example programs use a channel at all, and a
multi-way wait is a fraction of those. On the tie, criterion 2 (AI
understanding) and the cost of change decide: a library function adds no
syntax for a model to learn, and no second parser (the Rust one and the
self-hosted one in `compiler/syntax/`) to keep in step. `task.select` ships
and a `select` block is rejected, not deferred. A later lane may revisit it
if multi-way waits become common in the agent corpus (re-measure then).

## Rationale

Tokens are a tie by ADR 0076, so AI understanding decides: one function with
one rule (`conc.select.1`) against a statement form with arms, a `closed`
arm and its own scoping.

## Consequences

- `task.select` has no effects (it reads the running task's state), like
  `task.cancelled`.
- Stale waiter entries on the other channels are removed on return.
- ADR 0429 is not used.

## Rejected alternatives

- A `select` block with an arm per channel (option B): 19% fewer tokens (84.0 against 104.2 for `task.select`). Rejected in the decision: the difference is in a rare construct (3 of 65 corpus programs use a channel), so it counts as a tie under ADR 0076. Criterion 2 then decides, since a block adds syntax and a second parser to keep in step. Re-measure if multi-way waits become common.
- `select` returning a value and its index as a pair (option C): rejected because AIL has no tuples, and two channels may carry different value types.
