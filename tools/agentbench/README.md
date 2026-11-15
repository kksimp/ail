# agentbench: AIL vs C and Rust agent benchmark

Harness for [ADR 0916](../../docs/decisions/0916-agent-end-to-end-benchmark.md). Run on a designer machine only; trials call a paid model.

```sh
tools/agentbench/agentbench.py gen ledger          # expected outputs from tasks/ledger/ref.py
tools/agentbench/agentbench.py docs ledger         # doc-size parity check
tools/agentbench/agentbench.py run ledger ail-plain 1   # one trial -> results/ledger/ail-plain-1/
tools/agentbench/agentbench.py run ledger ail-tools 1   # the same plus AGENTS.md, CLAUDE.md and the MCP server
tools/agentbench/agentbench.py run ledger ail-core-card 1   # ail-tools, but DOCS holds only the core card (`ail card`)
tools/agentbench/agentbench.py run ledger c 1
tools/agentbench/agentbench.py validate ledger     # solutions pass and every arm's shims log (no model)
tools/agentbench/agentbench.py batch pilot --max-usd 100   # plans/pilot.txt, in order; resumable
tools/agentbench/agentbench.py batch e13 --max-usd 100     # the 45 pre-numbered counted trials
tools/agentbench/agentbench.py spend               # total recorded spend
tools/agentbench/agentbench.py report
tools/agentbench/agentbench.py trials --plan e13   # per trial: lookups, calls before the first write, failed compiles and tests after it
```

## Spend caps and isolation

`run` and `batch` need `--max-usd`, a hard cap on the whole ledger `results/spend.jsonl` (every agent invocation appends its cost; rerunning a trial never deletes it). `--trial-usd` (default 4) caps one trial. Each invocation gets the CLI's `--max-budget-usd` set to what is left of both caps, and an invocation killed at its timeout is charged its whole budget, so the ledger never undercounts. A trial stopped by a cap counts as a failure (`stopped` in `metrics.json`), never dropped. `batch` stops before the first trial the cap leaves less than $0.50 for. A trial that hits an API error is INVALID: `invalid.json`, not scored, listed by `report`, rerun by the next `batch` (the old attempt is kept as `ARM-TRIAL.invalid-STAMP`); `batch` stops at the first one with exit 4.

The agent runs with every host `CLAUDE*` variable removed, a fresh `CLAUDE_CONFIG_DIR` per trial and `--strict-mcp-config`: none of the machine's user settings, memory, plugins or MCP servers reach it, and with no stored sign-in it can bill only `ANTHROPIC_API_KEY`, which `run` requires. The tool arms get their `.mcp.json` through `--mcp-config`.

Dry run, no model and no spend: `AGENTBENCH_AGENT=tools/agentbench/fake/agent.sh AGENTBENCH_RESULTS=$TMPDIR/ab tools/agentbench/agentbench.py batch pilot --model fake --max-usd 0.6`.

`AGENTBENCH_AGENT` overrides the agent command (default `claude`; it must accept `-p`, `--model` and stream-json output). A task is `tasks/NAME/{task.json,spec.md|issue.md,ref.py,seed/{ail,c,rust},solution/,cases/}`; a change task (`fix`, `refactor`) names a `base` whose spec, oracle, solution and cases it reuses unless it has its own. Tasks: `ledger`, `depgraph`, `ledger-fix`, and `stock` with its four changes on an existing four-module codebase (`stock-locations`, `stock-decimal`, `stock-bugfix`, `stock-refactor`). `till` is a shop back-office program of 14 modules (about 1,050 lines of AIL, 1,350 of Rust, 1,600 of C) that serves only as the base of three changes large enough to need the agent tools: `till-bugfix` (a rounding bug in a shared helper whose callers in other modules patched around it), `till-returns` (a feature across parsing, stock, credit and four reports) and `till-rename` (customers become accounts in input, output and code, checked by `structure.forbid`: regexes no path or text under `src/` may match). `till` itself is too large for a from-scratch trial and is in no plan. `results/` is untracked. Arms: `ail-plain`, `ail-tools` and `ail-core-card` (kit files in `arms/ARM/`, the text `ail new` writes plus the tools it names), `c` and `rust` (std only, `rustc`, overflow checks on); `ail` means `ail-plain`.

