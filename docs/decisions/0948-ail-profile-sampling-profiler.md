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
no allocation. Samples map through `trace.tab` to function, file and line.
`AIL_PROFILE=FILE` enables it in any binary; `ail profile PROG` wraps it.
Output is JSON first (self and inclusive samples per function, hot lines,
callers), then a text table. Linux needs a `sigreturn` restorer kernel in both
backends; macOS uses libSystem `sigaction`.

## Rejected alternatives

- Instrumenting function entries: costs the fast path.
- Shelling out to `perf` or `sample`: not on every OS, no AIL names, no JSON.

## Consequences

Not implemented yet; this ADR stays Proposed until the runtime stage lands.
