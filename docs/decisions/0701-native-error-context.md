# 0701. Native `?` with a context message

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** err.ctx.1, err.ctx.2, err.ctx.3, err.prop.5
- **Builds on:** [ADR 0574](0574-native-optional-layout.md)

## Context

`e? "msg"` was outside the native subset, which blocked 4 examples first and 19 problems in all. The interpreter reports `error:`, then one `  context:` line per `?` that carried a message (innermost first), then the return trace.

## Decision

The trace list in global slot 9 gets a first element: a list of context messages (made by `err_set`). `rt.err_context(msg)` appends to it, `err_report` prints `  context:` lines after `error:` and then the trace lines from element 1. `lower_try` evaluates the message only on the failing path (the interpreter does too), in the block that leaves the function, then calls `rt.err_context` before `rt.err_trace`. An absent optional with a context (err.prop.5) does the same after `err_set`. No new global slot.

## Rejected

- A separate global slot for the context list: the slots are scarce (64 total) and the trace already travels with the error.
- Storing contexts in the trace list with a marker prefix: the report would need a scan to put contexts first.
