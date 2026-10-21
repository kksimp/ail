# 0728. Where a native program gets its limits

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** run.limit.4, run.limit.5
- **Builds on:** [ADR 0726](0726-native-memory-limit.md), [ADR 0727](0727-native-timeout.md), [ADR 0655](0655-build-cache.md)

## Decision

An executable reads `AIL_TIMEOUT_MS` and `AIL_MAX_MEMORY_MB` at start. The
environment wins over defaults that `ail build --timeout MS --max-memory MB`
bakes into the runtime source (`limits.ail`) before compiling. An invalid
value ends the program with exit 2 (run.limit.4). The build cache key includes
the baked values, so two builds of one program with different limits never
share an entry. `--max-steps` is not offered natively: a step count needs
lowering support.

## Rejected alternatives

- **Command-line flags on the program** (`./prog --timeout 5`). They would
  clash with the program's own arguments.
- **A file next to the executable.** Another thing to lose or forget.
