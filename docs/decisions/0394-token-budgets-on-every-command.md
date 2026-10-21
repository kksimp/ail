# 0394. Token budgets on every command: `--budget N`, most severe first, `--from K`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 22 — Tools; AI semantic API
- **Spec rules:** [budget.out.1–6](../spec/22-tools.md#token-budgets)
- **Builds on:** [ADR 0311](0311-ail-api-card-budget-and-trimming.md) (the token estimate)

## Context

An agent pays for every token a command prints. `ail check compiler` prints
about 4,900 tokens of warnings, `ail effects compiler` about 9,900 and
`ail explain` about 7,400, and the agent usually needs only the first few
errors to act. Today it pipes through `head`, which cuts mid-diagnostic,
keeps whatever came first rather than what matters most, and loses the
count of what was dropped. The plan (finish-plan.md §2.4) asks for
token-budgeted output on every command with a stable scheme.

## Options considered

### Option A — each command trims its own output
- **Pros:** each command knows its structure exactly.
- **Cons:** about fifteen commands to change, each in shared files
  (`main.rs`, `diag.rs`, `interp.rs`), and every new command must remember.

### Option B — one wrapper: run the command as a child, split its output into records, rank and cut (chosen)
- **Pros:** one module, `budget.rs`, and four lines in `main.rs`; every
  current and future command gets budgets for free; output that fits is
  byte-identical, so nothing else changes. Records are recognised from the
  formats AIL already guarantees (diagnostic header `FILE:L:C: severity[rule]`,
  schema v1 JSON lines, `ok`/`FAIL` test lines).
- **Cons:** one extra process start (about 1 ms) when `--budget` is given;
  standard output and standard error are captured separately, so their
  interleaving is lost (each record stays on its own stream).

### Option C — `head -n`-style line limits
- **Pros:** trivial.
- **Cons:** lines are a poor proxy for tokens (a JSON diagnostic is one line
  of ~150 tokens); cuts records in half; keeps the first output, not the
  most severe.

Ranking: errors and traps first, then failed tests, warnings, other lines,
passed tests. A summary line is always kept. Alternatives rejected: ranking
by rule or by file (no reason one rule matters more); dropping warnings
entirely (an agent asked for a budget, not a filter).

Continuation: `--from K` skips the first `K` records in rank order. AIL's
output is deterministic (wishlist 6.3), so the same command gives the same
ranking and `K` is stable. Rejected: an opaque cursor token (more tokens,
nothing gained when output is deterministic).

## Measurements

Default budgets were chosen from the output of every command over the 552
programs in `examples/`, `bootstrap/tests/cases/` and the tokbench corpus
(estimated tokens, ADR 0311; the estimate is 3–17% above the largest of
the five tokbench tokenizers, so a budget here is a ceiling for each):

| Command | p50 | p90 | p95 | p99 | max |
|---|---|---|---|---|---|
| `check` | 0 | 319 | 449 | 1,000 | 2,126 |
| `check --json` | 0 | 531 | 743 | 1,533 | 3,312 |
| `test` | 8 | 319 | 438 | 1,000 | 2,126 |
| `test --json` | 23 | 528 | 743 | 1,533 | 3,312 |
| `effects` | 50 | 344 | 524 | 1,000 | 2,126 |
| `api --text` | 68 | 343 | 526 | 1,497 | 2,126 |
| `run` | 89 | 377 | 575 | 1,707 | 3,289 |

Defaults (`budget.out.5`) sit at about the 99th percentile, so a bare
`--budget` changes almost no single-file output and cuts only the large
cases: `check`, `fix`, `edit` and `test` 1,500; `run`, `explain`, `effects`
and `diff` 2,000; anything else 4,000 (the program card's default, ADR 0311).

Old way against new, on the repository's own large outputs (estimated
tokens, bare `--budget`):

| Task | Without | With `--budget` | Saved |
|---|---|---|---|
| `check compiler` (38 warnings) | 4,860 | 1,420 | 71% |
| `check --json compiler` | 6,750 | 1,432 | 79% |
| `effects compiler` | 9,948 | 1,939 | 81% |
| `explain` (rule list) | 7,406 | 2,033 | 73% |
| `api --text compiler` | 19,759 | 4,017 | 80% |

Each cut output ends in one line saying what was left out and how to get it,
for example
`[budget 1500: records 1-12 of 38 shown (~4860 tokens in all); not shown: 26 warnings; continue with --from 12]`.

## Decision

Option B, with the ranking, the `--from` continuation and the defaults
above. A record is never cut (`budget.out.6`): a command whose output is a
single JSON line is shown whole; its `--text` form splits.

## Consequences

- Every command, including those added later, takes `--budget` and `--from`.
- Commands keep printing deterministic output; that is now load-bearing for
  `--from`.
- Not captured: interleaving between standard output and standard error
  under `--budget`. A program that reads standard input still can (it is
  passed through).
- Windows: the wrapper uses `std::env::current_exe` and pipes, which work
  there as on Linux and macOS (ADR 0004).

## Rejected alternatives

- Each command trims its own output (option A): rejected because about fifteen commands would change in shared files, and every new command would have to remember to do it.
- Line limits in the style of `head -n` (option C): rejected because lines are a poor proxy for tokens (one JSON diagnostic is about 150 tokens), cut records in half, and keep the first output rather than the most severe.
- Ranking by rule or by file: rejected because no reason makes one rule matter more than another.
- Dropping warnings entirely: rejected because an agent asked for a budget, not a filter.
- An opaque cursor token for `--from`: rejected because deterministic output makes a plain record index stable, so a cursor adds tokens for nothing.
