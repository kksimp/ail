# 1229. `std.test` golden files and temporary fixture directories

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** Testing; std
- **Spec rules:** [test.temp.1-2, test.golden.1-4](../spec/19-testing.md)
- **Builds on:** [ADR 1100](1100-sandboxed-tests.md) (the per-test directory and read roots), [ADR 1101](1101-test-stubs.md) (`std.test`)

## Context

A test of a formatter, a renderer or a report compares large text with a stored copy. Without a standard way, each
agent writes the read, the compare and the rewrite by hand, and the sandbox (ADR 1100) blocks the write unless the
test takes `@needs(fs)`. A test that needs real files also has to make and clean up a directory by hand.

## Decision

- `test.temp_dir()` and `test.temp_files(map)` make directories inside the test's own temporary directory. The
  runner already removes that directory, so no cleanup call exists and none can be forgotten. Pure AIL over
  `fs.make_temp_dir`; no runtime change.
- `test.golden(name, actual)` reads `testdata/NAME.golden` beside the test file. `ail test` passes the test file's
  directory as `AIL_TEST_DIR`, because the program has no other way to know where its source is. That directory is
  already a read root, so reading needs no grant.
- Rewriting is chosen by the environment of the run, `AIL_UPDATE_GOLDEN=1`, not by a call argument, so a source
  file never contains "update mode" and a normal run can never rewrite a golden file. Only that run's sandbox gets
  `fs.write=DIR/testdata`; the grant is that one directory.
- A mismatch names the first differing line and the command that accepts the new output.

## Options considered

- **An `update` argument on `test.golden`.** Rejected: an agent that left it in would silently pass every run.
- **A runtime or checker change that gives a test its own source path.** Rejected: an environment variable is enough
  and touches no hot file.
- **Cleanup by `defer` in each test.** Rejected: the runner removes the directory anyway.
- **Writing golden files to a `.golden` beside the source file.** Rejected: `testdata/` keeps them out of the way of
  `ail fmt` and package listings and matches the common convention.

## Consequences

- `ail test` sets `AIL_TEST_DIR` for every test; a test binary run by hand has no golden files and fails with a
  message that says so.
- Update mode writes only inside `testdata/`; a test that needs another write still takes `@needs(fs)`.
- Not built here: Windows path forms in `AIL_TEST_DIR` are the test file's path as given and use `/`.
