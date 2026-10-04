# 0030. Performance benchmarks: standard suites plus automated head-to-head "drag races"

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 28 — Testing strategy; 25 — Optimization
- **Spec rules:** none
- **Resolves:** OQ-008 (initial scope; the three-bar suite is deferred, not rejected)

## Context

Philosophy #4 says performance must be measurable. The architect proposed a
comprehensive "three-bar" suite (typical C, C with the best libraries, expert
C) on day one. The designer chose a lighter starting point: standard suites
plus real programs, and added an idea of their own, small paired programs
"drag raced" automatically.

## Options considered

### Option A — Three-bar suite on all targets (architect's recommendation)
- **Pros:** The most complete and honest picture.
- **Cons:** Big infrastructure and hardware cost up front.

### Option B — Standard benchmark suites (chosen, in combination)
- **Pros:** Comparable with published numbers for other languages.
- **Cons:** Heavy on microbenchmarks, and implementations are often very tuned.

### Option C — Real applications (chosen, in combination)
- **Pros:** Measures what users experience.
- **Cons:** Few data points, and harder to attribute differences to causes.

### Designer's addition — Automated drag races
Small programs implemented in both AIL and C, run head to head
programmatically.
- **Pros:** Cheap to add more. Each race isolates one workload, which makes
  attribution easier. Fully automatable, so it fits per-merge CI.
- **Cons:** Equivalence between the two implementations must be checked
  carefully. The C side's coding style (typical vs. expert) affects results
  (OQ-025).

## Decision

The initial performance benchmark consists of:

1. **Standard suites:** established benchmark programs ported to AIL.
2. **Real applications:** a few small real programs (e.g. an HTTP server, a
   JSON processor, a compiler pass) in AIL and C.
3. **Drag races:** a growing set of small paired AIL/C programs. A harness
   builds both, checks that their outputs are identical, runs them
   repeatedly, and reports time, memory, binary size and compile time
   with statistical confidence.

All of this runs automatically on all five targets (ADR 0027), and results
are published. The comprehensive three-bar suite may be revisited later.

## Consequences

- The drag-race harness is a Stage 4 deliverable (the first native binaries).
  The AIL side can be written earlier and run on the interpreter for
  correctness only.
- C comparison binaries are built with clang and gcc at -O2 and -O3 (ADR 0010
  allows this for comparison).
- Each race states which kind of C it is compared against (see OQ-025).
- Regression thresholds for CI gating will be set when the harness exists.

## Rejected alternatives

- None rejected outright. The three-bar suite is **deferred**.
