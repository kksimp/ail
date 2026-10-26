# 0917. `ail task`: the agent task protocol

- **Status:** Accepted
- **Date:** 2026-10-26
- **Decided by:** Designer (2026-10-25, backlog item "agent task protocol"); the details by AI (delegated, ADR 0076), for integrator review
- **Builds on:** [ADR 0882](0882-edit-impact.md) (impact), [ADR 0881](0881-ail-ask.md), [ADR 0892](0892-tool-functions-mcp-server.md) (MCP)
- **Topic:** 20 — AI semantic API

## Context

An agent doing one change calls `ail ask --affected`, `ail check`, `ail test --affected` and a diff, remembers the base revision itself, and decides alone when it is done. Each of those steps costs tokens in prompts and wrong turns. A task needs a recorded base, one verdict, and a stated next step.

## Decision

- One open task per directory, in `.ail-task` (gitignored): program, goal, base revision, result of the last verify.
- `ail task start PROGRAM GOAL... [--rev REV]` records the base (default `HEAD`; it must compare, as `api.impact.3`).
- `ail task status` prints the impact since the base (changed, affected, tests, effects) and whether the tree was verified.
- `ail task verify` runs `ail check`, then `ail test --affected --since BASE`, and records green or red. A failure shows the last 20 lines.
- `ail task finish` refuses unless the last verify was green; it prints the final impact and closes the task. `ail task abort` closes it.
- Every answer ends with `next`, the one command to run after it; `--json` gives the same fields.
- It adds no analysis; it only sequences existing commands. Rule keys: `task.state.*`, `task.start.*`, `task.verify.*`, `task.finish.*`.
- MCP: each subcommand is one tool (`task_start`, `task_status`, `task_verify`, `task_finish`) with the `--json` fields as the result, once the compiler serves MCP (ADR 0892 serves user programs only; the compiler's own tool list is a follow-up).
- `verify` runs the tests with `--bundle-failure` (ADR 0910) and lists the bundle files of a red run (`task.verify.4`). When `ail verify --diff` (ADR 0909) lands, `verify` calls it too.

Cost when unused (ADR 0868): none. It is a separate tool run only by `ail task`.

## Consequences

An agent loop is `start`, edit, `verify` until green, `finish`. A task is not tied to a session, so another agent can resume it.

## Rejected alternatives

- **A server holding task state:** needs a daemon; a file survives restarts and is readable.
- **A task id per task:** a directory has one working tree, so one open task is enough.
- **`finish` re-running the tests:** doubles the cost; the recorded green is checked against the base only, and the next `verify` is cheap with `--affected`.
