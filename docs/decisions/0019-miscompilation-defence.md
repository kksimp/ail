# 0019. Four-layer defence against miscompilation

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 28 — Testing strategy; 23 — Compiler architecture
- **Spec rules:** none
- **Resolves:** [OQ-020](../open-questions.md)

## Context

[ADR 0018](0018-optimizing-backend-from-start.md) removes the simple baseline
backend that would normally serve as a correctness reference. Miscompilation
(the compiler silently producing wrong code) is the most expensive class of
compiler bug.

## Options considered

### Option A — All four layers (chosen)
1. **Differential testing:** every conformance test and every fuzzer-generated
   program must produce identical observable results in the interpreter (the
   semantic reference) and in native code.
2. **IR verifiers** run after every optimisation pass.
3. **Translation validation** (research track): machine-check that a pass
   preserved semantics for a given input.
4. **Bisection:** any individual pass or optimisation decision can be
   disabled, so the decision that introduced a bug can be found automatically.
- **Pros:** Defence in depth. Layer 4 also powers the AI API ("which decision
  changed this behaviour?").
- **Cons:** A large test-infrastructure investment. Layer 3 is research-grade.

### Option B — Differential testing + IR verifiers only
- **Pros:** Cheaper.
- **Cons:** Subtle bugs slip through, and isolating them is slower.

## Decision

All four layers are required parts of the compiler's architecture and test
strategy.

## Consequences

- The Stage 3 interpreter must be deterministic and must define the
  "observable results" used for comparison (output, exit status, error values,
  and so on).
- A program fuzzer that generates well-typed AIL programs is a required tool.
- Optimisation decisions must be individually identifiable and switchable.
  This fits ADR 0015's recorded decisions.
- Translation validation starts with the highest-risk passes (e.g. memory
  optimisations and allocation-ladder placement).

## Rejected alternatives

- **Two layers only:** too thin without a baseline backend.
