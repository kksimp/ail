# 0351. Cancel trees, and `send` in a cancelled task drops its value

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 11 Concurrency
- **Spec rules:** `conc.cancel.3`, `conc.cancel.4`
- **Resolves:** OQ-097 parts (2) and (3); extends ADR 0321

## Context

ADR 0321 made cancellation a cooperative flag. Two gaps remained (OQ-097):
cancelling a task didn't reach tasks it had already started (only ones it
started later), and a cancelled task blocked in `send` on a full channel
kept waiting, so a consumer that stopped early and cancelled its producer
deadlocked unless it drained the channel first.

## Decision

1. **Cancel trees (OQ-097 (2), option a).** Each task records the tasks it
   starts on the scheduler. `cancel` marks the task and walks its unfinished
   descendants, marking and waking each, so a whole subtree stops the way a
   group's tasks do. This is what scope-bound tasks (ADR 0096) imply: a
   child can't outlive its parent's scope, so stopping the parent means
   stopping the child.
2. **Cancelled `send` (OQ-097 (3), option c).** In a cancelled task, a send
   on a full channel drops the value and returns; a cancel wakes a send
   that is already blocked. A send with room still queues, and a send on a
   closed channel still traps.

Decided by criterion 1 (tokens), with criterion 2 agreeing. The brief's
measurement: the `tail` and `runner` corpus programs, rewritten so the
consumer stops early (the third `ERROR` line; the first failed step) and
cancels the producers, written under (b) and under (c). Under (b) the
consumer must drain the channel before waiting, or the blocked producer
deadlocks the scope:

```ail
                feed.cancel()
                break
    for _ in ch          # (b) only: release the producer blocked in `send`
        continue
    feed.wait()?
```

| Program | (b) keep waiting | (c) drop and return | Saved |
|---|---|---|---|
| `tail`, stop at third error | 137.8 tokens, 17 lines | 129.2 tokens, 15 lines | 6.2% |
| `runner`, stop at first failure | 196.8 tokens, 26 lines | 188.2 tokens, 24 lines | 4.4% |

(Mean of the five tokbench tokenizers.) Both are above the 2% tie. The
drain loop is also the kind of non-local obligation an agent forgets: the
missing two lines show up only as a deadlock, under the schedules where the
producer filled the channel first.

## Options considered

### Cancel trees: (a) the whole tree (chosen)
- **Pros:** matches structured concurrency; one `cancel` at the top stops
  an agent's whole fan-out; no new syntax.
- **Cons:** a child the parent meant to finish regardless also stops;
  it can still finish its work, since cancellation is cooperative.

### Cancel trees: (b) per task (rejected)
Each task cancels its own children by hand. Every parent then needs the
handles and a cancel check, which is the code (a) saves, and a task that
forgets leaves a subtree running past the cancel.

### Cancelled send: (a) `send` returns `! Cancelled` (rejected)
Changes the type of every existing `send` call, so each needs `?` or an
`else`: the most tokens of the three.

### Cancelled send: (b) keep waiting (rejected)
The ADR 0321 behaviour. Costs the drain loop above and deadlocks when it is
missing.

### Cancelled send: (c) drop the value and return (chosen)
- **Pros:** no new code in producer or consumer; a cancelled producer runs
  to its end quickly (each further send on the full channel returns at
  once) and closes the channel as usual.
- **Cons:** the dropped values are lost silently. A cancelled task's output
  is unwanted by definition, and a producer that must know can test
  `task.cancelled()`.

## Consequences

- The `Cancelled` and `recv` rules of ADR 0321 are unchanged; `send` now
  joins `recv` in returning early.
- Pooled tasks (`conc.par.10`) are pure and aren't in the tree.
- Random-schedule tests (`bootstrap/tests/schedule/cancel.ail`, 24 seeds)
  cover a cancel that reaches an already-running child and a cancelled
  producer blocked on a full channel; both deadlocked before this change.
