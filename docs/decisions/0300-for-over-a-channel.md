# 0300. Iterate a channel with `for v in ch`; no `select`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 11 Concurrency
- **Spec rules:** `conc.share.7`, `expr.loop.4`, `conc.par.2`

## Context

Lane 22 gave `Channel[T]` with `recv() T?` (ADR 0188), so every consumer
was written `while true` / `v := ch.recv() else break`. That is the most
common channel pattern and it costs a line and several tokens each time.
Go writes `for v := range ch`; Python writes `for v in iter(q.get, None)`.
The brief also asked whether a `select` over several channels pays off on a
fan-out, fan-in program (`examples/workers.ail`).

Measured with `tools/tokbench/run.sh` (mean of five tokenizers):

| Program | Before | After | Python | Go |
|---|---|---|---|---|
| drain loop (4 lines) | 25.0 | 17.6 (−30%) | | |
| `examples/pipeline.ail` | 219.0 | 204.2 (−6.8%) | 240.8 | 217.2 |
| `workers.ail` (with ADR 0301) | 394.6 | 366.4 (−7.1%) | 421.2 | 468.0 |
| `workers.ail` with a `select` | | 400.8 (+9.4%) | | |

## Options considered

### Option A — `for v in ch` (chosen)
- **Pros:** 30% fewer tokens on the loop; the form every agent already
  writes for lists and Go channels; also works in comprehensions
  (`[d for d in results]`); no new keyword.
- **Cons:** a `for` loop can now block, which `conc.par.2` must count as
  synchronisation.

### Option B — keep `while true` / `recv() else break`
- **Pros:** nothing new.
- **Cons:** three lines for the commonest pattern; an agent that writes
  `for v in ch` gets an error today.

### Option C — `ch.values()` returning a list
- **Pros:** no change to `for`.
- **Cons:** a list needs the channel closed first, so it can't stream; a
  pipeline stage would deadlock.

### `select`
A `select` with one arm per channel was written for `workers.ail`, where
workers report "done" on a second channel. It is 9.4% heavier than closing
the results channel from the last worker (ADR 0301), needs a new
statement with arm syntax, and adds nondeterministic choice that the
deterministic scheduler (ADR 0186) and `--deterministic` would have to
order. Not adopted.

## Decision

`for v in ch` (and a comprehension clause over a channel) receives until the
channel is closed and empty, waiting like `recv`. There is no `select`.

## Rationale

Tokens decide (criterion 1): −30% on the loop, −7% on whole programs. AI
understanding (criterion 2) agrees: one obvious form, matching lists and Go.

## Consequences

- `conc.par.2` treats `for v in ch` as a handle call: such a loop never runs
  on the pool, and `par for v in ch` is an error.
- A channel nobody closes makes the loop a deadlock (`conc.task.7`), caught
  with a trap that names the loop.
- `select` can be reconsidered if a timeout or cancellation program needs it.

## Rejected alternatives

- `while true` / `recv() else break` only: 30% more tokens on the commonest pattern.
- `ch.values()`: can't stream, deadlocks pipelines.
- `select`: 9.4% heavier on `workers.ail`, new syntax, and nondeterministic choice.
