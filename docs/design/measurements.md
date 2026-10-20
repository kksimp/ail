# Measurements: the before/after table

Non-normative. How to produce the benchmark table for a batch of work, and
what each column means ([ADR 0492](../decisions/0492-bench-automation.md)).

## Run it

From the repository root, on Linux or macOS:

```sh
tools/bench/run.sh                 # before = main, 5 timing runs per program
tools/bench/run.sh --before REV    # any git revision, e.g. the batch's base
tools/bench/run.sh --runs 10       # steadier timings
tools/bench/run.sh --json          # the same numbers as JSON
```

The first run builds two bootstraps (one at `REV`, one at the working tree)
and sets up the tokbench Python environment. The `REV` build lives in a
temporary worktree that is removed when the script exits.

## What is measured

| Section | Programs | Measure |
|---|---|---|
| Tokens | every file in `examples/`, the agent-tool corpus (`tools/tokbench/corpus/programs/*/`) and `tools/perfbench/programs/` | tokbench mean over the offline tokenizers (lower is better) |
| Speed | the perfbench examples, the agent corpus, `tools/perfbench/programs/` | minimum wall seconds over `--runs` runs |

Speed columns:

| Column | What runs |
|---|---|
| `before` | `ail run` with the `REV` bootstrap (bytecode tier) |
| `after` | `ail run` with the working-tree bootstrap (bytecode tier) |
| `walker` | `ail run --no-bytecode` with the working-tree bootstrap (ADR 0361) |
| `python` | the Python twin (`NAME.py` next to `NAME.ail`), when it exists |
| `c` | the C twin (`NAME.c` next to `NAME.ail`), built with `cc -O2`, when it exists |
| `a/b`, `a/walker`, `a/py`, `a/c` | `after` divided by the other column; below 1.00 means `after` is faster |
| `output` | `same` when `before` and `after` print the same bytes, `DIFF` otherwise, `err` if either exits non-zero |

Native is not a column yet: there is no `ail build` (ADR 0492).

The summary is the geometric mean of each ratio over the programs where both
columns exist.

## Reading the numbers

- Compare minimum times, not single runs. On a shared machine, use
  `--runs 10` before quoting a ratio below 0.9 or above 1.1.
- `walker` isolates the bytecode tier: `after` against `walker` is the tier's
  gain on the same binary.
- Python and C twins are reference points. They are not rebuilt by the script
  when their source changes, because the twins are part of the corpus.
- A token change of 2% or less on the mean is within the noise described in
  [ADR 0029](../decisions/0029-token-efficiency-benchmarks.md).

## Attaching to a PR

Paste the table into the PR body, or attach the `--json` output for the
integrator. Put the `REV` and both labels in the heading so the numbers can
be reproduced.
