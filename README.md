# AIL

**AI-First Language**, by Voltare. Pronounced "ale".

AIL is a native, memory-safe, garbage-collected systems programming language
designed from the ground up for humans **and** AI agents. It aims for C-class
performance, maximum semantic density per token, canonical formatting, and a
compiler that answers structured questions about your program.

```
main.ail      ail build · ail run · ail test · ail fmt · ail check
```

> **Status:** Stage 2, native self-hosting. The Rust bootstrap is deleted
> ([ADR 0904](docs/decisions/0904-rust-bootstrap-deleted.md)); `ail` builds itself.
> See [docs/status.md](docs/status.md).

**What to expect.** AIL is pre-1.0: the language and std still change, and spec
rule IDs are stable while the behaviour behind them may not be. The compiler,
the std and the tools are written in AIL by AI agents under the direction of a
human designer, who makes the serious design calls; every test gate runs
on the designer's own machines (there is no hosted CI). What is implemented is listed,
with its tests, in [docs/status.md](docs/status.md); experimental or not yet
built: Windows ARM64, GPU compute (`std.gfx`), NPUs and `std.audio`. The TLS and
cryptography are AIL's own and have had **no third-party audit**
([SECURITY.md](SECURITY.md)).

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
native `ail` with it into `.native/`. Nothing outside the repository is changed. On Windows use `.\ail.ps1 run examples\hello.ail`
(not yet tested on Windows).

## Targets

Linux, Windows, and macOS are all first-class.

| | x86-64 (v3) | ARM64 (v8.2-A) |
|---|---|---|
| Linux | ✓ | ✓ |
| Windows | ✓ | ✓ |
| macOS | — | ✓ |

All targets are 64-bit and little-endian. x86-64 and ARM64 are developed in
strict lockstep ([ADR 0008](docs/decisions/0008-isa-priority-policy.md)). For
now, testing is local on Linux and macOS
([ADR 0842](docs/decisions/0842-mac-first-gate.md)).

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

What the first run does:

1. Downloads the **seed** for your host (a native `ail` binary published as a
   GitHub release asset; macOS ARM64, Linux x86-64 and ARM64, Windows x86-64)
   with `curl`, or `Invoke-WebRequest` on Windows. No account is needed.
2. Checks its SHA-256 against `tools/release/seeds.sha256` and refuses it on a
   mismatch. Who builds seeds and how to check one yourself:
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

- [Documentation index](docs/README.md)
- [Vision and philosophy](docs/vision.md)
- [Design decisions](docs/decisions/README.md)
- [Language card](docs/language-card.md): the whole language in one short read
- [Language specification](docs/spec/README.md)
- [Open questions](docs/open-questions.md)
- [Roadmap](docs/roadmap.md)
- [Feasibility: token efficiency and performance estimates](docs/design/feasibility.md)
- [Research: beating C on systems and allocation-heavy code](docs/design/performance-beyond-c.md)

AI agents: read [AGENTS.md](AGENTS.md) first.

## Contributing and security

- [CONTRIBUTING.md](CONTRIBUTING.md): how a change gets tested without hosted CI, and what a PR needs.
- [SECURITY.md](SECURITY.md): report vulnerabilities privately through GitHub.

## Community token

An AIL community token exists on pump.fun: [HFwDPcrXvYLNTyNbv4CQuqx3rH54f4AHNRn1n56Npump](https://pump.fun/coin/HFwDPcrXvYLNTyNbv4CQuqx3rH54f4AHNRn1n56Npump).

It is not an investment and gives no rights in the project. The address above is the only official one; treat any other as an impersonation.

## License

Apache License 2.0 with a Runtime Library Exception. See [LICENSE](LICENSE):
the toolchain, its sources and docs are Apache-2.0; code the toolchain
embeds into the programs you compile carries no attribution requirement
(ADR 0510).
