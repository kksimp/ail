# Measurements: the before/after table

Non-normative. How to produce the benchmark table for a batch of work, what each
column means, and what the table does not show
([ADR 0492](../decisions/0492-bench-automation.md)).

## Run it

From the repository root, on Linux or macOS:

```sh
tools/bench/run.sh                 # before = origin/main, 5 timing runs per program
tools/bench/run.sh --before REV    # any git revision
tools/bench/run.sh --runs 10       # steadier timings
tools/bench/run.sh --json          # the same numbers as JSON
```

The first run builds two bootstraps (one at `REV`, one at the working tree)
and sets up the tokbench Python environment. The `REV` build lives in a
temporary worktree that is removed when the script exits. The header prints
the commit used for each side.

## What is measured

| Section | Programs | Measure |
|---|---|---|
| Tokens | every file in `examples/`, the agent-tool corpus (`tools/tokbench/corpus/programs/*/`) and `tools/perfbench/programs/` | tokbench mean over the offline tokenizers (lower is better) |
| Speed | the perfbench examples, the agent corpus, `tools/perfbench/programs/` | minimum wall seconds over `--runs` runs |

Speed programs are grouped, and each group's geometric mean is reported on its own:

| Group | Programs | Why separate |
|---|---|---|
| `agent` | the agent-tool corpus | start-up bound: each runs in a few milliseconds, so the numbers measure start-up |
| `compute` | the ten perfbench classics (binarytrees, collatz, fannkuch, fasta, mandelbrot, matmul, nbody, sieve, spectralnorm, wordfreq) | CPU-bound; the comparison the drag race cares about |
| `micro` | loops, parsum, tensor, tensor_loops, tensor_ops, tensor_ops_loops, race_smoke | targeted micro-benchmarks |
| `examples` | interp, json, sheet, logscan, tasks, build from `examples/` | application-shaped programs |

Speed columns:

| Column | What runs |
|---|---|
| `before` | `ail run` with the `REV` bootstrap (bytecode tier) |
| `after` | `ail run` with the working-tree bootstrap (bytecode tier) |
| `walker` | `ail run --no-bytecode` with the working-tree bootstrap (ADR 0361) |
| `python` | the Python twin (`NAME.py` next to `NAME.ail`), when it exists |
| `c` | the C twin (`NAME.c` next to `NAME.ail`), built with `cc -O2 -lm`, when it exists |
| `a/b`, `a/walker`, `a/py`, `a/c` | `after` divided by the other column; below 1.00 means `after` is faster |
| `output` | `same` when `before` prints the same bytes as `after`, `DIFF` otherwise |

Each program runs with its own directory as the working directory, and
`NAME.args` (if present) is passed after the file to every run.

## Correctness gate

A program is excluded from every ratio it would affect when:

- the `after` run exits non-zero, or its output is unstable across runs;
- a Python or C twin exits non-zero, or prints different bytes from `after`
  (the twin is then dropped from that program's ratios, and the program is
  listed under "Partly excluded");
- the `walker` run exits non-zero or prints different bytes.

Excluded programs are listed by name under the table. A table with excluded
programs is not a complete comparison, and the report says how many programs
are excluded.

## Reading the numbers

- **Compare groups, not one blended number.** The agent corpus measures start-up,
  so its ratios are dominated by the cost of starting the interpreter. Quote the
  `compute` line for CPU performance. A single mean over all groups hides
  the difference.
- **Compare minimum times, not single runs.** On a shared machine, use
  `--runs 10` before quoting a ratio below 0.9 or above 1.1.
- `walker` isolates the bytecode tier: `after` against `walker` is the tier's
  gain on the same binary.
- Python and C twins are reference points. They are not rebuilt by the script
  when their source changes, because the twins are part of the corpus.
- A token change of 2% or less on the mean is within the noise described in
  [ADR 0029](../decisions/0029-token-efficiency-benchmarks.md).
- **Native is not measured.** `ail build` is not on this platform's path yet
  (ADR 0492). When it is, the table gets a native column.

## Card plus program

The language card is part of what a model reads before it writes AIL. The
table reports program tokens only, and a separate line adds the card when the
reference cards are in the tree:

- AIL: `docs/language-card.md` (budget 8,000 tokens; 6,615 on the tokbench mean)
  plus the AIL programs.
- Python: a Python reference card of comparable scope plus the Python twins.

The line covers only programs that have a Python twin. The reference card for
Python lives in `tools/tokbench/reference/` on the branch that adds it; when it
is absent the line is not printed.

## Attaching to a PR

Paste the table's summary block into the PR body, or attach the `--json` output
for the integrator. Put both labels in the heading so the numbers can be
reproduced.
