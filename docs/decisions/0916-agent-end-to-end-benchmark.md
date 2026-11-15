# 0916. Agent end-to-end benchmark: the same model builds and fixes software in AIL and in C

- **Status:** Accepted (harness and tasks on main via #1099, `tools/agentbench`; amended 2026-11-10: three arms, tooling metrics, scripted replays; still missing: the full AIL-versus-C runs with logged debug rounds, and published results)
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
6. The model is chosen per run (`--model` or `AGENTBENCH_MODEL`, no default) and recorded only in the untracked results, never in committed files (AGENTS.md).

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

- The C card (`tools/agentbench/docs/c-libc-card.md`) is a general libc/POSIX reference sized to the AIL card; it holds generic building blocks (buffer, hash table, binary search), never a task's solution. `agentbench.py docs TASK` must pass before a counted trial.
- Tasks: `ledger` and `depgraph` (build), `ledger-fix` (fix: the ledger with two reported defects, a lost sign on small debits and a rejected 31st). Each build task keeps a reference `solution/` per arm; `agentbench.py validate TASK` builds them and checks they pass every hidden case, and that a fix seed fails some. This proves each task is solvable in both languages before a model tries.
- A model's prior on C is part of what is measured; the benchmark does not try to cancel it. A later task in a language the model knows less well would separate the two effects.
- Three tasks are a pilot, not proof. The headline `fix` run on a real upstream issue (not the seeded `ledger-fix`) is still to be chosen and recorded.

## Rejected alternatives

- **Self-reported debug rounds from the transcript:** depends on the model's wording; the wrapper log is exact.
- **Different docs per arm at whatever size is natural:** confounds tokens with docs size; parity makes the comparison about the language.
- **Partial-credit correctness only:** hides the reliability gap this benchmark exists to test; the count is kept as a secondary figure.
- **Rust or Python as the mature arm:** the designer asked for C; memory-unsafe C is the strongest test of the reliability claim.

## Amendment (2026-11-10): three arms, wrapper-logged tooling metrics, scripted replays

**Decided by:** AI (delegated, ADR 0076); the AI-first edge plan (`lanes/ai-first-edge-2026-11-10.md`, slice E3) asked for it. No rule IDs: tooling only.

**Context.** The decision above compares AIL with C. It cannot say how much of AIL's result is the language and how much is the agent tooling (`ail task`, `ail context`, `ail edit --test`, MCP), and every tooling slice on the 0.1.0 and 0.2 lists needs a number before and after. Counted trials call a paid model and wait for the designer's go-ahead, so most slices need a cheaper gate too.

**Decision.**
1. **The AIL arm splits in two.** Arms are `ail-plain`, `ail-tools` and `c` (`ail` in old results means `ail-plain`). `ail-plain` is the arm described above: `./build`, `./test`, the card. `ail-tools` is the same plus `AGENTS.md`, `CLAUDE.md` (which imports it) and `.mcp.json` in the workspace, from `tools/agentbench/arms/ail-tools/`. The MCP server runs behind a logging proxy (`agentbench.py mcpproxy`). The prompt is identical in all three arms apart from the language name (rule 3 still holds): the difference between `ail-plain` and `ail-tools` is only what the tooling gives an agent that is not told to use it. `AGENTS.md` is a stand-in for what E1's `ail new` writes and is replaced by it when E1 lands. Both AIL arms use the same card, seeds, build line and hidden cases.
2. **Doc parity (rule 4) is checked against the card only.** The arm's instruction file is the treatment, so it does not count toward the C card's size; `agentbench.py docs` prints its tokens beside the check.
3. **New wrapper-logged metrics** in `metrics.json`, written by the shims, not self-reported: `first_check_clean` (whether the first `ail check` or `./build` the agent ran passed; null if none), `tool_calls` (counts by name: `./build`, `./test`, `ail SUBCOMMAND`, `mcp:TOOL`), `calls_total`, `toolchain_tokens` (tokbench mean of everything the toolchain printed back, summed per call) and `turns_to_green` (logged calls up to and including the first passing `./test`; null if never). A `ail` shim on the agent's `PATH` logs each call; `./test` no longer logs the build it runs inside itself, so a failed build counts as one debug round, not two.
4. **Scripted replays, `tools/agentbench/replay/`.** A session is a fixed list of `write`, `put` and `run` steps for one task; `replay/run.sh` runs each against today's `ail`, counts tokens written and read and calls, requires each session to end green, and prints one table. The eight whole-task tasks (t1 to t8 of the 2026-10-20 study) are rewritten for today's commands. `--save` and `--compare` print a slice's before and after. A flow with a newer tool is a variant, `steps.NAME.txt`, reported as `task/NAME`. Replays cost no model spend and run on a designer machine like everything else (ADR 0842).
5. **Validation covers the new parts.** `agentbench.py validate` builds each of the three arms' solutions and then drives each arm's workspace through its shims (`ail check`, `./build`, `./test`), failing if a log line, an output file or an arm-specific file is missing.

**Rationale.** A third arm costs one more trial set but isolates the tooling's contribution, which is what the benchmark must show if AIL's claim is the whole cycle. Wrapper-logged metrics keep the rule of the original decision: nothing the agent says counts. Replays give each slice a number the day it lands and keep the paid batches for the runs that need a model.

**Consequences.**
- Counted trials of `ail-tools` need E1 and E2 to have landed to be the final arm; until then the stand-in `AGENTS.md` and the existing MCP tools are what is measured, and the trial is labelled by commit.
- The `ail-core` arm of slice E6 is one more entry in `ARMS` and an `arms/` directory. It landed as `ail-core-card` (2026-11-11, ADR 1046 amendment): the `ail-tools` kit, but `DOCS/` holds only the core card, and everything else is `ail card TOPIC` and `ail ask std`. `ail-tools` and `ail-core-card` both carry the joined-up kit text of ADR 1046; the paid E13 run must use it.
- The replay sessions are single flows: they measure the tools' tokens and calls, not whether a model would choose the flow.
- Old `metrics.json` files lack the new fields; `report` shows `-` for them.

**Rejected alternatives.**
- **One `ail` arm with tooling always on:** the language and the tooling stay confounded.
- **Telling the `ail-tools` agent in the prompt to use the tools:** the prompt must match; the share of trials that use them unprompted is itself a result.
- **Counting `AGENTS.md` toward doc parity:** it is the treatment; counting it would either weaken the C card or hide the treatment's cost.
- **Parsing tokens and calls out of the transcript:** depends on the model's output format; the shims are exact.
- **A replay driven by a model for variety:** costs spend and loses determinism; a deterministic script is the point.

## Amendment (2026-11-15): spend caps, agent isolation, pre-numbered plans

**Decided by:** AI (delegated, ADR 0076); the caps implement the designer's $100 approval (cap plan in `lanes/ai-first-edge-2026-11-10.md` §7).

**Decision.**
1. **Caps.** `run` and `batch` require `--max-usd`, a hard cap on a persistent spend ledger (`results/spend.jsonl`, one line per agent invocation, never deleted by a rerun), and take `--trial-usd` (default $4). Each invocation gets the agent CLI's own budget limit set to what is left of both; an invocation with no result event (timeout) is charged its whole budget. A trial stopped by a cap is a failure for its arm, recorded as `stopped`, never dropped.
2. **Isolation.** Each trial runs the agent with a fresh, empty config directory and only the arm's MCP servers (`--strict-mcp-config`), so no user settings, memory or instruction files reach any arm, and the agent can bill only the API key in the environment. Every `CLAUDE*` variable of the host is removed from the agent's environment first (a host Claude session exports its organisation and account, which made the child act as part of that session); only `ANTHROPIC_API_KEY`, `ANTHROPIC_CUSTOM_HEADERS` (the workspace header) and the fresh `CLAUDE_CONFIG_DIR` reach it.
3. **Plans.** `plans/pilot.txt` (ledger x 3 arms x 2), `plans/pilot-rust.txt` (ledger x rust x 2) and `plans/e13.txt` (3 tasks x `ail-plain`/`ail-tools`/`c`/`rust` x 8, trial-major) fix the trials before any run, arms interleaved so a cap stop hits every arm alike. `batch` runs a plan in order and resumes.
4. **Doc parity restored by adding to C, never cutting AIL.** The AIL card grew to 9,406 tokens (tokbench mean of the offline tokenizers, ADR 0817) by 0.1.0, which left the C card 22.5% short. The C card gained general sections (arguments and exit status, ownership and cleanup, reading compiler and sanitizer reports, an undefined-behaviour checklist, generic containers and multi-key ordering) and is now 9,474 tokens. They are the C counterparts of what the AIL card covers (absence and errors, ownership, diagnostics, lists and sorting), with no task's solution in them.
5. **API errors make a trial INVALID.** A trial whose agent hit an API error (the model never answered, or the API failed mid-trial) is a harness fault: it writes `invalid.json` instead of `metrics.json`, is never scored, is listed in the report, and is rerun with a fresh per-trial budget. `batch` stops at the first one (exit 4), so a setup fault cannot burn through a plan as a column of zeros. Its spend stays in the ledger.
6. **A Rust arm (`rust`).** The designer's budget rose to $200 (2026-11-15), and the external review asked for a mature memory-safe language beside C, which separates "AIL helps" from "any strict, safe compiler helps". Rust gets the same prompt apart from the language name, `std` only (no Cargo, no crates, like C's libc only), the build line `rustc --edition 2021 -O -C debug-assertions=on -C overflow-checks=on`, so overflow panics as AIL traps and C's UBSan reports, and a reference card (`docs/rust-std-card.md`, 9,357 tokens, 0.5% under AIL's, inside the 5% tolerance). Each task has a Rust seed and solution; `ledger-fix`'s Rust seed carries the same two defects.
7. **The `ail-tools` kit is what `ail new` writes** (`compiler/scaffold/scaffold.ail`, `agents_md`), with its file path `main.ail` written `src/main.ail` to match the seeds, plus `CLAUDE.md` importing it.

**Rejected alternatives.**
- **A cap checked only between trials:** one runaway trial could pass it; the per-invocation budget makes the cap hold inside a trial.
- **The agent's normal config:** user settings and memory would differ between machines and runs, and could reach a signed-in plan.
- **Go as well as Rust:** a fifth arm halves the trials per cell; Rust is the stronger test (a strict compiler that rejects whole classes of faults), Go adds little over C plus GC.
- **A hand-written agent loop on the raw API:** less like how agents are used, and a second harness to trust.
