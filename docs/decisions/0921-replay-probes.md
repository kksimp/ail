# 0921. Probes: `--at FILE:LINE --show a,b` prints chosen values each time a line runs

- **Status:** Accepted
- **Date:** 2026-10-26
- **Decided by:** Designer (2026-10-25, put the agent-feature backlog in order, wishlist 7.5 among it); the details by AI (delegated, ADR 0076), for integrator review
- **Topic:** 21 — Runtime traces
- **Builds on:** [ADR 0268](0268-watch-and-no-debugger-protocol.md) (`--watch`), [ADR 0871](0871-native-instrumentation-layer.md) (one instrumentation layer), [ADR 0398](0398-record-and-replay.md) (replay), [ADR 0823](0823-lower-ail-feature-files.md) (one lowering file per feature)
- **Rule keys:** `probe.run.1-4`

## Context

An agent debugs by adding `print` lines, rerunning and removing them: two edits to the source and a whole run's output for each question. `--watch NAME` answers "when does this variable change", not "what were `i` and `total` each time line 40 ran". A replay (ADR 0398) repeats a failing run exactly, so the question can be asked of the real failure.

## Decision

`ail run --at FILE:LINE --show a,b [FILE]`, alone or with `--replay R`, print one event on standard error each time the first statement starting on that line of that file begins: `probe: FILE:LINE a = V, b = W`. A name that is not a variable in scope there prints `a = <unset>`. Values are cut and escaped like trace values (`trace.val.1-2`), and only the printable types of `--values` are shown (numbers, `bool`, `str`, `char`, enums, and structs, lists and maps of them); other types print `<type>`. Probe events count against the event budget and summary of `--trace` (`trace.run.4`). With `--trace-json` they are JSON objects. Nothing in the source file changes.

Native lowering: the build reads `AIL_INSTRUMENT=probe=FILE:LINE:a|b`, parsed in `compiler/mir/lower_probe.ail` (ADR 0823), which adds one dispatch line and one state field to `lower.ail`. At the statement start it looks each name up in scope and emits the same value-describing call the watch events use. A program built without the mode gets no call and no table (ADR 0871, `native.instr.1`).

## Rationale

Token usage first: one command with one question replaces two source edits and a full output; on a replay it asks the exact failing run. Reusing the watch event path and budget keeps one runtime mechanism and one output format (AI understanding).

## Consequences

A probe is a rebuild, as `--watch` is. `ail test` has no probe yet (it has no `--watch` either). The first probe on a line wins; two probes in one build (`--at` twice) are a later slice. Names the lowering has spilled to registers are read from the scope map, so a value mid-expression on the same line is the value at the statement's start.

## Rejected alternatives

- **A debugger protocol (breakpoints, stepping):** ADR 0268's reasoning stands: an agent wants answers, not a session.
- **Inserting `print` lines in a copy of the source:** shifts line numbers and cannot read values the language does not let a `print` reach.
- **Probing in the interpreter:** there is none (ADR 0904).
