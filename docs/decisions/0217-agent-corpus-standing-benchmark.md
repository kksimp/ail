# 0217. Keep an agent-tool corpus as the standing token and speed benchmark

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 28 Testing strategy (benchmarks)
- **Spec rules:** none (tooling; amends [ADR 0029](0029-token-efficiency-benchmarks.md) layer 1)

## Context

ADR 0029's layer 1 is a fixed corpus of equivalent programs. The corpus grew
from compilers and data structures (`interp`, `json`, `sheet`), which are
CPU-bound and not what an AI agent writes when it operates a computer. The
designer wants to know continuously whether AIL beats Python on tokens and C
on speed, and AIL's pitch is agent work: read a file, parse text or JSON,
run a command, report. Seven examples did not cover it.

## Options considered

### Option A — three agent-tool programs with Python twins (chosen)
- **Pros:** Covers the work agents do (`std.fs`, `std.proc`, JSON, text
  scans, errors with `?`). Each program is a runnable example with a `.out`
  file, so selftest keeps it honest. Idiomatic Python twins give a fair token
  and speed comparison, and the twins' output is checked byte for byte.
- **Cons:** Python only for now; programs are small, so wall time is mostly
  start-up.

### Option B — wait for the layer 3 agent tasks
- **Pros:** Measures what matters: tokens an agent spends to finish a task.
- **Cons:** Needs an agent harness and paid model calls. Layers 1 and 2 are
  available tonight.

### Option C — add more CPU-bound programs
- **Pros:** Better for the speed goal against C.
- **Cons:** Already covered by `interp`, `json`, `sheet` and `loops`. It does
  not measure agent-tool work.

## Decision

`examples/logscan.ail`, `examples/tasks.ail` and `examples/build.ail`, each
with a Python twin under `tools/tokbench/corpus/programs/NAME/`, join the
standing benchmark set. `tools/tokbench/run.sh` measures their tokens and
`tools/perfbench/run.sh` their wall time against the twin. Results go in
[program-measurements-r3.md](../design/program-measurements-r3.md), section
"Agent-tool corpus", and are repeated whenever the language or interpreter
changes in a way that could move them.

## Rationale

ADR 0076 criterion 1 (tokens) needs a measurement that matches real use, and
it is the benchmark the designer asked for. Option A is the only one that
produces it tonight.

## Consequences

- Any syntax or library change that moves tokens on agent code shows up here
  first. Conventions for the programs are in [ADR 0218](0218-agent-corpus-conventions.md).
- C twins are still missing for the speed comparison; none of these programs
  are CPU-bound, so a C twin adds little until a heavier agent task exists.
- Layer 3 (agent tasks) still needs its own decision.

## Rejected alternatives

- **Option B:** needs paid model calls and a harness; not available tonight.
- **Option C:** duplicates programs already in the corpus.