`metrics.json` also holds wrapper-logged `fix_rounds` and `versions_to_green` (counted by checksum of `src/` per call, so several tools on one draft are one attempt), `first_check_clean`, `tool_calls`, `toolchain_tokens` and `turns_to_green` (calls) (ADR 0916, amended 2026-11-10). The untouched seed is not an attempt: a failed `./test` on it (reproducing a bug) counts in neither figure (`seed_checked` records it; prep writes the seed's checksum to `.bench/seed`).

Per-trial report (amended 2026-11-15b): `metrics.json` and `trials` give `lookups` (reading `SPEC.md` or `DOCS/`, `ail ask|card|explain|api|doc|help`, `man`, `--help`, MCP lookup tools), `lookups_before_write`, `calls_before_write` (agent tool calls before the first write to `src/`), `failed_compile_versions` and `failed_test_versions` (source versions after the seed). `trials` recomputes them from each trial's transcript and `runs.jsonl`, so it also reads the E13 archive. Lookups and writes are classified from the agent's tool calls by pattern (`classify` in `agentbench.py`); a write done by an unusual shell command may be missed.

DOCS files over 24 KB are split at a heading into `NAME.part1.md`, `NAME.part2.md`, ... in every arm, and every arm's `SPEC.md` ends with the same note, because the agent cuts a tool result at about 30 KB and a `cat` of a larger card cost a second read in E13. A part never ends inside a ``` code block (ADR 0916, amended 2026-11-15c).

## Scripted replays: tools, no model

`replay/run.sh [--only TASK] [--json] [--save FILE] [--compare FILE]` runs the fixed sessions in `replay/tasks/*/steps*.txt` (the eight whole-task tasks) against the current `ail`, and prints tokens written, tokens read and calls per session. Run it before and after a tooling change (`--save`, then `--compare`). It costs no model spend; run it on a designer machine. Format and variants: top of `replay/replay.py`.

## Demo: an agent resolves a real issue

`tools/agentbench/demo/run.sh [MODEL]` runs one recorded agent session on GitHub issue 163 (a report from before the interpreter was removed) in a scratch worktree. The agent may use only `./ail`, read-only `git` and the file tools, and is told to work through `ail task` (`start`, edit, `verify`, `finish`). It decides whether the report still reproduces on the native compiler and adds a regression test (`tests/native/run/NAME.ail` plus its golden), or fixes it. Output (untracked) is in `tools/agentbench/demo/runs/STAMP/`: `transcript.jsonl`, `summary.md` (commands, turns, tokens, cost), `changes.diff`, and `verdict.txt`, a check the agent does not control (each new native test must run to exactly its golden). It calls a paid model: run it only with the designer's go-ahead.

## The E14 readiness fixes (ADR 0916, amended 2026-11-15c)

- The agent works in a private folder outside the results and the tree; `./test` runs `.bench/visible.py`, a copy of
  `visible.py`, so the workspace names no harness path. `agentbench.py audit RESULTS` lists each trial's paths outside its
  workspace and any read that voids it; `report` leaves void trials out.
- `metrics.json` records `docs_bytes` (DOCS and SPEC received), cost by token class, `docs_policy`, `tree` and `agent_cli`.
- `--docs-policy read|reference` picks the prompt's wording about DOCS; `c-nodocs` and `rust-nodocs` give C and Rust no DOCS.
- A shimmed `ail` call on a scratch file (a `.ail` path outside `src/`) is not a fix round.
- Every arm gets a `sed` on PATH that accepts GNU `sed -i` on the Mac too (gsed when installed, else a bare `-i` gets an empty suffix).
- `plans/e14-pilot.txt` is the pilot: 48 trials, cap $10; `plans/e14-heldout.txt` is the held-out block of the full run (180 trials).
- `report` prints the primary measure first: iterations to working code per arm and task class (ADR 0916, 2026-11-15c).
- Held-out tasks (`invjson`, `calc`, `maze`) are not in `tasks/`: set `AGENTBENCH_HELDOUT` to their folder.
