# 0337. Grow the agent corpus to ten programs with Python twins

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Token efficiency, agent programs
- **Spec rules:** none (measurement only, [ADR 0070](0070-token-bench-tool.md))

## Context

The agent-tool corpus ([ADR 0217](0217-agent-corpus-standing-benchmark.md)) was five
programs and reached parity with Python in rounds 4–5. Rounds 5–7 added
implicit std modules, `fs.glob`, `proc.run` with input and limits, `net.get`,
`for v in ch`, `par` comprehensions and derived JSON, and none of the corpus
used them, so the headline number said nothing about the features agents will
reach for.

## Options considered

### Option A — five new programs, each with a twin (chosen)
- **Pros:** covers the new features with the work an agent does with them
  (scan a tree, tail a log, merge configs, fetch and summarise, run a plan);
  twin outputs are byte-identical, checked by running both.
- **Cons:** twins are idiomatic but still written by us.

### Option B — rewrite the old five around the new features
- **Pros:** no new files.
- **Cons:** it hides regressions (the old numbers stop being comparable).

## Decision

Add `scan`, `tail`, `merge`, `digest` and `runner` to `examples/` and
`tools/tokbench/corpus/programs/` with Python twins, add them to the
agent set in `corpus_from_examples.py` (comments are stripped, tests kept),
and report them in
[program-measurements-r8.md](../design/program-measurements-r8.md). The
config fixtures for `merge` sit in `examples/configs/`, apart from
`examples/data/`, which `dashboard` lists.

## Rationale

Tokens decide first (ADR 0076): the new five are 146.2 tokens (8.6%) under
Python, so the earlier parity holds on newer code. Where AIL still costs more
(`snapdiff`, `logscan`, `csvreport`) the cause was recorded as OQ-110 to
OQ-112 and resolved in ADRs 0340–0342.

## Consequences

- Future token rounds measure ten programs, not five.
- `runner` uses a time limit; switch it to cancellation when lane 72 lands.
- `scan` and `runner` need `git` and `sleep` on the `PATH`, as their twins do.

## Rejected alternatives

- Option B: hides regressions in the earlier numbers.
- Python twins using third-party packages: the corpus compares against what
  an agent has on any machine.
