# AIL documentation

Start here. Documentation is plain Markdown ([ADR 0003](decisions/0003-documentation-in-repo-markdown.md)).

Documents are written for people and for AI agents. They follow the usual order of a language's docs: install, tour, reference, standard library, tooling, then agents.

## Getting started

| Document | What it's for |
|---|---|
| [../README.md](../README.md) | What AIL is, install (no installer: the repository is the toolchain), targets and status |
| [windows.md](windows.md) | AIL on Windows: PowerShell 5.1 and 7, execution policy, where files go |
| [tutorial.md](tutorial.md) | A first hour with AIL: run, write a tool, test, `check`, `explain`, `fix`, `fmt` |

## Tour and guides

| Document | What it's for |
|---|---|
| [how-to.md](how-to.md) | Checked answers to common tasks; `ail ask how WORDS` searches them |
| [quick-reference.md](quick-reference.md) | AIL on one page, for people and AI agents writing programs (non-normative) |
| [vision.md](vision.md) | Identity, goals, and design philosophy |

## Language reference

| Document | What it's for |
|---|---|
| [spec/](spec/README.md) | Normative language specification with stable rule IDs |
| [glossary.md](glossary.md) | Shared terminology |

## Standard library

| Document | What it's for |
|---|---|
| [quick-reference.md](quick-reference.md) | The index of the std modules; `ail card std.MODULE` lists a module's functions and `ail ask std WORDS` finds one |
| [design/cards/](design/cards/README.md) | `ail api --card` output for the standard library, kept fresh by a test |

## Tooling

| Document | What it's for |
|---|---|
| [packages.md](packages.md) | `ail.pkg`, path and git dependencies, `ail.lock`, capability grants, a worked example |
| [design/native-guide.md](design/native-guide.md) | `ail build` for users: what builds natively, traps, MIR and profiling |
| [benchmarks.md](benchmarks.md) | Speed and token tables against C, Rust, Go and Python on Linux, macOS and Windows |

## For agents

| Document | What it's for |
|---|---|
| [agent-benchmark.md](agent-benchmark.md) | What was measured about agents writing AIL against C: tries to working code, cost and correctness, including where AIL lost |
| [for-agents.md](for-agents.md) | The loop, `ail check` output and exit codes, canonical patterns, the mistakes that cost the most and their fixes |
| [language-card-core.md](language-card-core.md) | The core of the language in about 3,000 tokens; `ail card` prints it (non-normative) |
| [language-card.md](language-card.md) | The whole language in one read, within 16,000 tokens (ADR 1110) (non-normative) |
| [../llms.txt](../llms.txt), [../llms-full.txt](../llms-full.txt) | The list of docs to load, and the full card plus how-tos in one file |

## Project

| Document | What it's for |
|---|---|
| [status.md](status.md) | What is decided, specified, implemented, and tested, at a glance |
| [roadmap.md](roadmap.md) | Phased development plan |
| [open-questions.md](open-questions.md) | Unresolved design questions, with options, pros, cons, and recommendations |
| [decisions/](decisions/README.md) | Architecture Decision Records: one per decision, including rejected alternatives |
| [design/](design/) | Non-normative design notes and analyses (below) |

Design documents dated before the Rust removal (ADR 0904) may mention `cargo`, the interpreter or `bootstrap/`; they are history. The gate is `tools/gate/mac-gate.sh` (ADR 0842), and `./ail selftest` runs its native suites (ADR 0865).

Design notes:
- [design/feasibility.md](design/feasibility.md): is AIL achievable, and what
  token-efficiency and performance levels are realistic?
- [design/performance-beyond-c.md](design/performance-beyond-c.md): research
  on performance for systems and allocation-heavy code.
- [design/automatic-parallelism.md](design/automatic-parallelism.md): using
  every core, SIMD and NPUs automatically without changing results.
- [design/toolchain-constraints.md](design/toolchain-constraints.md): hard
  external limits on a fully custom toolchain.
- [design/ai-author-wishlist.md](design/ai-author-wishlist.md): what would
  make AIL easiest and most efficient for an AI author (proposals).
