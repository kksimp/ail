# AGENTS.md — Guide for AI agents working on AIL

This file is the operating manual for any AI agent (Claude, Codex, Copilot, etc.)
contributing to AIL. Humans are welcome to read it too. Keep it short, accurate,
and current. If something here is wrong, fix it in the same change that
discovers it.

## What this project is

**AIL** (AI-First Language, pronounced "ale", by **Voltare**) is a native,
garbage-collected, memory-safe systems programming language designed so that
humans and AI agents are both first-class programmers. Source files use `.ail`.
The CLI is `ail` (`ail build`, `ail run`, `ail test`, `ail fmt`, `ail check`).

The project is in **Stage 2: native self-hosting**. The Rust bootstrap and its
interpreter are deleted (D5, ADR 0904); the compiler is written in AIL and builds itself from a
published seed (ADR 0855). The core design is done ([ADR 0086](docs/decisions/0086-finish-core-design-before-stage-2.md))
and the remaining design topics continue alongside. The first milestone is
`./ail run examples/hello.ail` on macOS (met). Every implemented behaviour must
cite its spec rule ID in code and in diagnostics.

## Git workflow

Per [ADR 0028](docs/decisions/0028-git-workflow.md):
- **`main` is the only long-lived branch.** Always reduce back to `main`.
- **Docs and spec changes:** commit directly to `main`.
- **Code changes:** use a short-lived branch. There's no hosted CI.
- **Gate ([ADR 0842](docs/decisions/0842-mac-first-gate.md)):** `tools/gate/mac-gate.sh`
  on the designer's Mac is the gate (the integrator runs it for every chain).
  Lanes do not open their own Mac sessions. Builds and tests run only on the
  designer's machines (the Mac, `ail-linux`, `ail-win`), never in a cloud
  container. There is no cargo; `./ail selftest` is the native gate suites
  (ADR 0865) and `ail test` runs a program's tests.
- **Windows release rule ([ADR 0925](docs/decisions/0925-windows-release-gate.md)):**
  before every release (published seed or tag) the full gate must be green on
  ail-win: `tools\gate\win-gate.ps1` runs the suites of `tools/gate/suites.txt`,
  the same list as the Mac and Linux gate, built with the tree's own stage-2
  compiler, and `tools\self-host\run.ps1` must reach the fixed point.
- **Merging ([ADR 0109](docs/decisions/0109-merge-via-pull-requests.md)):** when
  the gate passes, open a pull request for the branch and merge it through GitHub.
  GitHub deletes the branch on merge (repository setting).
- **One checker:** checker changes go in `compiler/check/` only. There is no
  Rust checker to compare with any more; the parity list `tests/checker_known_new.txt`
  was deleted on 2026-11-08 ([ADR 0740](docs/decisions/0740-self-hosted-checker-parity-gate.md), amended
  by [ADR 0904](docs/decisions/0904-rust-bootstrap-deleted.md)).
- **Native only ([ADR 0865](docs/decisions/0865-no-double-work.md)):** new
  features get no interpreter code. Native-only cases are listed in
  `tests/cases/native_only.txt`.
- **Native lowering ([ADR 0823](docs/decisions/0823-lower-ail-feature-files.md)):**
  a native feature lives in its own `compiler/mir/lower_*.ail` file. It may
  add at most one dispatch line and one state field to `lower.ail`. Only a
  PR that breaks this needs a slot in the lower.ail merge queue.
- Keep open branches to the minimum (one per change in flight). Track every
  branch you create and report it until it's merged and deleted.
- The AI's environment can push branches but can't delete them on GitHub.
  Deletion happens through the pull-request merge, with `tools/verify.sh`
  pruning merged branches as a safety net.
- Commit with clear messages, and push when a change is complete.

## Roles

- **The human is the systems designer.** They set direction and make every
  *serious* decision.
- **The AI is the implementer and lead architect.** It decides routine design
  questions itself, by the designer's priority order, and escalates serious
  ones ([ADR 0076](docs/decisions/0076-delegated-decisions.md)).

### Deciding (ADR 0076)

Choose between options using, in order:
1. **Token usage**, measured with `tools/tokbench`.
2. **AI understanding**: local reasoning, no ambiguity, good diagnostics.
3. **Human readability.**
4. **Speed**: runtime, then compile time.

A criterion only decides when the difference is meaningful. Under ~2% of
tokens on representative code, or confined to rare constructs, counts as a
tie. Record every delegated decision as an ADR with
`Decided by: AI (delegated, ADR 0076)` and the reasoning. Summarise each
batch for the designer.

