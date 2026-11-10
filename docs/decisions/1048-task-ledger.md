# 1048. The task ledger: `ail task` measures itself

- **Status:** Accepted
- **Date:** 2026-11-10
- **Decided by:** AI (delegated, ADR 0076); scope into 0.1.0 by the designer (2026-11-10)
- **Builds on:** [ADR 0917](0917-ail-task-protocol.md), [ADR 0311](0311-ail-api-card-budget-and-trimming.md) (token estimate)
- **Topic:** 20 — AI semantic API

## Context

How an agent reached a change (how many verify rounds, how many red, how many tokens the toolchain cost it, which rules it tripped on) is known only by scraping a transcript. A benchmark harness and a reviewer both want it from the repository.

## Decision

- `ail task verify` appends one line to `.ail/task-log.jsonl` (`task.log.1`): `stage`, `result`, `tokens` (the estimate of ADR 0311 of what `verify` printed), `fixed` (fixes `--fix` applied), `rules` (rule IDs in the output, with counts) and `ms` last. Every field but `ms` is a function of the change, so the log is byte-stable across reruns.
- The task file records how many lines the ledger had at `start`, so a task's own lines are the ones after it (`task.log.2`). `ail task finish --json` adds `summary`: `rounds`, `red`, `fixed`, `tokens`, `rules` (`task.log.2`).
- `ail task log [--json]` prints the open task's lines (all of them when none is open) and that summary (`task.log.3`).
- Nothing leaves the machine. `.ail/` is for tools to read; it is gitignored.

## Consequences

The agent benchmark can read rounds and toolchain tokens from the ledger (E3), and a mined corpus of first-attempt rules is available to the mistakes corpus (E13).

## Rejected alternatives

- **Upload or opt-out telemetry:** privacy, and there is no service to pay for.
- **A ledger line per command:** `verify` is the unit an agent loops on; other commands are not task steps.
- **Exact tokenizer counts:** needs a tokenizer in the toolchain; the card estimate is already the project's measure.
