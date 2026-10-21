# 0679. `ail test --native`: a differential runner for `@test` functions

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.test.1-4](../spec/23-native.md), test.assert.1, native.build.1
- **Builds on:** [ADR 0583](0583-native-coverage-tracker.md) (which programs build natively), [ADR 0085](0085-interim-local-testing-gate.md) (the local gate)

## Context

Every `@test` function in the tree is a ready-made differential test of the
native backend: the interpreter is the oracle (ADR 0032) and a native build
must give the same verdict. The coverage tracker (ADR 0583) compares whole
programs, which have few tests. The backend has no `assert` or `assert_eq`,
refuses to call a `@test` function (test.attr.3), and the prelude names can't
be redeclared, so a test file can't be built as it stands. Lane rules forbid
editing `compiler/mir/lower.ail`, where `assert` would be lowered.

## Decision

`ail test --native [PATH...]` (`bootstrap/src/native_test.rs`):

1. `ail test --json` gives the interpreter's verdict for each test.
2. A rewritten copy of each file is built with `ail build`, beside the
   original so its `use` lines resolve: `@test fn t()` becomes `fn t() int?`
   ending in `return 0`; `assert(c)` and `assert_eq(a, b)` become
   `ail_native_check(c, LINE)` and `ail_native_check((a) == (b), LINE)`; the
   copy's `main` is replaced by a dispatcher that picks the test by the
   length of its first argument (`==` on `str` isn't native yet) and exits
   with the test's result, 1 for a failed check and 3 for a `?` that passed
   `none` on.
3. Blank lines stand in for removed code, so a diagnostic's line maps back
   to the original. A refused test is cut from the copy and the build is
   tried again, up to 12 rounds, which names the first blocking construct of
   every test in about as many builds as there are distinct blockers.
4. Each remaining test runs as its own process (30 s limit) and its exit
   status is compared with the interpreter's.

The whole tree takes about 14 s on four cores, so `bootstrap/tests/native_test.rs`
(a mixed fixture) runs in `./ail selftest` and the full tree stays a manual
command. Counts are in [native-test-runner.md](../design/native-test-runner.md).

## Consequences

- Each native lane gets a per-test pass list as its constructs land; a real
  difference shows up as `DIFF` with the native failure line.
- A test that uses `?` on an error value can't be built: the copy is an
  ordinary function and the backend has no error return for it yet.
- A file inside a package (a directory of modules) can't be built alone, so
  those tests report that instead of a construct.
- When `assert` is lowered natively the rewrite can be dropped.

## Rejected alternatives

- **Lower `assert` and `assert_eq` in `lower.ail`.** The right end state, but
  `lower.ail` is the critical path of 18 queued PRs, and the rewrite gives the
  same verdicts today.
- **Run every test in one binary and report all failures.** A failed check
  would need to unwind, and the backend has no catch; one process per test
  also isolates traps and timeouts.
- **Build one binary per test.** About 1.3 s each is 20 times the cost of one
  build with a dispatcher.
- **Compare printed output instead of a verdict.** Tests print nothing; the
  verdict is what `ail test` reports.
