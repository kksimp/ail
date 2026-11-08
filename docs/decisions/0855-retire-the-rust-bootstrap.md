# 0855. Retire the Rust bootstrap: `ail` becomes a native AIL program, seeded from private release binaries

- **Status:** Accepted
- **Date:** 2026-10-22
- **Decided by:** Designer (Kaleb, 2026-10-22: full removal; seeds as releases on the private repository); detail by AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** native.build.3, native.build.5
- **Amends:** [ADR 0023](0023-rust-bootstrap.md) (Rust bootstrap), [ADR 0087](0087-private-rust-toolchain.md) (private Rust toolchain)

## Context

The native compiler builds itself up to the fixed point (ADR 0610), so the Rust
front end, interpreter and checker are the last Rust in the project. Keeping
them costs every change a second implementation (ADR 0740) and a Rust toolchain.
There is no AIL interpreter port (ADR 0359 and later decisions stand): after
removal every program, including the tools, runs natively.

## Decision

1. `ail` is the program `compiler/main.ail` (with `compiler/cli.ail`): the
   command line grows there, one command per change. `ail run FILE` builds the
   program into the build cache (`AIL_CACHE_DIR`, else `$XDG_CACHE_HOME/ail/build`,
   `%LOCALAPPDATA%\ail\build`, `~/.cache/ail/build`; 64 entries kept, native.build.5)
   and starts it. The key hashes the target, the path, and the path and SHA-256 of every `.ail` file of the backend, the program and the AIL std.
2. `ail prepare FILE` is `run` without the start and prints the executable's path.
   `./ail` execs that path, so the program keeps the terminal; the native std has
   no process-replacing call yet, and `ail run` run directly relays the output at exit.
3. The native CLI is the default for `./ail` (item 252): `run [--timeout MS] [--max-memory MB] FILE ARGS`,
   `check`, `fix`, `test`, `api` (outline, `--text`, `--pretty`, `--symbols`), `build` and `fmt`; `selftest` with no argument is the native gate (`tools/gate/mac-gate.sh`; `AIL_SELFTEST=cargo` keeps the cargo tests). `AIL_RUST=1` or a pinned `AIL_BINARY`
   is the one way to the Rust `ail` and its interpreter (step B removed `AIL_NATIVE_CLI` and `AIL_INTERP`). A program outside the native subset (`native.subset.1`) is an error, not a fallback.
   Commands and options the native CLI lacks (`run` with `--record --replay --max-steps --parallel`; `diff`, `--deterministic --seed --deny --unsafe` are native) still go to
   the Rust `ail`; each later change moves one and the Rust code is deleted only after a whole chain is green without it.
4. The first native `ail` comes from a **seed**: a release asset of the private
   repository (`ail-seed-ARCH-OS`; first release `seed-2026.10.22`), built by `tools/release/seed.sh` on the
   designer's Mac (macOS, Linux x86-64 and ARM64, Windows x86-64) after the fixed point holds. `tools/release/fetch-seed.sh`
   downloads it with `gh release download` (the signed-in `gh`) or `GITHUB_TOKEN`,
   checks its SHA-256 against `tools/release/seeds.sha256` (committed), and
   installs it as `.native/seed`, which builds `compiler/main.ail` into `.native/ail`. Nothing is published by an agent or automatically.
5. Until the Rust `check` is ported, `./ail run` under the switch runs the Rust
   `check` first; with no Rust toolchain it relies on the native backend's own
   diagnostics.

## Rejected alternatives

- **Seed binaries checked into git.** Megabytes per target per update, forever, in history.
- **A public release.** The project is private for now; a private release needs no new policy. (A public-release policy is proposed in [ADR 0976](0976-public-release-and-seed-trust.md).)
- **Trusting the download without a hash.** A swapped asset would run as the compiler.
- **Porting the interpreter to AIL.** Already declined; native-only keeps one execution model.
- **Deleting Rust first.** The native subset does not yet cover every tool and test program (RR-3).

## Consequences

- The native `run` has no stdin passthrough when run directly (relay at exit); the script path has it.
- Every run hashes the backend's and std's sources (a few MB); `fs.metadata` is not native yet, which a size-and-time key would need. Trimming the cache uses `ls -t`, so on Windows the cache is not trimmed.
- The seed's hashes are pinned in `tools/release/seeds.sha256`; the Linux x86-64 seed was run on Linux and builds `examples/hello.ail`.
