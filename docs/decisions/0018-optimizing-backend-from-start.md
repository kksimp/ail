# 0018. Build an optimising native backend from the start

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 23 — Compiler architecture; 25 — Optimization
- **Spec rules:** none
- **Resolves:** [OQ-013](../open-questions.md)

## Context

With no LLVM (ADR 0010) and no C emission (ADR 0017), AIL's first native
backend is its own. The architect recommended building a simple one first and
optimising later. The designer chose to build the optimising backend directly.

## Options considered

### Option A — Optimising backend from the start (chosen)
SSA-based IR, a sophisticated register allocator, and real instruction
selection from day one.
- **Pros:** No throwaway work. Good code as soon as it lands. The
  architecture is designed for optimisation (and for the observability
  required by ADR 0016) from the beginning.
- **Cons:** A much later first native binary. Harder to debug a complex
  backend without a simple baseline to compare against.

### Option B — Simple first, then optimise (architect's recommendation)
- **Pros:** The earliest binaries, and every optimisation can be measured.
- **Cons:** Some throwaway work. The designer rejected it.

### Option C — Two permanent backends (fast and optimising)
- **Pros:** Very fast debug compiles.
- **Cons:** Double maintenance, and conflicts with single-mode WPO (ADR 0015).

## Decision

AIL's first native backend is a full optimising backend for both ISAs.

## Rationale

Consistent with single-mode WPO (ADR 0015) and full observability (ADR 0016):
there is only one backend and one mode, and it is designed for optimisation
from the start.

## Consequences

- **Correctness risk is the main cost.** Without a simple backend to compare
  against, AIL needs other correctness oracles. Proposed mitigations, to be
  confirmed as questions:
  - **Differential testing against the Stage 3 interpreter** (the semantic
    reference): every program in the conformance suite and the fuzzer must
    produce identical results interpreted and compiled.
  - **Pass-level validation:** IR verifiers after every pass, and
    translation validation research for critical passes.
  - **Optimisation bisection:** the ability to disable individual passes or
    decisions to isolate miscompilations (this also helps the AI API).
- Roadmap Stages 4, 6 and 7 merge into a single "optimising native backend"
  track. The roadmap has been updated.
- The time to first native binary grows. Performance results are published
  from the first binary, as before.

## Rejected alternatives

- **Simple first:** throwaway work, and the designer prefers one architecture.
- **Two backends:** contradicts ADR 0015.