### Escalating to the designer

Ask the designer (structured questions, pros and cons, recommendation first,
examples in previews or files) only when a decision:
1. changes or contradicts an ADR the designer made;
2. is hard to reverse after Stage 2 starts (core semantic models);
3. materially costs a lower-ranked criterion, especially runtime performance
   users can't opt out of;
4. involves money, legal or licensing matters, accounts, or publishing; or
5. sets the criteria strongly against each other on a high-impact choice.

Also: challenge the designer's assumptions when a choice conflicts with the
project goals. If the designer dismisses a question, **stop and wait**. Back
syntax choices with **measurements** (ADR 0029).

### Things the designer runs

Never ask the designer to type multi-step or OS-specific setup commands.
Anything they run must be a **single self-setting-up script**: `run.sh`
for macOS and Linux, and `run.ps1` for Windows (ADR 0071). The designer works
on macOS (Apple Silicon) from the repository root. Give them the exact command
to paste from there (e.g. `tools/tokbench/run.sh`). Never require paid
services or credentials unless the designer explicitly opts in.

## Documentation rules (mandatory)

Documentation is a first-class deliverable. A change is **not complete** until
its documentation is complete.

| When you… | You must… |
|---|---|
| Get a design decision from the designer | Write an ADR in `docs/decisions/` (see the template), add it to the index, and update `docs/status.md` |
| Identify an unresolved question | Add it to `docs/open-questions.md` with options, pros and cons, and a recommendation |
| Resolve an open question | Move it out of `open-questions.md` into an ADR, and link the ADR from where the question used to be |
| Add or change normative language behavior | Edit `docs/spec/` using rule IDs (see `docs/spec/README.md`) and cite the ADR that justifies it |
| Introduce a new term | Add it to `docs/glossary.md` |
| Finish a roadmap item or change its scope | Update `docs/roadmap.md` and `docs/status.md` |
| Supersede a decision | Never delete the old ADR. Set its status to `Superseded by NNNN` and link both ways |

Style:
- Use plain Markdown, with no generated or site-specific syntax (ADR 0003).
- Write dates as ISO 8601: `YYYY-MM-DD`.
- Use relative links between documents.
- Every topic records its **rejected alternatives** and why they were rejected.
- Write precisely and concisely. Docs are loaded into AI context, so every
  sentence should earn its tokens.
- Never put a model name or model identifier in any committed file.
- Licence: Apache 2.0 with a Runtime Library Exception (ADR 0510). Only
  Apache-2.0-compatible code or algorithm ports may be added; runtime code
  embedded into user programs falls under the exception.

## Repository layout

