# 0948. `ail profile`: a sampling profiler with JSON output

- **Status:** Proposed
- **Date:** 2026-11-05
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 26 — Debugging/profiling
- **Spec rules:** none yet
- **Builds on:** [ADR 0641](0641-native-trap-stack-from-frame-pointers.md), [ADR 0947](0947-native-dwarf-debug-info.md)

## Decision

A timer signal (`SIGPROF` via `setitimer`; per-thread `timer_create` for tasks)
whose handler walks frame pointers into a preallocated per-thread buffer, with
no allocation. The runtime (`rt/native/prof.ail`) writes the raw code addresses to standard
error at exit; `tools/profile/profile.py` resolves them with `addr2line` over
the DWARF of ADR 0947. `AIL_PROFILE=1` enables it in any Linux binary;
`ail profile FILE` builds, runs and prints the result.
Output is JSON first (self and inclusive samples per function, hot lines,
callers), then a text table. Linux needs a `sigreturn` restorer kernel in both
backends; macOS uses libSystem `sigaction`.

## Rejected alternatives

- Instrumenting function entries: costs the fast path.
- Shelling out to `perf` or `sample`: not on every OS, no AIL names, no JSON.

## Consequences

Written, not yet run on a machine (2026-11-06); this ADR stays Proposed until
it is. The resolver is Python for now (tooling only, not in user programs);
an AIL resolver replaces it once the ELF reader exists. macOS and Windows are
not done. The handler uses `SIGPROF`, so a program that blocks it gets no
samples.
