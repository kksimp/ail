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

Known AIL bug found while validating `depgraph`: `mut s := m.get(k) else List[str]()` then `s.pop()` also shrank the list stored in the map (native build; repro `bugs/native-map-get-list-alias.ail` in the project files). The solution copies the list instead, so `validate` does not depend on the fix. Agents hitting it in a trial is real data; note it in that trial's record.
