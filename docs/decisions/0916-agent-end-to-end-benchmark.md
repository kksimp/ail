# 0916. Agent end-to-end benchmark: the same model builds and fixes software in AIL and in C

- **Status:** Accepted (harness and tasks on main via #1099, `tools/agentbench`; amended 2026-11-10: three arms, tooling metrics, scripted replays; amended 2026-11-15: caps, isolation, Rust arm, harder tasks; first counted run done 2026-11-15, results below and on the wiki's Agent benchmark page; amended 2026-11-15b: seed is not a fix round, stated argument rules, split docs, per-trial report, plan E14)
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
3. **Plans.** `plans/pilot.txt` (ledger x 3 arms x 2), `plans/pilot-rust.txt` (ledger x rust x 2) and `plans/pilot3.txt` (the five stock tasks x 5 arms x 1) and `plans/e13.txt` (8 tasks x `ail-plain`/`ail-tools`/`ail-core-card`/`c`/`rust` x 12 = 480, trial-major; the pilots cost about $0.11 a trial on the small tasks, and 480 trials fit the $190 cap even at three times that on the stock tasks, and `ail-core-card` is in because the `ail-tools` workspace holds both the full card (rule 4) and the core card its kit loads, a doubling no real `ail new` project has) fix the trials before any run, arms interleaved so a cap stop hits every arm alike. `batch` runs a plan in order and resumes.
4. **Doc parity restored by adding to C, never cutting AIL.** The AIL card grew to 9,406 tokens (tokbench mean of the offline tokenizers, ADR 0817) by 0.1.0, which left the C card 22.5% short. The C card gained general sections (arguments and exit status, ownership and cleanup, reading compiler and sanitizer reports, an undefined-behaviour checklist, generic containers and multi-key ordering) and is now 9,474 tokens. They are the C counterparts of what the AIL card covers (absence and errors, ownership, diagnostics, lists and sorting), with no task's solution in them.
5. **API errors make a trial INVALID.** A trial whose agent hit an API error (the model never answered, or the API failed mid-trial) is a harness fault: it writes `invalid.json` instead of `metrics.json`, is never scored, is listed in the report, and is rerun with a fresh per-trial budget. `batch` stops at the first one (exit 4), so a setup fault cannot burn through a plan as a column of zeros. Its spend stays in the ledger.
6. **A Rust arm (`rust`).** The designer's budget rose to $200 (2026-11-15), and the external review asked for a mature memory-safe language beside C, which separates "AIL helps" from "any strict, safe compiler helps". Rust gets the same prompt apart from the language name, `std` only (no Cargo, no crates, like C's libc only), the build line `rustc --edition 2021 -O -C debug-assertions=on -C overflow-checks=on`, so overflow panics as AIL traps and C's UBSan reports, and a reference card (`docs/rust-std-card.md`, 9,357 tokens, 0.5% under AIL's, inside the 5% tolerance). Each task has a Rust seed and solution; `ledger-fix`'s Rust seed carries the same two defects.
7. **Attempts are counted by source version, not by call.** Every logged call records a checksum of `src/`. `fix_rounds` is the number of source versions on which a build, test or checking call failed, and `versions_to_green` the number of versions checked up to the first passing `./test`. The pilot showed why: an agent that runs `ail check`, `./build` and `./test` on one draft logged three failures, so the per-call figures (`debug_rounds`, `turns_to_green`, now shown as calls) charged the tool arms for running the tools their kit names. The per-call figures stay in `metrics.json` and the report.
8. **Harder tasks on an existing codebase.** All 14 pilot trials on the first three tasks were correct, so completion could not separate the arms, and the external review asked for work on code the agent did not write. `stock` is a stock-log CLI of four modules per language (parsing, state and average-cost rules, reports, main), written the same way in AIL, C and Rust, with one Python oracle. It is a `build` task itself and the seed of four more:
   - `stock-locations` (a cross-module feature: per-location quantities, a `move` event, a `where` command);
   - `stock-decimal` (an interface change: quantities become exact thousandths in every module and every caller);
   - `stock-bugfix` (two defects whose symptoms show far from their causes: a division before a multiplication in the store shows as a few cents in the report, and a line counter in main that skips comments shows as wrong line numbers);
   - `stock-refactor` (a one-file program to split into modules without changing behaviour; kind `refactor`, scored by the hidden cases plus a mechanical check: `src/main` at most 90 non-blank lines and at least three other source files).
   The stock tasks also compare stderr exactly (`check_stderr`), since their error lines are part of the behaviour. The stock pilot (25 trials, uncounted) failed two AIL trials on one hidden case, a line ending in `\r`. The spec's field rules made that line invalid, but it never mentioned carriage returns, and the line reader both agents used (`io.read_line`) strips `\r\n` as documented. Hidden cases must test only what the spec states, so the spec now says it in one sentence, as `depgraph`'s already did. The case stays. A change task may carry its own oracle and solution; `validate` checks a change seed fails some hidden cases and a refactor seed passes them all but fails the structure check.
