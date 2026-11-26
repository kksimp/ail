# AIL

**AI-First Language**, by Voltare. Pronounced "ale".

AIL is a native, memory-safe, garbage-collected systems programming language
designed from the ground up for humans **and** AI agents. It compiles to fast,
efficient native code, with canonical formatting and AI-first tooling: a compiler that answers
structured questions about your program and checks an agent's changes.

```
main.ail      ail build · ail run · ail test · ail fmt · ail check
```

> **Status:** Stage 2, native self-hosting. The Rust bootstrap is deleted
> ([ADR 0904](docs/decisions/0904-rust-bootstrap-deleted.md)); `ail` builds itself.
> See [docs/status.md](docs/status.md). Current release: v0.2.0
> ([changelog](CHANGELOG.md)).

**What to expect.** AIL is pre-1.0: the language and std still change, and spec
rule IDs are stable while the behaviour behind them may not be. The compiler,
the std and the tools are written in AIL by AI agents under the direction of a
human designer, who makes the serious design calls; every test gate runs
on the designer's own machines (there is no hosted CI). What is implemented is listed,
with its tests, in [docs/status.md](docs/status.md); experimental or not yet
built: Windows ARM64, GPU compute (`std.gfx`) and NPUs. `std.audio` decodes FLAC and
Opus but is young and not gated on every host. The TLS and
cryptography are AIL's own and have had **no third-party audit**
([SECURITY.md](SECURITY.md)).

Models have seen little AIL, so an AI agent needs more tries to reach working code in AIL than
in C today. How much more, what the compiler's diagnostics recover, and where the measured
runs fell short of the target are in [docs/agent-benchmark.md](docs/agent-benchmark.md).

## Try it

From the repository root on macOS or Linux:

```sh
./ail run examples/hello.ail
```

```
fn main()
    print("Hello, world!")
```

The first run downloads the published seed compiler, checks it, and builds the
native `ail` with it into `.native/`. On Windows use `.\ail.ps1 run examples\hello.ail`.
Seeds are published for macOS ARM64, Linux x86-64, Linux ARM64 and Windows x86-64.
Windows notes (PowerShell 5.1 and 7, execution policy, where files go):
[docs/windows.md](docs/windows.md).
New to AIL? Start with the [tutorial](docs/tutorial.md).

## Targets

Linux, Windows, and macOS are all first-class.

| | x86-64 (v3) | ARM64 (v8.2-A) |
|---|---|---|
| Linux | ✓ | ✓² |
| Windows | ✓ | experimental¹ |
| macOS | — | ✓ |

¹ The compiler emits Windows ARM64 executables, but they have not been run on
Windows ARM64 hardware and there is no seed for that host
([ADR 0968](docs/decisions/0968-aarch64-windows-target.md)).

² Linux ARM64 is tested under `qemu-aarch64` user emulation, not on hardware.

All targets are 64-bit and little-endian. x86-64 and ARM64 are developed in
strict lockstep ([ADR 0008](docs/decisions/0008-isa-priority-policy.md)). For
now, testing is local: the gate runs on macOS and Linux
([ADR 0842](docs/decisions/0842-mac-first-gate.md)), and the same suites must
pass on Windows x86-64 before every release
([ADR 0925](docs/decisions/0925-windows-release-gate.md)).

## Install

There is no installer: the repository is the toolchain. Clone it (a shallow
clone is much smaller) and run `./ail` from its root.

```sh
git clone --depth 1 https://github.com/kksimp/ail
cd ail
./ail run examples/hello.ail
```

On Windows (PowerShell 5.1 or later):

```powershell
git clone --depth 1 https://github.com/kksimp/ail
cd ail
.\ail.ps1 run examples\hello.ail
```

Put `ail` on your PATH, so a project's agent kit and MCP registration can run a bare `ail`
([ADR 1058](docs/decisions/1058-ail-on-path-and-doctor.md)). `ail doctor` checks the toolchain and
`ail doctor --path` prints the line to add to your shell profile; nothing edits your files for you.

```sh
mkdir -p ~/.local/bin && ln -s "$PWD/ail" ~/.local/bin/ail   # a symlink works: ail finds its root through it
ail doctor --path                                            # the PATH line for your profile, if ~/.local/bin is not on it
ail help                                                     # every command; `ail help CMD` for one
```

On Windows, add the clone's directory to `PATH` (`ail doctor --path` prints the PowerShell line) and use `ail.cmd`.

What the first run does:

1. Downloads the **seed** for your host (a native `ail` binary for macOS ARM64,
   Linux x86-64 and ARM64, Windows x86-64) with `curl`, or `Invoke-WebRequest`
   on Windows. It tries `https://seeds.voltare.us/ail/<tag>/` first and falls
   back to the GitHub release of the same tag. No account is needed.
2. Checks its SHA-256 against `tools/release/seeds.sha256` and refuses it on a
   mismatch, whichever source it came from. A copy that fails the check is
   discarded and the next source is tried. Who builds seeds and how to check one yourself:
   [ADR 0976](docs/decisions/0976-public-release-and-seed-trust.md).
3. Builds the current compiler from source with the seed, into `.native/`.
   Later runs rebuild only when `compiler/` or `std/` changes.

