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

Ship Option A now as the runtime primitive: it works for child streams and
channels on both tiers, and a block form would lower to it. The block form
(Option B) is decided by token count and follows in the next slice with its
own ADR (0429), because it touches the parser and the AIL front end.
`select` blocks index and `closed` semantics will match this rule.

## Consequences

- `task.select` has no effects (it reads the running task's state), like
  `task.cancelled`.
- Stale waiter entries on the other channels are removed on return.
