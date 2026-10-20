# 0221. `./ail` runs the built binary directly when it is up to date

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling, [ADR 0087](0087-private-rust-toolchain.md)
- **Spec rules:** none

## Context

Every `./ail` command first looked for a Rust toolchain (`rustc --version`) and
then ran `cargo build`, whose own freshness check takes about 0.1 s. On a
10 ms program that was ten times the work, and timing through the wrapper hid
the interpreter's speed.

## Options considered

### Option A — skip both when nothing is newer than the binary (chosen)
- **Pros:** `./ail run examples/hello.ail` takes 13 ms instead of about 140 ms;
  no change for a build that is needed.
- **Cons:** relies on modification times, as cargo does. A source file restored
  with an old modification time (for example by an archive extractor) would not
  trigger a rebuild; `./ail selftest` always builds.

### Option B — leave as is
- **Pros:** cargo decides.
- **Cons:** the 0.1 s on every command.

## Decision

`./ail` execs `bootstrap/target/release/ail` directly when it exists and no file
under `bootstrap/src`, nor `Cargo.toml`, `Cargo.lock`, `build.rs` or
`rust-toolchain.toml`, is newer than it. Otherwise it behaves as before.

## Rationale

ADR 0076 criterion 4 (speed) for every command; the tool's behaviour is
unchanged.

## Consequences

- The measurement note in [interpreter-performance.md](../design/interpreter-performance.md)
  can time `ail run` through the wrapper again, though `tools/perfbench/run.sh`
  still times the binary directly.

## Rejected alternatives

- **Hash the sources:** slower than the check it saves.
