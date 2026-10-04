# 0013. Data layout is compiler-chosen by default, with explicit fixed layout on request

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 4 — Type system; 5 — Memory model; 14 — FFI; 25 — Optimization
- **Spec rules:** none yet. To be specified in the Types and FFI chapters.
- **Research:** [design/performance-beyond-c.md §3.1](../design/performance-beyond-c.md#31-compiler-chosen-data-layout)
- **Resolves:** OQ-015

## Context

C fixes struct layout to declaration order, which stops its compiler from
fixing bad layouts. The founding brief asks for both aggressive optimisation
and "predictable data layouts". These conflict unless predictability is
defined as *visible and controllable* rather than *always declaration order*.

## Options considered

### Option A — Compiler-chosen by default, opt-in fixed layout, always reported (chosen)
- **Pros:** Allows field reordering, hot/cold splitting, and AoS→SoA
  transformation, which C can't do. Layout remains predictable through
  reports and the fixed-layout opt-in.
- **Cons:** Layout can't be seen from the source alone. FFI and serialization
  types need annotations.

### Option B — Declaration order by default; optimisations opt-in
- **Pros:** Layout is predictable from the source, and FFI is simple.
- **Cons:** Leaves a whole class of speedups unused.

## Decision

Unless a type explicitly requests a fixed layout, the compiler may choose its
field order, splitting and representation. The chosen layout is **always
reported** and queryable. Types that request a fixed layout (exact syntax to
be designed) get a defined layout suitable for C interop, serialization and
hardware access.

## Rationale

This reconciles the brief's two goals: predictability through transparency
and control, and performance through compiler freedom.

## Consequences

- The brief's "predictable data layouts" is interpreted as *reported and
  controllable*. Recorded in [vision.md](../vision.md).
- FFI types crossing into C must use fixed layout. The compiler should
  diagnose a missing annotation at the FFI boundary.
- Layout must be deterministic for a given program and compiler version
  (reproducible builds).
- Open design questions: whether layout may differ between debug and release,
  whether layout decisions may use PGO data, and how unsafe code that takes
  field offsets interacts with all this.

## Rejected alternatives

- **Declaration order by default:** gives up performance that only AIL's
  semantics allow.
