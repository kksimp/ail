# 0865. One implementation, one checker, one gate: no double work

- **Status:** Accepted
- **Date:** 2026-10-23
- **Decided by:** Designer ("Cut out all double work", 2026-10-23)
- **Topic:** 23 — Compiler architecture
- **Amends:** [ADR 0740](0740-self-hosted-checker-parity-gate.md), [ADR 0842](0842-mac-first-gate.md)

## Context

While the Rust bootstrap is being retired, most work was done twice. New
features went into the interpreter and the native compiler, and checker
changes went into Rust and AIL. Each native test program was built up to three
times per gate, the integrator ran a Linux pass before the Mac gate, and lanes
regenerated the cards, which then conflicted at merge.

## Decision

1. **Native only.** New language and std features are implemented only in the
   native compiler and runtime. The interpreter and other Rust code get bug
   fixes only where diagnostics need them. Tests for new work are native run
   programs with a `.out` golden.
2. **One checker.** Checker changes go only into `compiler/check/`. The
   `self_hosted_check` parity test allows the AIL checker to report more than
   the Rust one (a known-new list), never less.
3. **One gate.** The Mac gate runs once per batch, with no per-batch Linux
   cargo pass. Within it, each native program is built once, by parallel
   processes running the native compiler, and is compared with a frozen
   golden, not a fresh interpreter run. Cargo tests that have AIL copies
   are dropped. The self-host stage runs on request.
4. **Generated files once.** The integrator regenerates the cards once per
   batch; lanes don't.
5. **One owner per task.** Lanes claim a task before building it, and own
   their files. A duplicate PR is closed.

## Rejected alternatives

- **Keep both paths until the switch-over.** This doubles the cost of every
  feature for the remaining weeks, and the interpreter is being deleted anyway.
- **Freeze the AIL checker until Rust is gone.** This would block diagnostics
  work that the native-only path needs.
