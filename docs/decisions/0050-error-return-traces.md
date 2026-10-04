# 0050. Errors carry automatic return traces; an optional message adds context

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 8 — Error handling; 26 — Debugging
- **Spec rules:** [err.ctx.1–3](../spec/08-errors.md)
- **Partly resolves:** OQ-029
- **Evidence:** cl100k tokens: `read_file(path)?` 7; with `? "msg"` 11; with `.context("msg")?` 13

## Context

Propagated errors need to show where they came from (for debugging and for AI
repair) and sometimes why the operation was attempted.

## Options considered

### Option A — Automatic trace plus an optional message (chosen)
- **Pros:** No extra tokens in the common case, yet every error shows its
  full path. Messages appear only where they add meaning.
- **Cons:** A small runtime cost per propagation (recording a location), and
  a new syntax form for the message.

### Option B — Explicit `.context()` only
- **Pros:** No cost unless used.
- **Cons:** Most errors end up without context. More tokens when used.

### Option C — Automatic trace only
- **Pros:** Simplest.
- **Cons:** Says where, but never why.

## Decision

1. Each propagation through `?` records its source location in the error's
   **return trace**, with no syntax required (inspired by Zig's error return
   traces).
2. `e? "message"` propagates like `e?` and also attaches `message` as
   human-readable context.
3. Error reports show the error, any context messages, and the trace.

## Consequences

- Recording must be cheap: a bounded per-task trace buffer, with no heap
  allocation on the error path. The cost is reported, like any other runtime
  cost.
- Traces and messages are structured data in the AI semantic API and in
  diagnostics.
- Whether the message can be an interpolated string depends on the string
  design.

## Rejected alternatives

- **`.context()` only:** context is too often missing.
- **Trace only:** gives location without reasons.