9. **The `ail-tools` kit is what `ail new` writes** (`compiler/scaffold/scaffold.ail`, `agents_md`), with its file path `main.ail` written `src/main.ail` to match the seeds, plus `CLAUDE.md` importing it.

**Rejected alternatives.**
- **A cap checked only between trials:** one runaway trial could pass it; the per-invocation budget makes the cap hold inside a trial.
- **The agent's normal config:** user settings and memory would differ between machines and runs, and could reach a signed-in plan.
- **Go as well as Rust:** a fifth arm halves the trials per cell; Rust is the stronger test (a strict compiler that rejects whole classes of faults), Go adds little over C plus GC.
- **A hand-written agent loop on the raw API:** less like how agents are used, and a second harness to trust.

## Result (2026-11-15): first counted run

`plans/e13.txt` ran in full at commit `40eec283`: 480 of 480 trials, $41.82, no invalid trial, no cap stop, no nudge. Raw data is the `agentbench-e13` pre-release asset.

| Arm | Correct of 96 | Mean fix rounds | Trials with no fix round | Median output tokens | Median cost |
|---|--:|--:|--:|--:|--:|
| `ail-plain` | 85 | 0.97 | 31 | 3,194 | $0.117 |
| `ail-tools` | 83 | 0.88 | 42 | 2,690 | $0.090 |
| `ail-core-card` | 83 | 1.55 | 25 | 2,690 | $0.089 |
| `c` | 85 | 0.38 | 62 | 2,570 | $0.070 |
| `rust` | 85 | 0.47 | 67 | 2,466 | $0.072 |

- **Completion tied.** All 59 incorrect trials built and failed exactly one hidden case, one of two: `depgraph`'s `usage-bad-name` (57, every arm) and `stock-locations`' `loc-usage-where-bad` (2, AIL arms). Both test a rule the specs imply but do not state: that a command-line argument must follow the input grammar for names or SKUs. They are task faults; the counted figures keep them, and the next run states the rule in both specs.
- **C and Rust converged faster and cost less.** The gap is widest on building from nothing (`ledger`, `depgraph`, `stock`) and smallest on changes to existing code; on `stock-refactor` Rust needed more fix rounds than `ail-plain`.
- **The tools helped.** `ail-tools` against `ail-plain`: 16% fewer output tokens, 23% lower median cost, 42 against 31 trials with no fix round. It was the best arm on the two seeded-bug tasks. 73 of 96 `ail-tools` trials ran an `ail` tool unprompted (`ail card`, `ail ask`, `ail fmt`); no trial in either tool arm used the MCP server.
- **The core card alone cost fix rounds** (the most of any arm) for no saving in cost.

