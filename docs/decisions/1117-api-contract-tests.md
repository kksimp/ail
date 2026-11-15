# 1117. Contract tests for the semantic API, and the edits they needed

- **Status:** Accepted
- **Date:** 2026-11-15
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); asked for by the designer for 0.1.1 after the external review of 2026-11-15
- **Topic:** 22 — Tools
- **Spec rules:** [fix.edit.7, edit.op.11, api.refs.4, mcp.proto.3](../spec/22-tools.md)
- **Builds on:** [ADR 0145](0145-diagnostics-schema-v1.md), [ADR 0397](0397-ail-edit-structured-edits-by-id.md), [ADR 0918](0918-ail-mcp-compiler-tools.md), [ADR 1113](1113-check-output-for-agents.md)

## Context

The API's position contract (columns are Unicode scalar values, the end is exclusive) was written down and never
tested: the mistakes corpus checks rule IDs, not positions, and no case had non-ASCII text, tabs, CRLF or a
missing final newline. A fix edit carried only a span, so a client applying edits from `ail check --json` to a file
that had changed since the check replaced whatever text was there. `ail api --refs` answered a short list for a
local of an imported module's file without saying it had not looked everywhere. `ail mcp` stayed silent on a
request line that was valid JSON but not an object, and nothing compared each tool's `inputSchema` with what the
tool accepts.

## Decision

1. A new gate suite, `apicontract`, tests the contract on all three hosts: schema and `api` version of every JSON
   record, byte-identical output on a second run, exit statuses; line, column and end of diagnostics over ASCII,
   multibyte, emoji, tab, CRLF against LF, a missing final newline and an empty file, and a call spanning lines;
   fix edits (`expect` matches the source, applying them gives a checking file, `ail fix` is idempotent, `--dry-run`
   writes nothing, a stale edit is refused); MCP `tools/list` against the spec table, a missing required
   argument and a bad argument on every tool, and structured errors.
2. Each edit in `ail check --json` gets an optional `expect` key: the text its span holds in the checked source.
   Adding a key keeps `"api":1` (semantic-api.md, Versioning).
3. `ail edit` gets a `span` op (`edit.op.11`) that takes such an edit and fails, writing nothing, when the span no
   longer holds `expect`.
4. `ail api --refs` adds `complete` and `scope` (`api.refs.4`). `ail context` already says what it cut (`api.ctx.4`)
   and `ail ask --affected` carries `partial`, so an empty answer is told from one that could not search.
5. `ail mcp` answers a non-object JSON line with -32600 and id `null` (`mcp.proto.3`).

## Rejected alternatives

- **A hash of the replaced text instead of the text.** Shorter, but an agent cannot show it, and a mismatch says
  nothing about what changed. The span is short.
- **`expect` required, `"api":2`.** Every consumer would break for a guard most do not need yet.
- **Refuse stale edits inside `ail check`.** `check` never writes; the refusal belongs to the command that applies.
- **Generate the schemas from one table.** Right for 0.2 (the review's point on one definition), too large for a
  patch release; the suite diffs the MCP schemas against the spec table meanwhile.
