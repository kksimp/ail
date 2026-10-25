# 0883. Native `--max-steps` on the instrumentation layer

- **Status:** Accepted
- **Date:** 2026-10-24
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** run limits; 21 — Runtime traces
- **Spec rules:** `run.limit.3`
- **Builds on:** [ADR 0871](0871-native-instrumentation-layer.md), [ADR 0873](0873-rust-only-features-after-rust-removal.md)

## Context

`--max-steps N` stopped a run after N statements and belonged to the
interpreter. Deleting Rust (step D5) needs it natively. A counter in every
program would cost speed for a flag almost nobody passes.

## Decision

`ail run --max-steps N` and `ail test --max-steps N` build with
`AIL_INSTRUMENT=steps` and run with `AIL_MAX_STEPS=N`. The lowering then
starts each statement of the program's own files with `rt.instr_step(PATH,
LINE, COL)` (compiler/mir/lower_instr.ail, rt/native/instr.ail); past N the
program stops with `PATH:LINE:COL: trap[run.limit.3]` and status 1. Without
the flag nothing is emitted. `ail test` runs each test in its own process, so
it counts per test.

A step is one statement of the program's own files; a `while` turn adds none
of its own (its body's statements count), and the std's own statements are not
counted. The count is the same on every machine.

## Rationale

Speed: zero cost without the flag, and the layer is the one coverage and
traces already use. Deciding criterion: token usage (no new mechanism to
describe), then speed.

## Consequences

- N means statements of the program's files, so a limit found with the
  interpreter is not the same number.
- The counter lives after the call trace in the table behind global slot 56.

## Rejected alternatives

- Count in every build: a few percent on tight loops for a rare flag.
- Count `while` turns as steps: needs a second hook in `lower.ail`; the body's
  statements already stop every loop that does work.
