# Contributing to AIL

Thanks for looking. AIL is designed by one person (the designer) and mostly
implemented by AI agents working under that direction. Contributions from
people and from agents are both welcome and go through the same process.

Read [AGENTS.md](AGENTS.md) first. It is the operating manual for every
contributor, not only AI ones: workflow, roles, documentation rules and the
settled constraints.

## How a change gets tested

There is **no hosted CI** ([ADR 0085](docs/decisions/0085-interim-local-testing-gate.md),
[ADR 0842](docs/decisions/0842-mac-first-gate.md)). Builds and tests run on the
designer's own machines: a Mac (macOS ARM64) and Linux and Windows test
machines. So a pull request is tested like this:

1. **You** run the cheap checks before you open the PR, from the repository root:
   - `tools/gate/ready-check.sh`: `ail fmt --check` and `ail check compiler`
     on the files you changed, `bash -n` on changed scripts, no conflict markers.
     The last line must be `READY-CHECK GREEN`.
   - Your own new or changed programs: `./ail run FILE` or `./ail test FILE`,
     compared with their `.out` golden.
   - If you can, the whole gate: `./ail selftest` (which runs
     `tools/gate/mac-gate.sh`). It is slow and prints only failures; the last
     line is `GATE GREEN` or `GATE RED (N failures)`.
2. **The integrator** (a maintainer) collects ready PRs into a batch, merges
   them onto `main` in a scratch branch, and runs the Mac gate once for the
   batch on the designer's Mac.
3. If the batch is green, each PR is merged through GitHub
   ([ADR 0109](docs/decisions/0109-merge-via-pull-requests.md)). If it is red,
   the failing PR is sent back with the gate's output.
4. Before every published seed or release, the same gate must also pass on
   Windows (`tools\gate\win-gate.ps1`,
   [ADR 0925](docs/decisions/0925-windows-release-gate.md)).

So expect a PR to wait for the next batch rather than get an instant green
check. Say in the PR what you ran and on which host.

`./ail` in a checkout runs a stage 2 compiler built from your working tree (the seed builds stage 1, stage 1 rebuilds the tree), so a change to `compiler/` is what `./ail build` uses. Set `AIL_USE_SEED=1` to keep the seed-built stage 1 instead ([ADR 1208](docs/decisions/1208-checkout-ail-is-stage-2.md)).

## Branches and pull requests

- `main` is the only long-lived branch
  ([ADR 0028](docs/decisions/0028-git-workflow.md)). Work on a short-lived
  branch, one change per branch, and open a PR against `main`.
- Keep PRs small: one feature or fix, ideally under ~400 changed lines.
- Merge `main` into your branch before asking for review; do not force-push a
  branch someone else is reviewing.
- The PR template asks for the ADR and spec rule IDs your change implements.

## What a change needs

- **Language or std behaviour** needs a spec rule. Normative behaviour lives in
  [docs/spec/](docs/spec/README.md) under stable rule IDs; code and
  diagnostics cite the rule ID they implement, and every rule has an example.
- **A design decision** needs an ADR in [docs/decisions/](docs/decisions/README.md),
  with the options considered and why the others were rejected. Ask in an
  issue first: ADR numbers are allocated by a maintainer so parallel work does
  not collide.
- **Syntax choices** are backed by measurements
  ([ADR 0029](docs/decisions/0029-token-efficiency-benchmarks.md)), made with
  `tools/tokbench`. The priority order for routine choices is AI
  understanding, then human readability, then speed, then tokens
  ([ADR 0076](docs/decisions/0076-delegated-decisions.md), order set by
  [ADR 1077](docs/decisions/1077-decision-order-ai-understanding-first.md)).
- **Documentation** is part of the change: status, roadmap, glossary and open
  questions as the table in AGENTS.md says.
- **Native only** ([ADR 0865](docs/decisions/0865-no-double-work.md)): the
  compiler, checker and tools are written in AIL. There is no interpreter and
  no Rust; new features are native from the start.
- **No external backends** ([ADR 0010](docs/decisions/0010-no-llvm-backend.md)):
  no LLVM, no C emission, no outside code generators.

Big changes (a new language feature, a new std module, a new target) are
best proposed in an issue before any code, so the design can be agreed first.

## AI-written contributions

Most of this repository was written by AI agents, and AI-written PRs are
normal here. The same rules apply to them: a person is responsible for every
PR they open, whatever wrote it; the PR passes `ready-check.sh`; it cites its
spec rules and ADRs; and the description says what was actually run. Do not
put model names or model identifiers in committed files.

## Reporting bugs and asking for features

Use the issue templates. For a **security** problem, do not open an issue:
follow [SECURITY.md](SECURITY.md).

## Licence

AIL is licensed under Apache 2.0 with a Runtime Library Exception
([LICENSE](LICENSE), [ADR 0510](docs/decisions/0510-licence-apache-2-with-runtime-exception.md)). By
contributing you agree that your contribution is licensed the same way. Only
add code you have the right to contribute under Apache 2.0: no copied code
under incompatible licences, and say where any ported algorithm comes from.
