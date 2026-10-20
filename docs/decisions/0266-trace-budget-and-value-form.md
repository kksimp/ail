# 0266. Traces are budgeted, and show values structurally without user code

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling for AI agents (semantic API)
- **Spec rules:** `trace.run.4`, `trace.val.1–2`

## Context

A loop traced without a limit can print millions of lines, which no agent can
read and which fills a context window. Values can be large (a list of 100,000
items), and a type's `to_str` is user code that may itself trap, loop or
reach an effect.

## Options considered

### Option A — an event budget and a per-value budget (chosen)
`--trace-limit N` events (default 1,000), then count only, with a summary
line giving shown, dropped and per-kind counts. Values cut at 60 characters
in trace lines and 1,000 in watches, JSON snapshots and `--verbose`.
- **Pros:** output is bounded whatever the program does; the summary tells
  the agent what it missed and whether to raise the limit or filter by kind.
- **Cons:** the interesting event may be past the limit; the agent re-runs
  with a filter or a larger limit.

### Option B — a ring buffer of the last N events
- **Pros:** shows what happened just before the end.
- **Cons:** output only at the end, so a hung program shows nothing; the
  trap snapshot already covers "just before the end".

### Option C — unlimited
- **Cons:** unbounded output. Rejected.

## Decision

Option A. Values are shown in their structural form (trait.auto.2) without
calling `to_str`, so showing a value runs no user code.

## Rationale

Tokens first: 1,000 short lines is about 15,000 tokens, a reasonable upper
bound for one look; most traces of small programs fit well under it. Running
no user code keeps tracing from changing behaviour (trace.run.5).

## Consequences

- A type with a custom `to_str` shows its fields in traces, not its custom
  text. That is usually what an agent debugging it wants.
- Dropped events are still counted, which costs a lock per event while
  tracing; tracing is off by default.

## Rejected alternatives

- **Ring buffer:** shows nothing for a program that never ends.
- **Calling `to_str`:** user code inside the tracer could trap or recurse.
- **No value budget:** one large list would use the whole event budget's tokens.
