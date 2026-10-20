# 0268. `--watch NAME` shows a variable's assignments; no debugger protocol for now

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling for AI agents (semantic API)
- **Spec rules:** `trace.watch.1–2`

## Context

A common question while debugging is "how did this variable get its value?"
A debugger answers with watchpoints, but an agent drives a debugger through
many round trips.

## Options considered

### Option A — `--watch NAME`: every assignment to `NAME`, in one run (chosen)
Shows the variable's whole new value and position after `NAME = e`,
`NAME op= e`, `NAME.f = e`, `NAME[i] = e` and `mut NAME := e`, in any function.
- **Pros:** one flag, one run, bounded by the trace budget; cheap: a byte test
  per assignment, and a name comparison only when watching.
- **Cons:** changes through `mut` arguments and mutating methods aren't
  assignments and aren't shown (trace.watch.2); matching is by name, so two
  functions' variables of the same name both show.

### Option B — watch every mutation, including `push` and `mut` arguments
- **Pros:** complete.
- **Cons:** a check on every mutating builtin and every write-back, on hot
  paths shared with the interpreter-speed lane. Deferred.

### Option C — a debugger protocol (DAP) with breakpoints and stepping
- **Pros:** interactive; familiar to editors.
- **Cons:** an agent pays a round trip per step; large to build and keep in
  step with the interpreter; the editor doesn't need it yet.

## Decision

Option A, with Option C rejected for now. Several `--watch` names and
`NAME@function` are possible later additions.

## Rationale

Tokens first: a watch answers the question in one run with a handful of
lines; a stepping session costs a tool call per step. Readability: the output
reads as a log of the variable's history.

## Consequences

- Agents get a cheap debugger: trace for control flow, watch for data,
  snapshot for the failure.
- A debugger protocol can be designed later if the editor needs breakpoints.

## Rejected alternatives

- **Every mutation (B):** cost on hot paths; deferred until measured.
- **Debugger protocol (C):** round trips per step; not needed yet.
- **Watching expressions (`--watch a.b + c`):** needs evaluation at every
  assignment; names cover the common case.
