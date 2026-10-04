# 0016. Debug builds are fully optimised and fully observable

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 26 — Debugging/profiling; 23 — Compiler architecture
- **Spec rules:** none

## Context

Following [ADR 0015](0015-always-whole-program-incremental.md), every build is
fully optimised. In optimised C/C++, debuggers often report variables as
"optimized out" and step through code in confusing orders.

## Options considered

### Option A — Full optimisation plus full observability (chosen)
The compiler emits rich debug information: where each source variable lives
at each point, or how to *recompute* it. Every variable stays observable.
Where that is truly impossible, the debugger explains why.
- **Pros:** You debug the code you ship, at full speed.
- **Cons:** A hard R&D problem. Debug info gets large. A few
  value-destroying optimisations may need to keep recovery information.

### Option B — Debug builds limit optimisation
- **Pros:** A perfect debugging experience, and proven.
- **Cons:** The debugged code isn't the shipped code, which conflicts with
  ADR 0015.

### Option C — No special handling
- **Pros:** No extra work.
- **Cons:** A poor experience. Contradicts the brief's "debugging is part of
  the language" goal.

## Decision

Optimisations must preserve **recoverability** of source-level values: every
optimisation pass must update debug metadata describing where each value is,
or how to reconstruct it. If a value is unrecoverable at some point, the
compiler must record the reason, and the debugger and AI API must surface it.

## Rationale

This is the only option consistent with single-mode WPO, and it serves both
humans and AI ("why can't I see this variable?" gets a structured answer).

## Consequences

- Recoverability is a **correctness requirement for every optimisation
  pass**, not an afterthought. It must be part of the IR design.
- AIL's own debug-info format must be able to express "recompute this value
  from X and Y". DWARF can partly express this (location expressions); PDB is
  more limited. See OQ-019.
- There may be a measurable cost in debug-info size. It is reported, not hidden.

## Rejected alternatives

- **Limiting optimisation in debug builds:** contradicts single-mode WPO.
- **No handling:** contradicts the brief's debugging goals.
