# 0583. A native coverage tracker: which programs `ail build` handles, and why the rest fail

- **Status:** Accepted
- **Amended by:** [ADR 0906](0906-retire-native-coverage-tracker.md)
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** none (a measurement tool; native.subset.1 and native.build.1–3 are unchanged, see [23-native.md](../spec/23-native.md))
- **Builds on:** [ADR 0022](0022-staged-self-hosting.md) (the native backend is the path off Rust),
  [ADR 0070](0070-token-bench-tool.md) (Python for measurement only),
  [ADR 0085](0085-interim-local-testing-gate.md) (no hosted CI; the gate is local)

## Context

Getting AIL off the Rust interpreter needs the native backend to cover more of
the language. Every lane needs to know which constructs block the most
programs, and whether a native change moved the count. Today the only signal is
the first `native.subset.1` error of a single program.

## Decision

`tools/native-coverage/run.sh` runs `ail build` on every program in
`examples/*.ail`, `bootstrap/tests/cases/*.ail` and
`tools/perfbench/programs/*.ail`. For each it records one status, counts every
`native.subset.1` problem by construct, and writes
[docs/design/native-coverage.md](../design/native-coverage.md) and
`tools/native-coverage/results.json`. The classification and counting rules are
in [ADR 0584](0584-native-coverage-classification.md).

- The tracker is measurement only. It is not a merge gate and does not change
  the compiler. It runs in about 90 seconds on four workers.
- The script is Python 3 standard library, run with `python3 -I`, as ADR 0070
  allows for measurement tools.
- Lanes rerun it after a native PR merges. The run that writes the files prints
  the change since the last `results.json`.

## Rejected alternatives

- **A hosted dashboard.** There is no hosted CI and no hosted testing (ADR 0085).
  A dashboard would be a second source of truth that drifts from the repository.
- **Grepping the subset text in `docs/spec/23-native.md`.** The spec describes the
  subset; it does not say what the current compiler accepts. The tracker runs the
  real `ail build`, so the counts match what users see.
- **A `ail build --coverage` flag in the Rust bootstrap.** It would touch the
  bootstrap, which is outside lane 180's files, and it would be deleted when the
  native compiler replaces the Rust driver. A script outside the bootstrap lasts
  as long as the programs it measures.
- **Adding the tracker to `./ail selftest`.** A run of about 90 seconds would slow
  every selftest. It is a report, not a test; a failing native build is already
  reported by `bootstrap/tests/native.rs`.
