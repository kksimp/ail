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
```

## Spend caps and isolation

`run` and `batch` need `--max-usd`, a hard cap on the whole ledger `results/spend.jsonl` (every agent invocation appends its cost; rerunning a trial never deletes it). `--trial-usd` (default 4) caps one trial. Each invocation gets the CLI's `--max-budget-usd` set to what is left of both caps, and an invocation killed at its timeout is charged its whole budget, so the ledger never undercounts. A trial stopped by a cap counts as a failure (`stopped` in `metrics.json`), never dropped. `batch` stops before the first trial the cap leaves less than $0.50 for. A trial that hits an API error is INVALID: `invalid.json`, not scored, listed by `report`, rerun by the next `batch` (the old attempt is kept as `ARM-TRIAL.invalid-STAMP`); `batch` stops at the first one with exit 4.

The agent runs with a fresh `CLAUDE_CONFIG_DIR` per trial and `--strict-mcp-config`: none of the machine's user settings, memory, plugins or MCP servers reach it, and with no stored sign-in it can bill only `ANTHROPIC_API_KEY`, which `run` requires. The tool arms get their `.mcp.json` through `--mcp-config`.

Dry run, no model and no spend: `AGENTBENCH_AGENT=tools/agentbench/fake/agent.sh AGENTBENCH_RESULTS=$TMPDIR/ab tools/agentbench/agentbench.py batch pilot --model fake --max-usd 0.6`.

`AGENTBENCH_AGENT` overrides the agent command (default `claude`; it must accept `-p`, `--model` and stream-json output). A task is `tasks/NAME/{task.json,spec.md|issue.md,ref.py,seed/{ail,c},cases/}`. `results/` is untracked. Arms: `ail-plain`, `ail-tools` and `ail-core-card` (kit files in `arms/ARM/`, the text `ail new` writes plus the tools it names), `c` and `rust` (std only, `rustc`, overflow checks on); `ail` means `ail-plain`.

`metrics.json` also holds wrapper-logged `first_check_clean`, `tool_calls`, `toolchain_tokens` and `turns_to_green` (ADR 0916, amended 2026-11-10).

## Scripted replays: tools, no model

`replay/run.sh [--only TASK] [--json] [--save FILE] [--compare FILE]` runs the fixed sessions in `replay/tasks/*/steps*.txt` (the eight whole-task tasks) against the current `ail`, and prints tokens written, tokens read and calls per session. Run it before and after a tooling change (`--save`, then `--compare`). It costs no model spend; run it on a designer machine. Format and variants: top of `replay/replay.py`.

## Demo: an agent resolves a real issue

`tools/agentbench/demo/run.sh [MODEL]` runs one recorded agent session on GitHub issue 163 (a report from before the interpreter was removed) in a scratch worktree. The agent may use only `./ail`, read-only `git` and the file tools, and is told to work through `ail task` (`start`, edit, `verify`, `finish`). It decides whether the report still reproduces on the native compiler and adds a regression test (`tests/native/run/NAME.ail` plus its golden), or fixes it. Output (untracked) is in `tools/agentbench/demo/runs/STAMP/`: `transcript.jsonl`, `summary.md` (commands, turns, tokens, cost), `changes.diff`, and `verdict.txt`, a check the agent does not control (each new native test must run to exactly its golden). It calls a paid model: run it only with the designer's go-ahead.
