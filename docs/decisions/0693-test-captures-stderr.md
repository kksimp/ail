# 0693. `ail test` captures a test's standard error

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; lane 215, night run)
- **Topic:** Testing (lane 215)
- **Spec rules:** extends `test.run.2` in [docs/spec/19-testing.md](../spec/19-testing.md)
- **Confirms:** [ADR 0167](0167-test-functions-and-ail-test.md) (tests capture what they print)

## Context

`io.eprint` writes straight to the terminal, so a test's standard error bypasses the runner. A test that logs cannot be checked, and a failing test's error text is lost in the run's output. `std.log` writes to standard error only, so its tests (lane 211, PR #249) could not run under `ail test`.

## Decision

`io.eprint` inside `ail test` appends to a per-test buffer, the way standard output already does. A failing test shows that text under its `FAIL` line, after its standard output and marked `!`. A passing test's standard error is not shown. The JSON report is unchanged: it still covers only the `test` status lines. Under `ail run`, standard error is written as before.

Reading the captured text from inside a test is **not** part of this decision. It needs a new standard-library function, which is a language change, and so it is left to the designer.

## Rejected alternatives

- **Merge standard error into the standard-output capture.** Rejected: the report would lose which stream a line came from, and `!` costs nothing to show it.
- **Show passing tests' standard error as well.** Rejected: it reproduces the noise the capture removes; a passing test's text is not part of its result.
- **A `captured_err()` function now.** Rejected for this lane: a new standard-library function changes the language, so it needs the designer's decision (see the lane report).

## Consequences

- `bootstrap/tests/test_stderr.rs` checks the failing and passing cases and that `ail run` still writes standard error directly.
- Tests that print to standard error no longer print it on the terminal when they pass.
