# 0267. A trap carries a snapshot of the stack and its values

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Error handling (traps); tooling for AI agents
- **Spec rules:** `trace.snap.1–4`

## Context

A trap used to print one line: position, rule and message. To learn why, an
agent had to add prints and run again. The values in scope at the trap are
what it needs: the arguments of each frame, the locals, and for a contract
the values its condition read.

## Options considered

### Option A — always capture, short by default, `--verbose` and `--json` for all (chosen)
The interpreter keeps each frame's locals as a trap unwinds (the innermost
block records them before blocks drop them). The human report keeps the old
first line and adds the source line with a caret, the condition's values and
one line per frame, at most ten lines; `--verbose` shows every frame and each
local on its own line; `--json` gives one object with `"severity":"trap"`.
- **Pros:** no flag needed for the common case; the first line is unchanged,
  so tools that read `trap[RULE]` still work; cost only when a trap happens.
- **Cons:** the report grows from one line to a few; five existing `.err`
  cases were updated.

### Option B — snapshot only with a flag
- **Pros:** the old one-line report by default.
- **Cons:** the agent needs a second run to see values, which is the cost
  this lane removes.

### Option C — a core dump or full heap snapshot
- **Cons:** huge, needs a tool to read. Rejected.

## Decision

Option A. At most 64 frames are captured (the rest counted); values use the
form and budgets of ADR 0266. Only the main interpreter of `ail run` keeps
snapshots; `ail test` and task interpreters don't. A trap inside a task is
reported to its waiter (conc.task.6) as before.

## Rationale

AI understanding: one run gives the failing line, the values that made it
fail and how execution got there. Tokens: ten lines by default keeps the
common report small; `--verbose` is there when the agent needs more. Speed:
capture runs only on the trap path, so normal execution is unchanged.

## Consequences

- `ail run --json` now reports a trap as a JSON object, like diagnostics.
- The failing expression's *span* is its start position and source line; an
  exact end column needs end positions in the parser (follow-up).

## Rejected alternatives

- **Flag-only snapshots:** costs a second run in the common case.
- **Core dump:** unreadable without tooling, large.
- **Showing every frame of a deep recursion:** 10,000 near-identical frames;
  64 plus a count says the same.
