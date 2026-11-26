# 1192. `ail test` shows the local values of a test that a trap failed

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); no syntax change
- **Topic:** Tooling for agents
- **Spec rules:** [test.run.7](../spec/19-testing.md)
- **Builds on:** [ADR 0886](0886-native-values-mode.md) (`--values`, trace.frame.3), [ADR 0290](0290-test-failure-shows-trap-snapshot.md) (test.run.5)

## Context

A trap in a native program prints its stack without values (trace.frame.1). `ail run --values` and
`ail test --values` print each frame's arguments and locals (trace.frame.3), from a build that stores them at every
statement. Agents do not pass the option: in the final run's 464 transcripts no trial used it, and a failed test was
followed by added `print` lines and another run. The values are the fastest route from a failed test to its cause.

## Decision

**test.run.7:** when a test fails by a trap, `ail test` builds the tests once more with the values kept
(`AIL_INSTRUMENT=values`), runs that test again in its own sandbox with its own fixtures, and, when the second run
fails with the same message, reports that run: its frames with arguments and locals, and a failed contract's
`values:` line. A second run that passes or fails differently leaves the first report as it was.

- Only failed tests run again, and the second build happens once per `ail test` and only when a test failed by a
  trap. A run where every test passes costs nothing more. The default build is unchanged: no run-time cost.
- Left out: property tests (the shrunk input is reported, test.prop.4), doc examples, tests of other modules, a run
  limit's trap (run.limit.1-3: the second run would wait the whole limit again), and runs under `--values` (already
  that build), `--trace`, `--coverage`, `--fuzz`, `--mutate`, `--update`, `--record`, `--replay` and `--max-steps`.
- `--json` carries the same frames in `"snapshot"`.

By the priority order (ADR 1077): AI understanding gains a cause in the first report; nothing is lost in
readability; speed costs one more build of the test file, only on a failure.

## Rejected alternatives

- **Build every test run with the values.** One build, but each passing run pays the per-statement stores, and
  timing-sensitive tests change behaviour.
- **Location tables read from the frame at a trap, with no second build** (the full form of this idea). It needs the
  code generators to write where each local lives at each trap site and to say "not kept" for a value the optimiser
  removed. That is the later step; this decision gives the report now and the tables can replace the second build
  without changing the report.
- **A hint line naming `--values`.** One more turn for the agent on every failure.
- **Run the failed test again only on request (`--rerun`).** The same unused-option problem as `--values`.

## Consequences

- A test that fails by a trap runs twice, so its effects inside the sandbox happen twice. Each run has a fresh
  sandbox directory. A test whose failure does not repeat keeps the first report.
- A failing `ail test` takes one more build of the test file.
