# 0027. All five Tier 1 targets gate every merge

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 28 — Testing strategy; 24 — Cross-platform
- **Spec rules:** none
- **Resolves:** OQ-004

## Context

The founding principle is that Linux, Windows and macOS are all first-class.
ADR 0008 requires strict ISA lockstep. A platform that isn't tested on every
change drifts.

## Options considered

### Option A — All 5 targets gate every merge (chosen)
- **Pros:** No platform drifts into second-class. Cheap while the compiler is small.
- **Cons:** Slower and more costly CI. Windows ARM64 runners are scarcer.

### Option B — 4 gate, windows-arm64 nightly
- **Pros:** Faster.
- **Cons:** The smallest platform can break for a day and become de facto Tier 2.

### Option C — Only Linux gates
- **Pros:** Fastest.
- **Cons:** macOS and Windows become ports.

## Decision

`linux-x64`, `linux-arm64`, `macos-arm64`, `windows-x64` and `windows-arm64`
must all pass the required test suite (build, tests, conformance,
differential testing, and benchmarks with regression thresholds) before any
change merges to `main`.

## Consequences

- CI infrastructure must provide runners for all five targets, including
  native Windows ARM64 and macOS ARM64.
- Long-running jobs (extended fuzzing, full benchmark sweeps) may run on a
  schedule, but must cover all five targets equally.
- **Process note:** during Stage 1 (specification only) there is no code to
  gate. The gate takes effect from the first code in Stage 2.

## Rejected alternatives

- **Windows ARM64 nightly:** lets the smallest platform drift.
- **Linux-only gating:** contradicts first-class OS support.
