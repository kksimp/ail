# 0326. `ail test --trace` keeps a failing test's trace

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Semantic API (topic 12); modules (topic 5); testing
- **Spec rules:** `trace.test.1–2`, `test.run.6`

## Context

`ail run --trace` writes events to stderr as they happen (ADR 0265). A test run
has many tests, most passing; interleaved events from all of them would bury
the failing one, and the editor's trace pane (ADR 0312) wants the failing
test's events as data.

## Options considered

### Option A — capture per test, report only failures (chosen)
- **Pros:** output is proportional to failures; JSON gets a `trace` list under
  the test's entry; the per-test budget keeps it small.
- **Cons:** a passing test's trace can't be seen (run it with `ail run`).

### Option B — stream every test's events to stderr
- **Pros:** trivial; same as `ail run`.
- **Cons:** thousands of lines for a green suite; the entry has no link to its events.

### Option C — capture all tests' traces into the JSON
- **Pros:** complete.
- **Cons:** the largest output by far, for data nobody asked for.

## Decision

`ail test --trace[=SPEC] [--trace-limit N] [--watch NAME]` records each test's
events and reports them only for a failing test: as `"trace"` in its JSON entry,
or indented under `FAIL` in text. The budget applies per test; dropped events
are counted in a closing line. Property tests are not traced.

## Rationale

Tokens decided (ADR 0076, criterion 1): the output an agent reads is only the
failing path. Tracing is off by default and the interpreter's off cost is
unchanged (trace.run.5).

## Consequences

- Capture is thread-local, so it needs no change to the event call sites.
- A property test's many generated cases are not traced; the shrunk input is
  already reported (`test.prop.4`).

## Rejected alternatives

- Streaming and capture-everything, as above.
