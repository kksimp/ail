# 1190. `ail check` runs the tests an edit can affect

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the feature on 2026-11-25 (idea 4 of the edge-ideas list). Command behaviour, no language change
- **Topic:** 22 — Tools
- **Spec rules:** [check.tests.1-6, check.exit.1](../spec/22-tools.md)

## Context

`ail check` is the command agents run after every edit. Knowing whether the edit broke a test took a second
command, `ail test`, and each command is a model turn: the ail-train comparison took 11 turns in AIL against 3 in
Python for the same task. `ail verify --diff` already checks and tests what a diff affects, and the 464 transcripts
of the final run show no agent calling it. A tool is used when it is the command the agent already types.

`ail check` takes 16 to 24 ms and `ail test` 200 to 365 ms on a small program (`docs/benchmarks.md`), so a check
that runs every test each time would be ten times slower for nothing after most edits.

## Decision

- After a check with no error, `ail check FILE|DIR` runs the tests `ail test` would run, less the ones with a
  stored pass, and prints `tests: R run, P pass (C cached)` (check.tests.1). No test, no line.
- A pass is stored under a key: the compiler, the test's name and the text of every declaration the test can
  reach (check.tests.2). **Reach is by name.** A declaration is reached when its name, its type's name or its
  trait's name is written in a declaration already reached. Naming a type therefore reaches all of its methods and
  `impl` blocks, which covers the calls the source does not spell (`"{x}"`, operators, `for`, `sorted`).
- A test with an outside effect (`fs`, `env`, `proc`, `clock`, `random`, `net`, `net.serve`, `ui`, `app`) is never
  stored and runs on every check (check.tests.3). Only a pass is stored (check.tests.4).
- A failing test is exit status 1, with `FAIL NAME: MESSAGE`, its place and a `next:` line (check.tests.5).
  Status 2 stays "does not compile".
- `--no-tests` and `AIL_CHECK_TESTS=0` turn the step off, and `--tests` turns it on over the variable
  (check.tests.6). The gates set the variable; `ail test`, `ail verify` and `ail task verify` pass `--no-tests` to
  the check they make, since they run tests themselves.
- The tests run through `ail test --json --filter NAME...`, so sandboxing, fixtures, property inputs and doc
  examples behave as in `ail test`.

## Options considered

- **Key on the checker's reference graph (`compiler/api/impact.ail`).** Rejected for the key. The graph is the
  precise one, and it has holes a cache cannot afford: calls made through a trait by the runtime, references in a
  constant's value, and generated code are not edges of a function. A missed edge is a stale pass, which is worse
  than no feature. The name closure cannot miss a spelled reference, and its cost is a wider reach: on a file of
  unrelated functions it is exact, and a common method name (`len`) pulls in every user method of that name.
- **Run only tests the last git commit's diff reaches (`ail test --affected`).** Rejected: it needs a repository
  and a commit, runs the same tests again on every check until the next commit, and is the graph above.
- **Store passes of tests with outside effects too.** Rejected: an agent that edits a data file and reads
  `0 run (5 cached)` has been told something false.
- **Leave tests with outside effects to `ail test`.** Rejected: the line would report green for a program whose
  test fails. They cost a test run per check; `--no-tests` is there for a program where that is too slow.
- **Report a failing test and exit 0.** Rejected: agents and scripts read the status first.
- **A new command or a flag that turns it on.** Rejected: `ail verify` shows that agents do not find one.
- **Hash each test's machine code after the build.** Rejected for now: exact, and it needs the build that the
  cache is there to avoid.

## Consequences

- `ail check` executes code when the program has tests. It runs them as `ail test` does, with the test sandbox.
- A program whose tests all have stored passes pays the key computation only; one with a changed test pays one
  `ail test` run of the tests that have no pass. Timings on the Mac are in the pull request.
- A program of more than 10,000 lines of its own is not built for its tests by a check: the line says so and
  names `ail test`. Measured on ail-linux: a small program's check with a test run takes 1.2 s (as `ail test`
  does), and the compiler's own 12 tests took 152 s, which is not a check.
- The effect sets come from a second, full analysis, made only for a program that has tests.
- An edit to a comment inside a function runs that function's tests again (the key is the text).
- A name written only in text a program generates at run time is not an edge. Generated `impl` blocks are reached
  through their type's name, like written ones.
- `compiler/checktests/checktests.ail` holds the feature; `check_cmd` in `compiler/main.ail` calls it.
- Windows: the step runs only when the launcher can be started as a program; otherwise the line says the tests
  were not run. Parity is deferred with the rest of the Windows work.
- Not built here: the same cache for `ail test` itself, and a fuzz gate that checks the cache against full runs
  (idea 8 of the same list).
