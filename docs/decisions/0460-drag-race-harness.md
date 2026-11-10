# 0460. The drag race is one Python harness that times AIL, native AIL, C and Python from twins

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 28 — Testing strategy (tool)
- **Spec rules:** none (a tool); harness rule IDs `drag.out.1`, `drag.native.1`

## Context

`tools/perfbench/run.sh` timed `ail run` against a Python twin with `date`
and `bc`. The finish plan ([finish-plan.md](../design/finish-plan.md)) needs
one table of AIL, native AIL, `clang -O2`, `gcc -O2` and Python, on Linux and
on macOS, where BSD `date` has no `%N`. Criterion **AI understanding**: one
place, one output, nothing to misread; **speed** of the harness doesn't matter.

## Options considered

### Option A — `race.py` behind the unchanged `run.sh` (chosen)
- **Pros:** `time.perf_counter` is portable; one table builder; Python is
  allowed for measurement ([ADR 0070](0070-token-bench-tool.md)); `run.sh`
  keeps its command and its 5-run default.
- **Cons:** `python3` becomes a requirement of `perfbench` (the Python column
  already needed it; macOS ships it).

### Option B — keep the shell script and add flags
- **Pros:** no new file.
- **Cons:** `date +%s.%N` fails on macOS, `bc` is another dependency, and the
  table with optional columns is awkward in shell.

### Option C — a harness written in AIL
- **Pros:** dogfooding.
- **Cons:** it would measure itself with a clock capability and a process
  module that the native backend is still being measured against; later, not now.

## Decision

- `run.sh [N] [--runs N] [--native] [--c] [--python] [--all] [--md FILE] [--programs F...]`.
  With no column flag it behaves as before (interpreter against Python).
- **`drag.out.1`** Twins `NAME.c` and `NAME.py` sit beside `NAME.ail` (or in the tokbench
  corpus). A column whose stdout differs from the interpreter's prints `DIFF`
  and is left out of every ratio.
- **`drag.native.1`** `--native` runs `ail build FILE -o OUT` (arguments in
  `AIL_BUILD_ARGS`); if the command isn't there yet, the column says `-` and the
  run goes on.
- **`drag.size.1`** `--calibrate SECS` (used by `tools/bench/publish.sh` with 0.3) starts from the size in
  `NAME.args` and doubles it (adds one for fannkuch and binarytrees) until `gcc -O2` runs at least SECS;
  every column runs that size and gcc's output is the reference. Runs under 0.3 s are inside the noise band
  (`tools/perfbench/README.md`), so a published ratio there says nothing. Added 2026-11-10 (slice F1).
- C is built once per program with `-O2 -lm`; a missing compiler drops its column.
- `--md FILE` writes the table, the environment (commit, OS, CPU, compiler and Python
  versions, runs) and the date; `docs/design/drag-race.md` is its output.
- Ratios are `min / min`, below 1 beats the reference.

## Consequences

Lane 143 adds C twins; lane 149's before/after script calls this one. The
integrator runs the same command on macOS ARM64 and commits the second report.

## Rejected alternatives

- Keeping the shell script and adding flags (option B): rejected because `date +%s.%N` fails on macOS, `bc` is another dependency, and an optional-column table is awkward in shell.
- A harness written in AIL (option C): deferred. It would measure itself with a clock capability and a process module that the native backend is still being measured against.
