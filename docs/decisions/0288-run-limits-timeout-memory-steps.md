# 0288. `ail run` takes `--timeout`, `--max-memory` and `--max-steps`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling for agents
- **Spec rules:** run.limit.1-4

## Context

A runaway program (an endless loop, a list that never stops growing) hangs `ail run` or an agent's test run until something outside kills it, and leaves nothing to debug from. The trap snapshot (ADR 0267) already shows where a trap happened.

## Options considered

### Option A — limits that trap at the next statement (chosen)
- **Pros:** a trap carries the position, rule and snapshot; one flag test per statement when on; no change to programs.
- **Cons:** a single blocking call or one huge allocation is not interrupted.

### Option B — kill the process from the watchdog
- **Pros:** also stops a blocked call.
- **Cons:** no snapshot, no position, no output flushed.

## Decision

Add `--timeout MS`, `--max-memory MB` and `--max-steps N` to `ail run` and `ail test`. A watchdog thread sets a stop flag for time and resident memory; the interpreter checks it and the step count before each statement, and traps with `run.limit.1`, `.2` or `.3`.

## Rationale

Tokens: a trap with the snapshot costs fewer tokens to debug than a killed process (AI understanding decided). Speed: a run without limits pays one predictable branch per statement; measured in the PR.

## Consequences

- An agent can bound any run and get a located trap. `--max-steps` repeats exactly.
- Memory is read from the process (`/proc` on Linux, `ps` elsewhere); Windows rejects `--max-memory` for now.
- Tasks and `par` bodies on other threads check the same flag, with their own step counts.

## Rejected alternatives

- Kill from the watchdog: loses the trap report, the thing the limit is for.
- A default limit: changes existing programs; limits stay opt-in.
