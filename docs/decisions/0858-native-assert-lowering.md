# 0858. `assert` and `assert_eq` lower natively; `@test` stays a rewrite

- **Status:** Accepted
- **Date:** 2026-10-22
- **Decided by:** AI (delegated, ADR 0076; number allocated by the integrator)
- **Topic:** 19 — testing; 23 — native backend
- **Spec rules:** test.assert.1, native.test.2
- **Builds on:** [ADR 0679](0679-native-test-runner.md), ADR 0855

## Context

`ail test` in AIL (`tools/ailtest/ailtest.ail`) builds tests natively. The
native backend refused `assert`, so the runner rewrote every call into a
checking function, scanning the text for strings, comments and `{}`
interpolations. That scanner is about 100 lines and gets column positions
wrong for multi-line arguments.

## Decision

1. `assert(c)` and `assert_eq(a, b)` lower in `compiler/mir/lower_test.ail`,
   reached by one dispatch line in `lower.ail`. They trap with the
   interpreter's text: `assertion failed`, or `assertion failed: left is A,
   right is B`, at the call (test.assert.1, err.trap.3).
2. `@test` functions and the test dispatcher `main` stay a text rewrite in the
   runner: lowering them needs a test-mode flag through the driver, which is a
   larger change for a later ADR.
3. The runner drops its assert scanner.

## Rejected alternatives

- **Keep the scanner.** Duplicate logic, wrong columns, and `assert` would not
  work in a plain `ail build`.
- **Lower `@test` and the dispatcher now.** More than one `lower.ail` line and
  a driver flag; deferred.