**Consequences.** The run does not show that AIL makes agents better at engineering software. Follow-ups: state the argument rule in the `depgraph` and `stock-locations` specs; study the first-draft errors behind AIL's fix rounds (card coverage and diagnostics); harder tasks so completion can separate the arms.

## Amendment (2026-11-15b): fixes from the E13 transcripts, plan E14

**Decided by:** AI (delegated, ADR 0076).

**Decision.**
1. **The seed is not a fix round.** In 35 E13 trials, mostly the bug-fix tasks, the agent first ran `./test` on the untouched seed to reproduce the bug, and `fix_rounds` counted it. Prep now writes the seed's `src/` checksum to `.bench/seed`, and `fix_rounds`, `versions_to_green` and `first_check_clean` skip that version; `seed_checked` records whether the agent looked. For a trial without the file the checksum is recomputed from the task's seed, so E13 can be recounted.
2. **The specs state their argument rules.** `depgraph` says a `deps`/`rdeps` NAME that is not a valid name is a malformed command (hidden case `usage-bad-name`, 57 failures); `stock-locations` says the same of `where SKU` (`loc-usage-where-bad`, 2). The `stock` spec writes the error lines as `error: line K: unknown ` and the SKU itself, with an example, and says the word `SKU` never appears in them: `unknown SKU` was read literally in about 30 trials of every language. No case changed, so a task's pass/fail against E13 differs only where the wording did.
3. **Docs fit the agent's tool-output limit.** The agent cuts a tool result at about 30 KB, so a `cat` of a card near that size cost an extra read (77 of 96 `ail-plain`, 59 of 96 `c` trials) and a growing card would make it certain. Every arm's DOCS file over 24 KB is split at a heading into numbered parts, and every arm's `SPEC.md` ends with one note saying so. The text is unchanged, so doc parity (counted on the source files) is unchanged.
4. **A per-trial report.** `metrics.json` and the new `trials` command give each trial's lookups, calls before the first write, and source versions after the seed that failed to compile or failed `./test`: the figures the E13 analysis needed transcripts for.
5. **Plan E14** (`plans/e14.txt`) repeats E13's 480 trials in the same order so every cell compares, with the large multi-module task appended in its own trial-major block when it lands. It needs the designer's go-ahead.

**Rejected alternatives.**
- **Making the two hidden cases visible:** the hidden cases should test what the spec states, and stating it is the fix; showing it would tell the agent which rule is tested.
- **Dropping the two hidden cases:** argument validation is part of the behaviour and every arm can meet it once stated.
- **Shortening the cards to fit:** the card's size is set by what it must cover (ADR 1110); splitting costs nothing in content and treats every arm alike.
- **Telling the agent to use one tool for reading:** ties the benchmark to one agent's tools; parts of a fixed size work for any agent.
- **Recounting E13's published figures:** they stay as published; `trials` and the new `fix_rounds` are reported beside them for E14.

## Amendment (2026-11-15c): the E14 readiness review, before any paid E14 trial