This repository is the language only. Every app lives in its own repository and project (AGPL-3.0, private): [garlic](https://github.com/kksimp/garlic) (browser), [garlic-press](https://github.com/kksimp/garlic-press) (PDF viewer), [ail-edit](https://github.com/kksimp/ail-edit) (code editor), [clove](https://github.com/kksimp/clove) (compressor), [husk](https://github.com/kksimp/husk) (Mac disk analyzer), [gridrun](https://github.com/kksimp/gridrun) (game) and [ail-kernel](https://github.com/kksimp/ail-kernel) (kernel proof of concept). Each pins an ail commit in `.ail-version`. This repo does not test them, and app bugs and features are filed there; a missing language or std feature is filed here.

```
AGENTS.md                 this file
CLAUDE.md                 imports AGENTS.md for Claude Code
README.md                 project overview for humans
LICENSE, NOTICE           Apache 2.0 with a Runtime Library Exception (ADR 0510)
docs/
  README.md               documentation index (start here)
  vision.md               goals and design philosophy (from the founding brief)
  quick-reference.md      AIL on one page for writing programs (keep it in step with the compiler)
  language-card.md        the whole language within the token budget, for agents
  roadmap.md              phased development plan
  status.md               what is decided, specified, implemented, tested
  open-questions.md       unresolved design questions with pros and cons
  glossary.md             shared terminology
  decisions/              Architecture Decision Records (ADRs), one per decision
  spec/                   normative language specification with rule IDs
  design/                 non-normative design notes, research, and constraints
ail                       command-line entry point; builds the native CLI from the published seed (ADR 0855)
ail.ps1                   the same entry point for Windows
compiler/                 the front end, checker and native backend written in AIL (ADR 0022)
examples/                 example programs; NAME.out holds the expected output
tools/
  gate/mac-gate.sh         the Mac gate; prints only failures (ADR 0842)
  gate/win-gate.ps1        the same gate on Windows (ADR 0925)
  gate/suites.txt          the suites both gates run, one per line
  gate/parity.ail          the parity probe: one line per feature, the same on the three hosts (suite `parity`)
  gate/ready-check.sh      what a lane runs before READY: fmt, check, bash -n on the files it changed
  app-host/macos/          the std.app host: SwiftUI, built by run.sh (ADR 0838)
  bench/                  before and after tables for tokens and speed (tools/bench/run.sh)
  check-doc-links.py      checks that relative links in the Markdown docs resolve
  check-doc-snippets.py   runs `ail check` on every `ail` and `ail fragment` block in the docs, outside `docs/spec/`
  card-spec-audit.py      audits the std functions against the spec (ADR 0698)
  card-wording-audit.py   checks the language card's calls against the spec (ADR 0698)
  check-doc-dupes.py      checks the ADR index, glossary and quick-reference std rows, and the cards' examples, for exact repeats
  coverage/               rule coverage report
  fmt/                    parity.sh: the AIL formatter (compiler/fmt/) against its frozen goldens
  golden/                 golden.py: explain, fmt, fix, api and edit against outputs frozen from the Rust binary (tests/golden/), so they check with no Rust binary (D0)
  explain/                parity.sh: `ail explain` in AIL (compiler/explain/) against its frozen goldens
  fuzz/                   fuzz and differential harness (ADRs 0269-0271)
  native-net/             the native HTTP client against the interpreter, over a scripted local server (ADR 0830)
  native-tls/             the native TLS primitives against the Rust client's test vectors (ADR 0836)
  native/                 writes compiler/rt/native/unicode.ail, the native runtime's Unicode tables (ADR 0567)
  perfbench/              drag-race programs: AIL, C and Python twins
  self-compile/           survey of the native subset over the compiler's own source
  tokbench/               token measurement harness (Python; measurement only, ADR 0070)
  vscode/                 VS Code extension: TextMate grammar and `ail lsp` client (not published)
  verify.sh               designer's one-command test of a pending branch (ADR 0085)
  prune-merged.sh         deletes remote branches already merged into main (ADR 0028)
  self-host/              the three-stage self-host bootstrap and fixed-point check (ADR 0610); run.ps1 on Windows
  rules-audit.sh          audits spec rule IDs against the code and `ail explain` (ADR 0002, ADR 0483)
```

## Settled constraints (summary; the ADRs are authoritative)

- Targets: Linux, Windows, and macOS are all first-class. macOS is ARM64 only (ADR 0004).
- All targets are 64-bit and little-endian (ADR 0007).
- x86-64 baseline is x86-64-v3, with runtime dispatch for higher levels (ADR 0005).
- ARM64 baseline is ARMv8.2-A (ADR 0006).
- x86-64 and ARM64 are developed in **strict lockstep**. Nothing lands unless
  it works and is optimised on both (ADR 0008). For now the merge gate is local
  Linux x86-64 plus macOS ARM64 runs, with no hosted CI (ADR 0085, amending ADR 0027).
- The memory model is abstract and weakly ordered. Safe code is data-race-free.
  Atomics default to SeqCst (ADR 0025).
- **Custom tooling by default.** An existing tool is acceptable only if it meets
  *every* requirement. Prefer R&D over losing a feature (ADR 0009).
- **No LLVM or any external backend, ever.** Clang, gcc and rustc are allowed
  only for benchmarks and differential testing (ADR 0010).
- Resemblance to existing languages is not a design goal. Tokenizer cost is
  still measured (ADR 0011).
- When a constraint blocks something the designer wants, **explain the
  constraint plainly** so the designer can design around it.
- **Mutable value semantics:** values are independent, `mut` is marked at
  call sites, borrows are second-class (no lifetimes), and shared mutation
  only goes through `ref T` (ADRs 0060–0063).
- **Concurrency:** colorless tasks (no async/await), scope-bound, with unique
  handles that must be waited (ADRs 0095–0096). **Modules** are directories,
  `pub` exports, and `use a.b` imports with qualified access (ADRs 0097–0099).
- The compiler, checker, tools and optimising backend are written **in AIL**
  (ADR 0022); the Rust bootstrap (ADR 0023) is deleted, and the published seed
  builds the first native `ail` (ADR 0855).
- See `docs/decisions/README.md` for the full list.
