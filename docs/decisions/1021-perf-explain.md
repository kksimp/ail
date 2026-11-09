# 1021. `ail perf --explain FN FILE`: optimisation remarks in plain text

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076); the slice is from the AI-first review of 2026-11-09
- **Topic:** 22 — Tools
- **Spec rules:** tool.perf.1-2 ([22-tools](../spec/22-tools.md))
- **Builds on:** [ADR 0897](0897-ail-check-perf.md), [ADR 0890](0890-optimizer-roadmap-to-match-c.md)

## Context

An agent that wants fast code has to read assembly to learn why a loop is slow.
Rust and C give no such answer. ADR 0897 planned `ail check --perf` with the
passes recording why they declined, but it was never built.

## Decision

`ail perf --explain FN FILE` (`compiler/mir/remarks.ail`, `perf_command` in
`compiler/main.ail`, one `RkNote` list in `bce_noting`) prints, per loop of
`FN`: bounds checks removed or kept (each kept access with the reason and a fix
such as "`xs[i + 1]` reaches 1 past the counter: stop the loop 1 earlier"),
vector kernels, the checker's parallel verdict (`ail effects --parallel`, reused
as is) and the calls that stay in the loop with the reason they were not inlined.

- **Same decisions as the build.** `bce_noting` is the body of `bce`; `bce` calls
  it with no notes wanted. The notes are built only when the launcher sets
  `AIL_PERF_EXPLAIN=FN`, so a build without it does one `env.get` per module
  (ADR 0868). The executable is the same either way.
- **Blocker wording is a classification.** `version` returns no reason, so for a
  loop it declines, the reason is read from the shape of each remaining access
  (offset index, product, index read from a list, list passed to a call). The
  verdict always comes from the pass; the reason is advice.
- **Vector kernels** are reported for the whole function (the kernels it calls);
  attributing a kernel to one loop needs the passes to name the loop they rewrote.

## Not in this slice

`--json` output, `perf.*` notes inside `ail check` (ADR 0897 stays Proposed), and
per-loop attribution of a vector kernel. The tokbench before-and-after on a
representative task is recorded when the integrator runs the Mac gate.

## Rejected alternatives

- **Re-deriving the facts from the final MIR.** `bce` keeps a checked copy of every
  versioned loop, so the final code cannot say which loops lost their checks.
- **Reading the assembly.** The thing this replaces.
- **A flag on `ail build`.** A build prints nothing on success; a separate command
  has its own exit status and output.
