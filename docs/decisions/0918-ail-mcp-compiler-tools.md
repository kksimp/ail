# 0918. `ail mcp`: the compiler's own tools as an MCP server

- **Status:** Accepted
- **Date:** 2026-10-26
- **Decided by:** Designer (2026-10-25, backlog "agent task protocol over MCP"); the details by AI (delegated, ADR 0076), for integrator review
- **Builds on:** [ADR 0917](0917-ail-task-protocol.md) (`ail task`), [ADR 0892](0892-tool-functions-mcp-server.md) (MCP for user programs)
- **Topic:** 20 — AI semantic API

## Context

An agent reaches the compiler by running shell commands and parsing text. MCP gives it typed tools. ADR 0892 serves the `@tool` functions *of a user's program* (`ail serve --mcp`); nothing serves the compiler itself.

## Decision

- `ail mcp` runs an MCP server on stdio in the current directory: one JSON-RPC message per line in and out (`initialize`, `ping`, `tools/list`, `tools/call`; a message without an id gets no reply).
- Tools, each one `ail` command: `ail_check`, `ail_test` (with `since`: `--affected --since`), `ail_ask`, `ail_impact` (`ask --affected --json`) and `task_start`, `task_status`, `task_verify`, `task_finish`, `task_abort` (ADR 0917, `--json`).
- A call returns the command's output as text; `isError` is true when its exit status is not 0, so a red verify or a failing check is an error result carrying the diagnostics. A missing argument is JSON-RPC error -32602, an unknown method -32601, bad JSON -32700.
- Each tool's `inputSchema` lists string arguments and the required ones. The list is a table in `tools/ailmcp/ailmcp.ail`; a new command is one row and one dispatch line.
- Rule keys: `mcp.proto.*`, `mcp.tools.*`, `mcp.call.*`.

**Difference from ADR 0892:** `ail serve --mcp` serves the tools a *program* declares with `@tool`, built from that program's types. `ail mcp` serves the *compiler's* fixed tools and needs no program. The two share the wire format only.

Cost when unused (ADR 0868): none. It is a separate tool run only by `ail mcp`.

## Consequences

Tools shell out, so a call costs a process start; this keeps the server free of analysis and always equal to the CLI. No streaming and no resources or prompts in version 1.

## Rejected alternatives

- **Call the checker in process:** faster, but a second code path to keep equal to the CLI.
- **Reuse `ail serve --mcp` for both:** it takes a program path and derives tools from it; the compiler tools have no program.
- **One generic `ail_run` tool taking argv:** untyped, and it lets an agent run any command.
