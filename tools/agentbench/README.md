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
