# 0610. A self-host bootstrap harness

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.build.1](../spec/23-native.md)
- **Builds on:** [ADR 0022](0022-staged-self-hosting.md), [ADR 0585](0585-self-compile-survey-tool.md)

## Context

The goal is that the native AIL compiler builds itself (ADR 0022). The
survey tool (ADR 0585) counts what blocks it but cannot tell when it works
or whether the result is right.

## Decision

`tools/self-host/run.sh` ([self-host-harness.md](../design/self-host-harness.md))
runs the standard three-stage bootstrap: Rust-hosted `ail build` makes
stage 1; stage 1 builds the compiler into stage 2; stage 2 builds it into
stage 3. Correct means stage 1, 2 and 3 are byte-identical (the compiler's
output is deterministic) and every `bootstrap/tests/native/run` program
built by the stage-2 compiler matches `ail run` in stdout, exit code and
first trap line (native.build.1). It runs as far as it can and names the
first blocker, so one command shows progress.

## Rejected alternatives

- **Extending the survey tool.** It lowers files one at a time and has no
  notion of an executable; the stages need a different driver.
- **A Rust integration test.** The check is a long pipeline of shell and
  compiler runs, and ADR 0071 asks for a single `run.sh` for what the
  designer runs.
- **Comparing only test programs.** They pass for any correct backend; only
  the fixed point shows the compiler compiles *itself* correctly.
