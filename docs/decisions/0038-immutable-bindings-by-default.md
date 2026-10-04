# 0038. Local bindings are immutable by default

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 4 — Type system; 7 — Ownership/lifetimes
- **Spec rules:** none yet
- **Scope:** **bindings** only (whether a name can be reassigned). Object and
  field mutability is decided in the ownership topic.

## Context

Immutable bindings make code easier to reason about for humans and AI, and
they supply facts the compiler needs for in-place reuse (ADR 0012) and for
concurrency safety (ADR 0025).

## Options considered

### Option A — Immutable by default; mutable bindings are marked (chosen)
- **Pros:** A reader can trust that a binding never changes after it's set.
  Mutable state stands out. Gives the optimiser more facts.
- **Cons:** About one token wherever mutation is needed.

### Option B — Mutable by default
- **Pros:** Familiar from C, Go and Python.
- **Cons:** Every binding might change, so readers must scan for
  reassignments. Fewer guarantees.

### Option C — No distinction; the compiler infers
- **Pros:** The fewest tokens.
- **Cons:** Intent is invisible, and accidental reassignment is never caught.

## Decision

A local binding cannot be reassigned unless it is declared mutable. The marker
syntax is set by [ADR 0039](0039-declaration-syntax.md).

## Consequences

- Reassigning an immutable binding is a compile error. The diagnostic offers
  the machine-applicable fix "declare it `mut`".
- The compiler should warn about mutable bindings that are never reassigned
  (fix: remove `mut`), so the marker stays meaningful.
- Under mutable value semantics ([ADR 0060](0060-mutable-value-semantics.md)),
  an immutable binding holds a deeply immutable value. Only `ref` cells
  provide shared mutation. This supports GC barrier elision
  ([performance-beyond-c.md §2.5](../design/performance-beyond-c.md#25-barrier-elision-through-immutability)).

## Rejected alternatives

- **Mutable by default:** weaker guarantees for readers and the optimiser.
- **Inferred:** hides intent, and accidental reassignment is never caught.