`ail run` keeps built programs in a cache (`AIL_CACHE_DIR`, else
`$XDG_CACHE_HOME/ail/build`, `~/.cache/ail/build`, or
`%LOCALAPPDATA%\ail\build` on Windows); delete it at any time. Nothing else
outside the repository is written. Needs: `bash`, `curl`, `git`, and
`shasum` or `sha256sum` (macOS and Linux have them); no C compiler, no
Python, no package manager. To remove AIL, delete the clone and the cache.

## Documentation

Documents are written for people and AI agents, in the usual order:

1. **Getting started:** [Install](#install), [AIL on Windows](docs/windows.md), [tutorial](docs/tutorial.md)
2. **Tour:** [How-to](docs/how-to.md), [quick reference](docs/quick-reference.md)
3. **Language reference:** [specification](docs/spec/README.md), [glossary](docs/glossary.md)
4. **Standard library:** the module index in the [quick reference](docs/quick-reference.md); `ail card std.MODULE` lists a module's functions, `ail ask std WORDS` finds one; rules in [spec chapter 17](docs/spec/17-std.md)
5. **Tooling:** [packages](docs/packages.md), [benchmarks](docs/benchmarks.md), [native build guide](docs/design/native-guide.md)
6. **For agents:** [agent benchmark](docs/agent-benchmark.md) (measured results), [For agents](docs/for-agents.md) (the loop, `ail check` output and exit codes, canonical patterns, the mistakes that cost the most), [language card](docs/language-card.md) (core card: `ail card`; everything in one file: [llms-full.txt](llms-full.txt))

Also: [documentation index](docs/README.md), [vision](docs/vision.md), [design decisions](docs/decisions/README.md), [open questions](docs/open-questions.md), [roadmap](docs/roadmap.md).

AI agents: start with [For agents](docs/for-agents.md), then [llms.txt](llms.txt) for the list of docs to load. To work on AIL itself, read [AGENTS.md](AGENTS.md).

### Agent kit for your project

`ail new NAME` writes an `AGENTS.md` and a `.mcp.json` into the new project (`--no-agent` skips them). In an existing project, run `ail new .`: it writes the same two files and never overwrites one (if `AGENTS.md` exists it prints the block to add). The kit is this, and nothing else (at most 475 tokens, [tool.new.3](docs/spec/22-tools.md)):

```
# AGENTS.md

AIL project: source is `.ail`, the CLI is `ail`. Use a tool when its line applies, not as a ritual.

- Read `ail card` once, all of it, before writing code. It names the spellings agents guess wrong (Habits from other languages) and the code that compiles but does not do what you meant ("Compiles, but is not what you meant"); no diagnostic reaches the second, so check it yourself.
- `ail check`, `build`, `run` and `test` rewrite some wrong spellings in your files and print `fixed ...`. Read the file again before you edit it.
- Then look up, never guess: `ail card TOPIC`, `ail ask std WORDS` (std and methods), `ail ask how WORDS`.
- A diagnostic `error[RULE]`: `ail check --fix FILE` applies the fixes it carries; `ail explain RULE` gives the meaning and a fix.
- Before editing a function you did not write: `ail context FILE FN --budget 800` (callees, callers, types, tests), not the whole file.
- Before changing a signature or a type: `ail ask FILE what calls FN` (also: who uses T). Never grep.
- Two candidate fixes: `ail try FILE` checks and tests each on a copy.
- A change across several functions in a program with `@test` functions: `ail task start FILE GOAL`, `ail task verify` (check, tests, what no test covers), `ail task finish` (writes `.ail/cert.json`).
- `ail task verify` runs only the check and the `@test` functions: run the project's own build and test commands as well.
- `ail fmt FILE` formats; `ail test FILE` runs the `@test` functions.
- `.mcp.json` registers `ail mcp`: the same tools as MCP calls (`ail_ask`, `ail_check`, `ail_context`, `task_start`, ...), kept warm between calls.
```

`.mcp.json` registers `ail mcp` on stdio, so the same tools are MCP calls.

## Built with AIL

First-party apps, each in its own repository (AGPL-3.0), written in AIL and pinned to an ail commit:

- [garlic](https://github.com/kksimp/garlic): a web browser
- [garlic-press](https://github.com/kksimp/garlic-press): a PDF viewer
- [ail-edit](https://github.com/kksimp/ail-edit): a code editor
- [clove](https://github.com/kksimp/clove): a compressor
- [Husk](https://github.com/kksimp/Husk): a disk analyzer for macOS
- [Gridrun](https://github.com/kksimp/Gridrun): a game
- [ail-kernel](https://github.com/kksimp/ail-kernel): a kernel proof of concept

## Contributing and security

- [CONTRIBUTING.md](CONTRIBUTING.md): how a change gets tested without hosted CI, and what a PR needs.
- [SECURITY.md](SECURITY.md): report vulnerabilities as GitHub issues with the Security report template (public while the project is small).

## License

Apache License 2.0 with a Runtime Library Exception. See [LICENSE](LICENSE):
the toolchain, its sources and docs are Apache-2.0; code the toolchain
embeds into the programs you compile carries no attribution requirement
(ADR 0510).
