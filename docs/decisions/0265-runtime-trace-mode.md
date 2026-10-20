# 0265. `ail run --trace` writes one event per line to standard error

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling for AI agents (semantic API)
- **Spec rules:** `trace.run.1–3`, `trace.run.5`

## Context

Plan principle "read the result" ([ai-first-plan.md](../design/ai-first-plan.md)):
an agent that runs a program should see what happened without editing the
program to add prints, which costs an edit, a re-run and a revert. The
designer asked that AIL be built around how AI uses the computer. The
interpreter already knows every call, value, effect and contract check.

## Options considered

### Option A — event lines on stderr, selected by kind (chosen)
`--trace[=calls|effects|all]`: `call`, `return`, `contract` (calls) and
`effect`, `spawn`, `wait` (effects), human lines or JSON lines with `--json`.
- **Pros:** one flag; standard output stays the program's, so an agent can
  diff it; JSON lines stream and need no closing bracket; a kind filter cuts
  noise (an agent debugging I/O doesn't need every call).
- **Cons:** a trace of a busy program is long (bounded by ADR 0266).

### Option B — a trace file (`--trace-out PATH`)
- **Pros:** separates trace from diagnostics.
- **Cons:** one more file for the agent to find and read; stderr already
  carries machine-readable diagnostics.

### Option C — a debugger protocol (DAP-style server)
- **Pros:** breakpoints, stepping, inspection on demand.
- **Cons:** an agent drives it with many round trips, each costing tokens
  and latency; needs a client. Rejected (see ADR 0268).

## Decision

Option A. Human lines start `trace:` and indent calls by depth; JSON events
are objects with `"api":1,"trace":KIND`. Positions in the program's first
file are `LINE:COL` (with `"file"` only for other files), since every line
repeating the path costs tokens. Prelude functions are not traced. Tracing is
a byte flag on the interpreter tested once per call, assignment and standard
call; nothing changes when it is off.

## Rationale

Tokens first (ADR 0076): one run with a filtered, budgeted trace replaces
several edit-print-run cycles. AI understanding: every event names its rule
area, and the JSON keys match the diagnostics schema (`api`, `line`, `col`).

## Consequences

- An agent can answer "what did this program do" from one run.
- The editor (`tools/edit/edit.ail`, ADR 0241) can show `--trace --json`
  events in its run panel; not done in this lane.
- `ail test --trace` is not designed yet.

## Rejected alternatives

- **Trace file:** an extra artifact for no gain over stderr.
- **Tracing through stdout:** mixes with the program's output.
- **Debugger protocol:** see ADR 0268.
- **Timestamps in events:** break determinism and byte-stable output.
