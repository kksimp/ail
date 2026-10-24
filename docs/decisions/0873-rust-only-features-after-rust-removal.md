# 0873. Rust-only features: native record/replay and deterministic scheduler; the bytecode tier goes

- **Status:** Accepted
- **Date:** 2026-10-24
- **Decided by:** Designer (2026-10-24, option 1 of 3)
- **Topic:** 23 — Implementation
- **Supersedes:** the bytecode tier: [ADR 0359](0359-bytecode-tier-in-the-bootstrap.md), [0361](0361-bytecode-tier-design.md), [0362](0362-differential-harness-for-the-bytecode-tier.md), [0363](0363-bytecode-tier-methods-and-data.md), [0364](0364-bytecode-tier-limits-coverage-and-traces.md), [0471](0471-watch-on-the-bytecode-tier.md), [0618](0618-bytecode-typed-fast-paths.md), [0619](0619-bytecode-calls-loops-and-fusion.md), [0620](0620-bytecode-element-method-calls.md), [0621](0621-bytecode-bounds-checks-stay.md), [0756](0756-bytecode-differential-fuzzing.md), [0759](0759-bytecode-tier-stalls-not-frames.md)
- **Amends:** [ADR 0868](0868-ai-first-tooling-after-rust-removal.md) item 4

## Context

Three features exist only in the Rust interpreter, which is being deleted
([plan](../roadmap.md)): the bytecode tier, record and replay
([ADR 0398](0398-record-and-replay.md)), and the deterministic task scheduler
([ADR 0186](0186-deterministic-task-scheduler.md), [ADR 0207](0207-deterministic-mode.md),
[ADR 0322](0322-random-schedule-testing.md)).

## Decision

1. **Record and replay** is ported to native code on the instrumentation layer of
   ADR 0868 item 4 (the same events that serve coverage and `--trace`).
2. **The deterministic scheduler** is ported to the native task runtime:
   `--deterministic` and random-schedule testing run tasks one at a time in a seeded order,
   as ADRs 0186, 0207 and 0322 specify. Only programs that ask for it pay for it.
3. **The bytecode tier is dropped** with the interpreter. It existed to speed up the
   interpreter; native code replaces it, so its ADRs are superseded and its spec rules leave
   with it.

Rust deletion (step D5) waits for items 1 and 2.

## Rejected alternatives

- **Drop all three, rebuild later:** deletes Rust soonest, but leaves agents without replay
  and tests without deterministic schedules for an unknown time.
- **Port all three:** the bytecode tier would be a second, slower execution engine beside
  native code, which is double work with no user benefit.
