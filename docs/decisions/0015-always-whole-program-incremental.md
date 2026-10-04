# 0015. Always optimise the whole program, made fast by fine-grained incremental caching

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 23 — Compiler architecture; 25 — Optimization; 27 — Build system
- **Spec rules:** none
- **Resolves:** [OQ-017](../open-questions.md)

## Context

Whole-program optimisation (WPO) enables specialisation, devirtualisation and
cross-module constant propagation, which are key to beating C
([performance-beyond-c.md §3.4](../design/performance-beyond-c.md#34-whole-program-optimisation-by-default)).
The usual price is a slow edit-compile loop. The designer leaned toward
"always WPO" over "WPO in release only".

## Options considered

### Option A — Always WPO, with fine-grained incremental caching (chosen)
- **Pros:** One mode, so debug performance equals release. A fast edit loop
  when the cache works. Matches the R&D-over-feature-loss policy (ADR 0009).
- **Cons:** R&D-heavy. Edits to widely-inlined code, or changes to the set of
  interface implementers, cascade invalidations. The cache must be
  bit-for-bit deterministic.

### Option B — Always WPO, no special incremental work
- **Pros:** Simplest.
- **Cons:** The edit loop slows as projects grow, which hurts AI agents most
  because they compile very often.

### Option C — WPO in release, incremental in debug
- **Pros:** A proven approach.
- **Cons:** Performance differs between modes, so performance bugs appear
  only in release.

## Decision

There is a single optimisation mode: every build optimises the whole program.
The compiler records the dependencies of every optimisation decision (e.g.
"inlined `g` into `f`", "devirtualised call because `T` has one implementer")
and caches the results per function. After an edit, it recomputes only the
decisions whose inputs changed.

## Rationale

Debug performance equal to release removes a whole class of "only slow or
broken in release" problems. Fast incremental results keep the edit loop
usable for humans and AI agents.

## Consequences

- The compiler must be built around a **dependency-tracked query/cache
  architecture** from Stage 2 onward. This is an input to the compiler
  architecture topic.
- Optimisation decisions become first-class recorded facts, which also feeds
  the performance-transparency reports and the AI API ("what was inlined and
  why?").
- The cache must be deterministic and reproducible, and safe to share between
  machines (an input to the build system topic).
- Cascade-heavy edits are a known weak spot. The compiler should report *why*
  a rebuild was large.
- Budget controls (e.g. capping optimisation work per edit) may be needed and
  will be raised as questions when the build system is designed.

## Rejected alternatives

- **WPO without incremental caching:** the edit loop doesn't scale.
- **Release-only WPO:** gives different performance in each mode.
