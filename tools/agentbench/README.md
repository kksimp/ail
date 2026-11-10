# agentbench: AIL vs C agent benchmark

Harness for [ADR 0916](../../docs/decisions/0916-agent-end-to-end-benchmark.md). Run on a designer machine only; trials call a paid model.

```sh
tools/agentbench/agentbench.py gen ledger          # expected outputs from tasks/ledger/ref.py
tools/agentbench/agentbench.py docs ledger         # doc-size parity check
tools/agentbench/agentbench.py run ledger ail 1    # one trial -> results/ledger/ail-1/
tools/agentbench/agentbench.py run ledger c 1
tools/agentbench/agentbench.py report
```

`AGENTBENCH_AGENT` overrides the agent command (default `claude`; it must accept `-p`, `--model` and stream-json output). A task is `tasks/NAME/{task.json,spec.md|issue.md,ref.py,seed/{ail,c},cases/}`. `results/` is untracked.

## Demo: an agent resolves a real issue

`tools/agentbench/demo/run.sh [MODEL]` runs one recorded agent session on GitHub issue 163 (a report from before the interpreter was removed) in a scratch worktree. The agent may use only `./ail`, read-only `git` and the file tools, and is told to work through `ail task` (`start`, edit, `verify`, `finish`). It decides whether the report still reproduces on the native compiler and adds a regression test (`tests/native/run/NAME.ail` plus its golden), or fixes it. Output (untracked) is in `tools/agentbench/demo/runs/STAMP/`: `transcript.jsonl`, `summary.md` (commands, turns, tokens, cost), `changes.diff`, and `verdict.txt`, a check the agent does not control (each new native test must run to exactly its golden). It calls a paid model: run it only with the designer's go-ahead.
