# 0897. `ail check --perf`: speed hints that share the optimizer's facts

- **Status:** Deferred past 0.1.0
- **Date:** 2026-10-24
- **Decided by:** Designer (2026-10-24, chose the feature on a decision card); the details by AI (delegated, ADR 0076), for integrator review
- **Builds on:** [ADR 0874](0874-purity-driven-speed-optimizations.md), [ADR 0845](0845-vectorization-order-and-idioms.md), [ADR 0848](0848-idle-values-leave-registers-in-loops.md)
- **Topic:** 20 — AI semantic API

## Context

The optimizer knows why a loop is slow (an allocation each pass, a call it could not prove pure, a loop it could not vectorize), but the author never hears it.

## Decision

- `ail check --perf` adds `perf.*` notes to `ail check`, never errors. Each names the line, the fact, and a fix when one is mechanical.
- First notes: allocation inside a loop that could be hoisted; an impure call that blocks CSE or memoization, with the effect that makes it impure; a loop kept scalar and the reason (aliasing, a call, a data dependence); a copy of a large value that a move would avoid.
- The notes come from the MIR passes' own refusal reasons (ADR 0874): a pass records why it declined, and `--perf` prints the record. No separate analysis, so the hints cannot disagree with the compiler.
- Planned rule keys: `perf.alloc.*`, `perf.pure.*`, `perf.vec.*`.

Cost when unused (ADR 0868): none. The feature is a separate command or pass, so a program that does not use it gets no extra code, no runtime check and no slower build.

## Consequences

MIR passes gain a refusal record that is built only when this flag asks, so the normal build is unchanged. A pass that is changed in the optimizer lane must keep the record in step.

## Rejected alternatives

- **A separate profiler:** needs a run and tells where, not why.
- **Warnings on by default:** noise for code that does not care; the flag keeps `ail check` quiet.
