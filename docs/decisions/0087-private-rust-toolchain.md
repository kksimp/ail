# 0087. The bootstrap uses a pinned Rust toolchain installed privately inside the repository

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 23 — Compiler architecture (bootstrap); project tooling

## Context

The bootstrap front end and interpreter are written in Rust (ADR 0023).
ADR 0071 requires everything a person runs to set itself up without
system-wide changes.

## Options considered

- **Private copy inside the repository (chosen):** nothing changes outside the
  project folder, everyone uses the same version, and deleting the folder
  removes it. Con: about 1–1.5 GB of disk, and editors don't find it automatically.
- **Standard per-user rustup install:** the usual setup, but it changes the
  home folder and shell configuration.
- **Manual install:** a manual step, which goes against ADR 0071.

## Decision

1. The Rust version is pinned in the repository (`rust-toolchain.toml`).
2. The `ail` run script uses an existing Rust installation if one matches
   the pin. Otherwise it installs that pinned toolchain into a
   git-ignored directory inside the repository and uses only that.
3. No files outside the repository are created or modified.

## Consequences

- Takes effect when Stage 2 code starts (ADR 0086).
- The first run downloads the toolchain (a few minutes, depending on the network).

## Rejected alternatives

- **Per-user install:** touches the designer's system outside the project.
- **Manual install:** violates ADR 0071.
