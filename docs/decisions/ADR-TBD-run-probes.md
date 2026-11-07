# ADR-TBD. `ail run --at FILE:LINE --show a,b`: probes

- **Status:** Proposed (number to be allocated by the integrator)
- **Date:** 2026-11-07
- **Decided by:** Designer (backlog 7.5, agent-features list); the details by AI (delegated, ADR 0076), for integrator review
- **Builds on:** [ADR 0398](0398-record-and-replay.md) (replay), [ADR 0868](0868-ai-first-tooling-after-rust-removal.md) (events and budget), [ADR 0871](0871-native-instrumentation-layer.md) (native instrumentation)
- **Topic:** 22 — Tools; debugging
- **Rule keys:** `probe.run.1-3` (docs/spec/21-trace.md)

## Context

To learn a variable's value at one line an agent adds a `print`, reruns, reads the whole output and removes the print: two edits and a full run per question. Wishlist 7.5 asks for one command that prints chosen values where a line runs.

## Decision

`ail run --at FILE:LINE --show a,b FILE` builds the program with `AIL_INSTRUMENT=probe=LINE:a+b:FILE` (the same instrumentation layer as `--watch`). Before the statement that starts on that line runs, each listed name in scope is written to standard error as `probe: NAME = VALUE at LINE:COL`, using the same value text, event budget and summary line as `--watch`. A name out of scope prints `(not in scope)` and the run goes on. Over `--replay` the probe asks the exact failing run. Unused, it emits nothing (ADR 0868).

## Consequences

One command replaces the print-rerun-remove loop. Values are shown before the line runs; a probe on a loop header fires once per entry, a probe on a body line once per pass. Single sites only; a list of sites or expressions (`--show a.len()`) is a follow-up.

## Rejected alternatives

- **Evaluating arbitrary expressions:** needs the expression lowered in the scope of every hit and the checker's types; names cover the common question and keep the rules small.
- **A debugger session (breakpoints, stepping):** an interactive protocol an agent pays tokens for on every step; a probe is one command and one answer.
- **Printing after the line:** the statement may rebind the name; `--watch` already answers "what did it become".
