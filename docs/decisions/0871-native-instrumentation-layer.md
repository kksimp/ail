# 0871. One native instrumentation layer for coverage and call traces

- **Status:** Accepted
- **Date:** 2026-10-24
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** testing and tooling; 21 — Runtime traces
- **Spec rules:** `native.instr.1-3`, `test.cover.4`, `trace.test.1-2`
- **Builds on:** [ADR 0868](0868-ai-first-tooling-after-rust-removal.md) item 4

## Context

`ail test --coverage` and `--trace` belonged to the interpreter, and deleting
Rust (step D) needs them natively. A source-rewrite `cover` command existed
for coverage only; record/replay (ADR 0868 item 4) will need the same hooks.

## Decision

The lowering can emit instrumentation, chosen by the environment of the
`ail build` that makes the program, and it emits nothing otherwise
(native.instr.1). One layer serves every client: `lines` calls
`rt.instr_hit(ID)` at the start of each statement of the program's own files;
`calls` (and `tests`, which leaves out the test runner's own helpers) calls
`rt.instr_call` at each function's start with the arguments that are numbers,
`bool`s or `str`s. Later modes (values, record) add calls to the same file
pair, `compiler/mir/lower_instr.ail` and `compiler/rt/native/instr.ail`.

## Rationale

Tokens and speed: no flag, no cost. The test runner needs no source rewrite,
so lines match the file. One layer means record/replay does not start over.
Deciding criterion: AI understanding (one mechanism, local reasoning).

## Consequences

- Global slot 56 holds the table; `sys_exit` writes it (one line in `core.ail`).
- Argument values of other types, return values and depth are not traced yet.
- The source-rewrite `cover` command is removed.

## Rejected alternatives

- Keep the source-rewrite `cover`: it counts lines in a rewritten file, covers
  only the one file, and cannot carry call or value events.
- A `--instrument` command-line flag: the compiler's entry would need the
  option threaded through every lowering entry point; the environment reaches
  the same place with no signature change.
