# Program measurements, round 3

> **Non-normative design evidence,** 2026-10-20.

## Agent-tool corpus

Three programs of the kind an AI agent writes when it operates a computer
(ADR 0217, conventions in ADR 0218), each with an idiomatic Python twin that
prints the same bytes:

- `logscan`: reads a log fixture (`examples/data/logscan.log`), counts levels
  per hour into a table with format specs, ranks the top error messages.
  Uses `std.fs`, `str` methods, `Map`, `Set`, sorting.
- `tasks`: parses `examples/data/tasks.json` with `parse_json`, filters and
  updates tasks, runs `echo` per task through `std.proc`, writes the result
  back with `to_text` and reads it again. Errors with `?` and `else`.
- `build`: lists `examples/`, runs `ail check --json` on each file through
  `std.proc`, parses the diagnostics, prints a summary, exits 1 on errors.

### Tokens

Mean of five tokenizers (`tools/tokbench/run.sh`), header comment removed.

| Program | AIL | Python | AIL vs. Python |
|---|---|---|---|
| logscan | 672.2 | 482.2 | +39.4% |
| tasks | 888.8 | 675.2 | +31.6% |
| build | 513.6 | 463.0 | +10.9% |
| **Total** | **2074.6** | **1620.4** | **+28.0%** |

On the earlier corpus AIL was +10% over Python (round 2), so agent-tool code
costs AIL about three times as much extra. Where the tokens go, from reading
the programs side by side:

- **Optional chains in non-optional functions.** `j.get(k)?.as_str()` works
  only in a function that returns an optional, so `tasks` and `build` each
  need helper functions (`str_field`, `num_field`, `to_finding`) that Python
  replaces with `j.get(k)` and an `isinstance` check inline.
- **No sort key.** Ranking by count needs a struct plus `impl Ord` and a
  negated count (`Ranked`), where Python writes `sorted(..., key=...)`.
- **No counters or lambdas.** `(m.get(k) else 0) + 1` against `Counter`;
  building a table row needs a loop and a `mut` string instead of a
  comprehension and `join`.
- **No `dirname`.** Finding the fixture takes `split("/")`, a slice and
  `join`, on every program (three lines each).
- **Statement `else` blocks.** `x := f() else` plus an indented block with
  `continue` is two to three lines where Python's `if x is None: continue`
  is two.

Candidates for the token drive (lane 21): a sort-by-key method or `sorted`
with a key function, `?` inside functions that return a result when the
optional's `else` is wanted anyway, a `count`-style map update, and
`env.program_dir()`.

### Speed

`tools/perfbench/run.sh 10`, Linux x86-64, wall time including process
start-up, in seconds. The programs are small and I/O-bound, so this mostly
measures start-up and spawning.

| Program | AIL min | AIL median | Python |
|---|---|---|---|
| logscan | 0.005 | 0.005 | 0.027 |
| tasks | 0.011 | 0.013 | 0.040 |
| build | 0.978 | 1.037 | 1.013 |

`build` spawns `./ail check` about fifteen times, so both versions spend
nearly all their time in those processes. There is no C twin: none of the
three programs is CPU-bound, so the C comparison lives in the existing
`loops`/`interp` measurements.
