# 1101. `std.test`: stubs for files, variables, the clock and HTTP

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)), on the designer's instruction to ship every item of the language review in 0.1.0
- **Topic:** 19 — Testing
- **Spec rules:** [test.stub.1-6](../spec/19-testing.md)
- **Builds on:** [ADR 1100](1100-sandboxed-tests.md) (the sandbox and the per-test directory), [ADR 1072](1072-scoped-runtime-capabilities.md) (the check inside the standard library's calls)

## Context

With the sandbox (ADR 1100) a test can no longer reach the real network, programs or files, so tests of code that
reads a file, reads a variable, asks the time or calls an API need a way to say what those return. Without it an
agent writes a fake by hand around every call, or passes the tests with `@needs` and a real service.

## Decision

`use std.test` gives `files`, `env`, `clock`, `advance` and `http`, each active until the test ends. They are built
on the same place as the scoped capabilities: the runtime's `fs.read_text`, `fs.read_bytes`, `fs.exists`,
`env.get`, `time.now_ms`, `time.sleep_ms` and the HTTP request function ask a table first
(`compiler/rt/native/stub.ail`), so no program is rewritten and nothing is mocked by name.

The table is one file in the test's own temporary directory (`fs.temp_dir()/.ail-stubs`, named by `AIL_STUBS`),
one line per stub. A file, not a runtime slot, because it keeps no state in the collector's view, has the test's
lifetime for free (the directory is removed), works for tasks, and costs one scan of the environment when absent.
Each runtime call reads the table, which is fine for the few calls of a test and is never on the path of a program
that is not under `ail test`.

A sleep on a stubbed clock moves the clock and returns, so a test of a retry loop with `time.sleep_ms` is instant.

## Options considered

- **Function values passed in by the program (dependency injection).** Rejected: the program under test must be
  rewritten for every test, which is what an agent has no reason to do.
- **A global slot holding a table in memory.** Rejected: needs a collector root or a raw block and a slot from the
  integrator's allocation, with the same behaviour as a file.
- **A mock keyed by function name, like Python's `patch`.** Rejected: names the implementation, and AIL has no
  run-time lookup of functions.
- **Stubbing in the compiler (rewrite calls in test builds).** Rejected: two behaviours for one source, and a
  different binary from `ail run`.

## Consequences

- Stubs cover what the spec lists (`test.stub.2-5`); `fs.list_dir`, `fs.walk`, `env.vars()` and the monotonic clock
  are not stubbed. A program that uses them sees the real ones.
- The per-test directory is also set for `--allow all`, so stubs work there.
- Seed: the runtime change needs a new seed before the stage-1 build relies on it.
