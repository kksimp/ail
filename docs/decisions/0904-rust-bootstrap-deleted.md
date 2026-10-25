# 0904. The Rust bootstrap is deleted (D5)

- **Status:** Accepted
- **Date:** 2026-10-25
- **Decided by:** Designer ("Yes delete bootstrap", 2026-10-25)
- **Topic:** 23 — Compiler architecture
- **Amends:** [ADR 0023](0023-rust-bootstrap.md), [ADR 0087](0087-private-rust-toolchain.md), [ADR 0740](0740-self-hosted-checker-parity-gate.md)

## Context

Step D of the Rust removal made every `./ail` command and every gate suite
native ([ADR 0900](0900-refuse-rust-only-routes.md)). Nothing routes to
`bootstrap/` any more: `ail` and `ail.ps1` refuse a command with no native form,
and `tools/gate/mac-gate.sh` runs one native run with no cargo. The native CLI is
built from the published seed ([ADR 0855](0855-retire-the-rust-bootstrap.md)).

## Decision

`bootstrap/` (the Rust front end, interpreter and tests) is deleted, along with
the cargo cache that had been committed under `bootstrap/target`. The frozen
goldens in `tests/golden/` stay: they were produced from the Rust binary and
now check the native tools with no Rust binary.

- ADR 0023: the bootstrap front end and interpreter no longer exist.
- ADR 0087: no Rust toolchain is installed or pinned by the repository.
- ADR 0740: the Rust checker is gone, so there is no parity gate. The AIL
  checker in `compiler/check/` is the only checker, and `tests/checker_known_new.txt`
  is history.

## Rejected alternatives

- **Keep the crate for differential testing.** The frozen goldens give the same
  coverage with no toolchain, and keeping it invites double work (ADR 0865).
