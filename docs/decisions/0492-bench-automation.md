# 0492. One command produces the before/after benchmark table

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 28 — Testing strategy (benchmarks)
- **Spec rules:** none (measurement tooling, not language behaviour)
- **Builds on:** [ADR 0029](0029-token-efficiency-benchmarks.md),
  [ADR 0030](0030-performance-benchmarks.md), [ADR 0070](0070-token-bench-tool.md),
  [ADR 0071](0071-one-command-scripts.md), [ADR 0217](0217-agent-corpus-standing-benchmark.md)

## Context

Each speed or token change was measured by hand: build an old revision, run
`tools/tokbench` and `tools/perfbench`, copy numbers into a note. The designer
wants a before and after table for each large batch, and the final report
needs one table covering tokens and speed for every benchmark program.

## Decision

`tools/bench/run.sh [--before REV] [--runs N] [--json]` builds the bootstrap
at `REV` (default `main`) in a temporary git worktree, builds the working tree,
and prints one report:

- **Tokens** for every program in `examples/`, the agent-tool corpus
  (`tools/tokbench/corpus/programs/`) and `tools/perfbench/programs/`, counted
  by `tools/tokbench` as the mean over its offline tokenizers, with the change
  per program and the total.
- **Speed** for the perfbench programs, the agent corpus and the perfbench
  examples: minimum wall time over `N` runs (default 5) for the `before`
  binary, the `after` binary (bytecode tier), the `after` binary with
  `--no-bytecode` (tree walker), the Python twin and the C twin (`cc -O2`) where
  they exist. Ratios are after over each other column; the summary is the
  geometric mean.
- **Output check:** the before and after programs must print the same bytes;
  a difference is reported as `DIFF` and as a warning.

The script is one file for the work and one for the table: `run.sh` builds
and calls `bench.py` (Python standard library only, as tokbench already needs
Python). `docs/design/measurements.md` records the format.

Native is not measured yet, because `ail build` does not exist. It becomes a
column when a native backend lands.

The worktree and its build are removed on exit, so the script leaves no
`target/` directory behind.

## Criteria

- **Token usage** and **AI understanding** do not apply: the script measures
  them and changes no language.
- **Human readability** decided the shape: one table, one command, plain output.
- **Speed** of the script itself is not a criterion; a full run builds two
  bootstraps, so the first run takes minutes.

## Rejected alternatives

- **Extend `tools/perfbench/run.sh` and `tools/tokbench/run.sh` in place.**
  Rejected: both measure one revision. Adding a before revision to each would
  duplicate the build logic, and the two outputs still would not line up.
- **Keep the before build in the user's checkout with `git stash`.** Rejected:
  it changes the working tree, and a failed run leaves it half changed.
- **Report wall-clock per program with `bc` and `date`.** Rejected: `date
  +%s.%N` is not portable to macOS, and Python's `time.perf_counter` already runs
  on both platforms.
- **Use hyperfine.** Rejected: an extra tool to install on macOS and Linux
  (ADR 0009 prefers in-house tooling when it covers every need), and the
  output still needs the same reshaping.
- **Report the mean of the timings.** Rejected: one slow run from the machine
  would move it. Minimum wall time is the standard perfbench measure (ADR 0175).

## Consequences

- Every batch can attach one table. Timing on a shared machine is noisy;
  the minimum over several runs is the number to compare, and the report says
  how many runs were used.
- `tools/perfbench/race.py` ([ADR 0460](0460-drag-race-harness.md)) is the
  drag race against C, Python and native AIL. This table is the before and
  after view of the same programs. The two keep separate scripts because the
  drag race times native binaries and needs the C compiler and `ail build`.
- The corpus and perfbench list live in the script. Adding a program to either
  set adds it to the table with no other change.