**Decided by:** AI (delegated, ADR 0076), from the five-reviewer E14 readiness review; items 6 and 10 by the designer
(2026-11-15: "The definition of beating c isn't necessarily raw tokens. It's having the least iterations to fix an issue or
make new code that runs").

**Decision.**
1. **A docs part never ends inside a code block.** `split_doc` cuts only between lines outside a ``` block, so a `#`
   comment in a code block is no longer taken for a heading. The AIL card's part 1 had ended on an open fence in the
   Modules example; it now ends before `## Absence and errors`.
2. **A `\r` line is in a visible case.** depgraph's `crlf` becomes visible and stock gains a visible `crlf-line`, which
   its four change tasks inherit. A hidden-only `\r` rule failed the AIL drafts that read with `lines()`, and every
   arm's spec states the rule already, so showing it no longer tells the agent anything the spec does not.
3. **The workspace names no harness path.** `./test` runs a copy of the visible-case runner in `.bench/`. The agent
   works in a private folder outside the results and the tree, which moves to the trial's `ws/` when it ends. The agent's
   environment loses the `AGENTBENCH_*` and `AIL*` variables, and the E14 jobs give the agents an `ail` from a copy of
   the tree without `tools/agentbench`.
4. **An audit voids a trial that looked.** `metrics.json` records every path outside the workspace that the agent named.
   A trial that read the harness, a hidden case, a solution, a ledger or another trial is void: `report` lists it and
   leaves it out. `agentbench.py audit RESULTS` runs the same check over an archive, E13's included.
5. **What each trial read and paid for.** `metrics.json` records the DOCS and SPEC bytes the agent received
   (`docs_bytes`, as cut by the agent CLI), the cost by token class at prices fitted to E13 (input $2, cache write
   $2.50, cache read $0.10, output $10 per million), the tree commit and the agent CLI version. The report shows the
   median DOCS KB received and the cache-write, cache-read and output cost of each cell.
6. **Docs policy is a switch** (`--docs-policy read|reference`), recorded per trial; `c-nodocs` and `rust-nodocs` arms
   leave the language's DOCS out. The default is `reference` (item 10).
7. **Fix rounds count only calls on the program.** The `ail` shim logs `onsrc: 0` for a call that names a `.ail` file and
   no path under `src/` (a scratch probe), and such calls are no longer fix rounds. Every arm's `./build` and `./test`
   count as before. `./test` prints the build's output when the build fails.
8. **Arm order alternates** between trials in the E14 plans.
9. **The pilot decision rule**, set before the pilot: pooled mean cost per correct trial of the named AIL arm over C's,
   on the pilot's tasks. Above 1.15 is "still behind", below 0.87 "ahead", and between "too close to call". The pilot
   can show a harness fault, a regression, what the till tasks cost, whether C reads its card, and whether the `\r` and
   copy traps occur. It can't show a per-task result or a gap under about 30%. Pilot trials never join the full run.
10. **The win rule (decided by the designer, 2026-11-15).** Beating C means fewer iterations to working code, on new
    code and on changes to existing code; tokens and cost are secondary. The primary measure, per arm and task class
    (from scratch, change, held out, reported apart): the share of trials correct on the hidden cases, and among the
    correct trials the mean versions to green (source versions checked up to the first passing `./test`, the seed
    excluded) with the mean fix rounds (versions whose build, check or test failed). An AIL arm wins a class when its
    mean versions to green is lower than C's and its success is no lower. The headline AIL arm is `ail-core-card` (what
    `ail new` gives a user); every arm is reported. `report` prints this table first.
    The docs policy is `reference` (option (a)): every arm's prompt says DOCS is a reference to consult as needed, and the
    report shows what each trial read. This corrects E13, where no C trial from scratch received its card (0 of 39) and
    every `ail-plain` trial did, so E13's C cells measured C without docs; E13's published figures stay as published.
11. **Three held-out from-scratch tasks** (`invjson`: JSON output and escaping; `calc`: a small evaluator with 64-bit
    overflow; `maze`: a grid and shortest paths) live outside the repository, loaded from `AGENTBENCH_HELDOUT` (the Mac's
    `~/ail-heldout`), so no lane that tunes AIL sees their hidden cases. They are frozen before the pilot and reported
    apart. `invjson` is in the pilot; `calc` and `maze` wait for the full run. They were written by a lane that had read
    E13 transcripts, so they are held out from tuning, not from E13 knowledge.

**Rejected alternatives.**
- **Leaving hidden cases where the workspace can reach them and trusting the agent:** E13 had four depgraph trials that
  looked around the harness path; one that opens a solution voids a run.
- **Counting every `ail` call as before:** AIL agents probe scratch files with `ail run`, which C agents do with `cc`
  outside the shims, so the old count charged AIL alone for probing.
- **Choosing the docs policy here:** it decides whether C reads its card, which moves the headline by 40 to 70%, so it
  is the designer's.
