# 1100. `ail test` runs each test in a sandbox

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)), on the designer's instruction to ship every item of the language review in 0.1.0
- **Topic:** 19 — Testing; 20 — AI semantic API
- **Spec rules:** [test.sandbox.1-4](../spec/19-testing.md)
- **Builds on:** [ADR 1072](1072-scoped-runtime-capabilities.md) (`--allow`), [ADR 1075](1075-test-filter-and-fixtures.md) (`@needs` fixtures)

## Context

An agent writes a test and runs it at once. A test that writes next to the source, calls the network or starts a
program can damage the tree or hang, and nothing in the language says so. ADR 1072 made the run-time check; the
test runner is where it is most useful, because the code under test is the code just written.

## Decision

Every test runs with `AIL_ALLOW` set by the runner: `env`, reads under the current directory, the test file's
directory and the package root, and read and write in a fresh per-test directory that is also `TMPDIR`.
`net`, `net.serve` and `proc` are absent, so the run-time check refuses them. `@needs(fs)`, `@needs(net)`,
`@needs(net.serve)` and `@needs(proc)` lift one limit for one test. `ail test --allow all` lifts all of them.
A refusal fails only that test and names the `@needs` to add.

Bare words in `@needs` are capabilities and quoted names are fixtures, so the two lists never clash and the
checker can say which was meant (`test.sandbox.2`).

## Options considered

- **Deny by the compiler (`--deny`) for test builds.** Rejected: all tests of a file share one binary, so one test
  that needs the network would reject the build of the others.
- **A separate attribute `@allow_net`.** Rejected: four attributes for one idea, and `@allow` already names warnings.
- **Fixtures that grant capabilities.** Rejected: hides what a test can do behind a name.
- **A per-file or per-run list only.** Rejected: widening everything for one test weakens the rest.

## Consequences

- Tests that wrote beside the program need `@needs(fs)` or `fs.temp_dir()`. The spec's fixture example changed to the latter.
- It is a check on the standard library's calls (`eff.allow.9` lists the gaps), not an operating-system sandbox.
- A test that reads outside the package (an absolute path) now needs `@needs(fs)`.
- Doc examples and contract tests run in the default sandbox.
