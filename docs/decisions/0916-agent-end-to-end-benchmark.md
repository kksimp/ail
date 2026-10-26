# 0916. Agent end-to-end benchmark: the same model builds and fixes software in AIL and in C

- **Status:** Proposed
- **Date:** 2026-10-26
- **Decided by:** AI (delegated, ADR 0076); the claim to test is the designer's (2026-10-25)
- **Topic:** 28 Testing strategy (benchmarks)
- **Spec rules:** none (tooling; implements layer 3 of [ADR 0029](0029-token-efficiency-benchmarks.md), extends [ADR 0217](0217-agent-corpus-standing-benchmark.md))

## Context

ADR 0217 left layer 3 open: it measures how many tokens a program costs, not what an agent spends to finish one. The designer expects C to start faster, because a model has seen far more C, and AIL to win over the whole cycle, because its checker, diagnostics and tests catch faults sooner. The claim to test: **AIL finishes correct software more reliably.** A fair test needs the same model, task, specs, tests and budgets on both sides, with failures kept in the record.

## Decision

A harness, `tools/agentbench/`, runs **trials**. A trial gives one model one task in a fresh git repository in one arm (`ail` or `c`) and records the outcome.

**Task kinds.**
- `build`: implement a moderately complex CLI from `SPEC.md`. First task: `ledger` (CSV parsing, validation, aggregation, ordering, exit codes).
- `fix`: take an issue (`issue.md`) against a seeded program with a real defect; investigate, change, test, deliver. Same harness; the hidden cases are the regression plus the old suite. The headline demonstration is a `fix` run on a real issue, published with its full transcript including failures.

**Equivalence rules.**
1. One spec text and one language-neutral case file (`cases/*.jsonl`: args, stdin, expected stdout, exit code). A Python oracle (`ref.py`) generates expected output, so neither arm's author can bias it.
2. Visible cases are a subset of the hidden cases. The agent sees only the visible ones, through `./test`.
3. The arms get the same prompt apart from the language name, the same `./build` and `./test` commands, the same turn and wall-time caps, and the same model.
4. Docs: AIL gets the language card (8,000 tokens, ADR 0276); C gets a reference card of at least the same size (`agentbench.py docs` fails if C's is more than 5% shorter, token-counted by tokbench's mean, ADR 0817). The card carries signatures and pitfalls, never solutions. No arm gets web access.
5. C is built with `-Wall -Wextra -fsanitize=address,undefined` both for the agent and for scoring, so memory faults count as failures, as AIL's trap would. This favours neither arm: it makes C's faults visible, as AIL's are.
6. The model is fixed per run (default `claude-sonnet-5-5`). Model identifiers go in the untracked results, never in committed docs.

**Metrics (per trial, `metrics.json`).**
- Generated (output) tokens, plus input and cache tokens and cost, summed from the agent's stream-json result events.
- Debug rounds: `./build` or `./test` runs that failed, logged by the wrapper scripts, not self-reported.
- Elapsed wall time.
- Human interventions: the harness stands in for a human. If `./test` still fails when the agent stops, it sends "keep going", up to `max_nudges`; each send counts as one intervention.
- Final correctness: build succeeds and every hidden case passes (all-or-nothing), with the pass count kept.

**Trials.** At least 5 per arm per task; report the median, the correct count out of N, and every failing trial's transcript. No cherry-picking: trials are numbered in advance and none is dropped except for harness faults, which are listed.

**Runs.** On the designer's machines only (ADR 0842); trials call a paid model, so each batch needs the designer's go-ahead.

## Rationale

Wrapper-logged debug rounds and oracle-generated cases remove self-report and author bias, which are the two ways an AIL-friendly benchmark gets discredited. All-or-nothing correctness tests the reliability claim directly. Counting the nudge as an intervention makes a "needed help" outcome visible instead of hiding it in elapsed time.

## Consequences

- The C card must be grown to parity before the first counted trial; the placeholder does not yet reach 8,000 tokens.
- A model's prior on C is part of what is measured; the benchmark does not try to cancel it. A later task in a language the model knows less well would separate the two effects.
- One task is a pilot, not proof. The claim needs at least three tasks and a `fix` task before it is stated.

## Rejected alternatives

- **Self-reported debug rounds from the transcript:** depends on the model's wording; the wrapper log is exact.
- **Different docs per arm at whatever size is natural:** confounds tokens with docs size; parity makes the comparison about the language.
- **Partial-credit correctness only:** hides the reliability gap this benchmark exists to test; the count is kept as a secondary figure.
- **Rust or Python as the mature arm:** the designer asked for C; memory-unsafe C is the strongest test of the reliability claim.
