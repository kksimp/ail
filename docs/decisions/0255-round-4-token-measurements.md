# 0255. Round-4 token measurements on the agent-tool corpus

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Syntax and library; token efficiency
- **Spec rules:** none (records the measurements behind ADRs 0250–0254)

## Context

The agent-tool corpus (ADR 0217) measured AIL at +28.0% tokens over its
Python twins. The designer allowed design changes that cut tokens, staying
close to the existing design (2026-10-20). Each candidate below was measured
with `tools/tokbench/run.sh` (mean of five tokenizers) on the three programs
as rewritten, by putting back the old form for that candidate alone.

## Measurements

Before and after, with the corpus files regenerated from `examples/`:

| Program | Before | After | Change | Python | After vs. Python |
|---|---|---|---|---|---|
| logscan | 672.2 | 532.6 | −20.8% | 482.2 | +10.5% |
| tasks | 889.4 | 742.6 | −16.5% | 675.2 | +10.0% |
| build | 513.6 | 456.2 | −11.2% | 463.0 | −1.5% |
| **Total** | **2075.2** | **1731.4** | **−16.6%** | **1620.4** | **+6.9%** |

What each change saves (the corpus without that one change, minus the final
corpus):

| Change | ADR | Tokens | Share |
|---|---|---|---|
| Key functions (vs. `Ranked` + `impl Ord`) | 0250 | 83.4 | 4.6% |
| `Json` getters and `?` before `else` (vs. helper functions) | 0252, 0254 | 85.2 | 4.7% |
| `Json.set` (vs. rebuilding the members) | 0254 | 43.2 | 2.4% |
| `Json` getters alone (vs. `get(k)?.as_str()`) | 0254 | 39.2 | 2.2% |
| `std.path` (vs. `split`/`join`) | 0253 | 31.8 | 1.8% |
| `m[k] += 1` on a missing key | 0251 | 27.6 | 1.6% |
| `xs.take(n)` (vs. `[..n.min(len)]`) | 0250 | 5.8 | 0.3% |
| `for x in e else d` | 0252 | 4.8 | 0.3% |

Rejected spellings, measured on `logscan`: `fn(m) key` 533.0, `m => key`
531.6, implicit `it` 529.8, `sorted_by(|m| ...)` 531.8, against 532.6 for
`sorted(by: |m| ...)`: all within 0.5%, a tie (ADR 0250). `j["k"]?.as_str()`
saved 17.0 tokens against 39.2 for `j.str("k")` (ADR 0254).

## Decision

Adopt ADRs 0250–0254. Leave the remaining gap (+6.9%) to the items below.

## Rationale

See each ADR; tokens decided where a change cleared ADR 0076's ~2% threshold,
and AI understanding elsewhere.

## Consequences

- What still costs AIL tokens against Python here: the three `use std.x` lines
  per file (OQ-071), `if`/`else` blocks where Python has a conditional
  expression (`"done" if ok else "failed"`), struct declarations Python
  replaces with tuples and dicts, and `@test` bodies with escaped JSON.
- Measurements are repeatable: the ablation files are produced by reverting one
  change in the corpus files.

## Rejected alternatives

- `Map.count(k)`, `env.program_dir()`, `j["k"]`, implicit `it`, `fn(x)` keys,
  `.by(field)`: see ADRs 0250, 0251, 0253 and 0254.
