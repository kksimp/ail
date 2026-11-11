# 0918. `ail mcp`: the compiler's own tools as an MCP server

- **Status:** Accepted
- **Date:** 2026-10-26
- **Decided by:** Designer (2026-10-25, backlog "agent task protocol over MCP"); the details by AI (delegated, ADR 0076), for integrator review
- **Builds on:** [ADR 0917](0917-ail-task-protocol.md) (`ail task`), [ADR 0892](0892-tool-functions-mcp-server.md) (MCP for user programs)
- **Topic:** 20 — AI semantic API
- **Amended by:** [ADR 1056](1056-warm-ail-mcp.md) (warm calls)

## Context

An agent reaches the compiler by running shell commands and parsing text. MCP gives it typed tools. ADR 0892 serves the `@tool` functions *of a user's program* (`ail serve --mcp`); nothing serves the compiler itself.

## Decision

- `ail mcp` runs an MCP server on stdio in the current directory: one JSON-RPC message per line in and out (`initialize`, `ping`, `tools/list`, `tools/call`; a message without an id gets no reply).
- Tools, each one `ail` command: `ail_check`, `ail_test` (with `since`: `--affected --since`), `ail_ask`, `ail_impact` (`ask --affected --json`), `ail_verify` (`verify --diff --json`) and `task_start`, `task_status`, `task_verify`, `task_finish`, `task_abort` (ADR 0917, `--json`).
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

## Amendment (2026-11-10): every agent tool is one MCP call

**Decided by:** AI (delegated, ADR 0076). Slice E2 of `lanes/ai-first-edge-2026-11-10.md`; in 0.1.0 by the designer's decision of 2026-11-10.

Six commands an agent drove only through the shell now have tools: `ail_context` (ADR 1020), `ail_edit` (with `test`, `impact` and `dry_run`, so one call edits, checks and runs the affected tests), `ail_fix`, `ail_explain`, `ail_api` (`form` card, index, outline, symbols, text or pretty, plus `module`) and `ail_review` (ADR 1024). The server lists 16 tools. Each new tool passes `budget` through as `--budget N` and otherwise uses the command's own default.

- `inputSchema` types now include `integer` (`budget`) and `boolean` (`json`, `test`, `impact`, `dry_run`); a boolean is also accepted as the text `"true"`.
- **`mcp.call.3`:** when a command ran with `--json` and printed one JSON object, the result also carries it as `structuredContent`. Always-JSON tools (`ail_verify`, `ail_impact`, `task_*`) get it on every call; the others when the caller sets `json`. The text content is unchanged.
- **`mcp.call.4`:** a positional value (`program`, `symbol`, `rule`, `range`, `path`, `module`, `form`) that starts with `-` is refused, so it can never be read as an option.
- `ail_edit` sends `edits` on standard input; the compact hunk form and a JSON edit list both work.
- Existing rule IDs keep their numbers; `mcp.tools.1` lists the new names.

Rejected alternatives:

- **One generic `ail_run` tool taking a command line:** no schema, so the agent guesses flags (the rejection above stands).
- **A `json` default of true on every tool:** JSON costs more tokens than the text forms of `context`, `api` and `explain`; the agent asks for it when it wants fields.
- **Separate `ail_edit_test` and `ail_edit_impact` tools:** one call with two flags removes a turn from every edit-then-test pair and adds one row to the table instead of three.

## Amendment 2026-11-11: round 2, the tools that landed after the first round

Decided by: AI (delegated, ADR 0076), gap G5 of the final-gaps review. Token usage ranks first: an agent that works over MCP had no way to look up a std function without loading the full card.

Seven tools join, for 23 in all: `ail_card` (`topic`, optional), `ail_find` (`words` or `sig`; `ail ask std`), `ail_try` (`program`, `sets`, a list of patch sets sent inline; one verdict per set, never `--apply`), `ail_prove`, `ail_infer` (`program`, `function`; never `--apply`), `ail_merge_check` (`ail merge --check`, three files or two revisions) and `ail_verify_check` (`cert`). `ail_card`, `ail_find` and `ail_prove` are read-only and join the warm memo; `ail_try`, `ail_infer` and the two checks that read git state (`ail_merge_check`, `ail_verify_check`) do not. `ail try` gets `--sets-json` (try.in.2) so several sets travel in one call without temp files.

Every description is now one short sentence, and `required` is left out when empty, so the whole `tools/list` answer is 5.0 KB (about 1,250 tokens at 4 bytes a token; the gate fails above 6,000 bytes, rule mcp.tools.3). Rejected: one tool per `ail try` set (the agent compares sets in one call); a `--apply` argument on `ail_try` or `ail_infer` (an MCP call must not write on a guess); memoising the git-reading checks (their inputs are git objects, not the program's files).