- [design/diagnostics-audit.md](design/diagnostics-audit.md): 109 plausible AI
  mistakes, what each diagnostic says, and what `ail fix` resolves.
- [design/ai-first-plan.md](design/ai-first-plan.md): the 2026-10-20 plan
  for the AI-as-operator features (semantic API, effects and capabilities,
  standard modules), efficiency, and the buildout lanes.
- [design/syntax-measurements-r1.md](design/syntax-measurements-r1.md): first
  real tokenizer measurements of candidate syntax (block style, signatures,
  keywords).
- [design/indentation-examples.md](design/indentation-examples.md): the same
  code in each candidate indentation style (OQ-026).
- [design/ownership-options.md](design/ownership-options.md): value vs.
  reference semantics, borrows, and shared identity, with examples and
  measurements.
- [design/traits-generics-options.md](design/traits-generics-options.md):
  method declarations, trait conformance, generic constraints, and receiver
  mutation, with measurements.
- [design/concurrency-modules-options.md](design/concurrency-modules-options.md):
  task model, task lifetimes, module unit, visibility, and imports, with
  measurements.
- [design/program-measurements-r1.md](design/program-measurements-r1.md):
  tokens of whole programs against six languages, and how well fresh AI agents
  write AIL from the quick reference.
- [design/program-measurements-r2.md](design/program-measurements-r2.md):
  three 500–800 line programs in AIL, Python, TypeScript, Rust and Go.
- [design/gui-path.md](design/gui-path.md): how `std.ui`'s window layer
  moves from the bootstrap's crate shim to platform APIs through `extern c`.
- [design/semantic-api.md](design/semantic-api.md): the machine-readable
  interfaces of `ail`: diagnostics schema, fixes, `ail explain`, `ail api`,
  `ail fix`, with the `api.*` and `fix.*` rules.
- [design/packages.md](design/packages.md) (user guide: [packages.md](packages.md)): `ail.pkg` manifests, dependencies
  and per-dependency capability grants.
- [design/formatting.md](design/formatting.md): the canonical layout `ail fmt`
  writes.
- [design/compile-time-execution.md](design/compile-time-execution.md),
  [design/metaprogramming.md](design/metaprogramming.md),
  [design/reflection.md](design/reflection.md),
  [design/ir-architecture.md](design/ir-architecture.md): topics 16–19:
  compile-time constants, derive generators, compile-time reflection, and the
  compiler tiers.
- [design/native-performance.md](design/native-performance.md): `ail build` output raced against C and Python, where the time goes, and the compiler's own speed (ADR 0390).
- [design/native-guide.md](design/native-guide.md): `ail build` for users: what builds natively, traps, MIR and profiling.
- [design/program-measurements-r5.md](design/program-measurements-r5.md),
  [design/program-measurements-r8.md](design/program-measurements-r8.md): the
  agent-tool corpus against Python after rounds 5 and 8.
- [design/cards/](design/cards/README.md): `ail api --card` output for the
  editor and the standard library, kept fresh by a test.
- [design/program-measurements-r3.md](design/program-measurements-r3.md):
  where AIL's tokens go per construct against Python, and the round-3 changes.

Tools:
- Apps (Garlic, Garlic Press, ail-edit, Clove, Husk, Gridrun, ail-kernel) live in their own repositories: see the repository layout in AGENTS.md.
- [tools/perfbench/](../tools/perfbench/run.sh): drag-race programs (AIL, C and
  Python twins). Run `tools/perfbench/run.sh [RUNS]`; the older interpreter
  results are history in
  [design/interpreter-performance.md](design/interpreter-performance.md).
- [tools/tokbench/](../tools/tokbench/README.md): token measurement harness.
  Run `tools/tokbench/run.sh` (macOS/Linux) or `tools\tokbench\run.ps1`
  (Windows), and it sets itself up.

Contributors, human or AI, should read [AGENTS.md](../AGENTS.md) for the
documentation rules. **A change is not complete until its documentation is.**
