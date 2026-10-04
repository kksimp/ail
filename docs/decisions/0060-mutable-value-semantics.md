# 0060. AIL uses mutable value semantics

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 7 — Ownership/lifetimes; 5 — Memory model; 4 — Type system
- **Spec rules:** [own.val.1–2](../spec/07-ownership.md)
- **Background:** [design/ownership-options.md](../design/ownership-options.md)

## Context

How `b := a` behaves decides aliasing, data-race freedom, optimizer freedom,
lifetimes and GC pressure. Three models were compared: mutable value
semantics, reference semantics with GC, and Rust-style ownership with lifetimes.

## Options considered

### Option A — Mutable value semantics (chosen)
- **Pros:** Local reasoning. The optimizer can assume no aliasing everywhere
  (`restrict` for free). Data-race freedom by construction. Most data stays
  off the GC heap. No lifetime annotations. Measured 20–55% fewer tokens
  than Rust on the examples.
- **Cons:** Shared mutable structures need an explicit reference type. Less
  prior art at scale (Swift structs, Hylo).

### Option B — Reference semantics with GC
- **Pros:** Familiar, and object graphs are trivial.
- **Cons:** Aliasing everywhere, data-race freedom needs heavy extra typing,
  and more GC objects.

### Option C — Rust-style ownership with lifetimes
- **Pros:** The most expressive zero-copy references.
- **Cons:** Lifetimes, complex errors, and the most tokens.

## Decision

Binding, assignment, passing a non-`mut` argument, and returning all produce
values that are **observably independent**. A mutation made through one name
is never visible through another, except through an explicit shared reference
`ref` ([ADR 0063](0063-use-site-ref.md)). Implementations may share storage
between values (e.g. copy-on-write) as long as this is unobservable.

## Rationale

This is the model that best serves AI local reasoning, optimizer freedom
(the "beat C" strategy), data-race freedom (ADR 0025), and the allocation
ladder (ADR 0012) all at once.

## Consequences

- [ADR 0038](0038-immutable-bindings-by-default.md)'s open question is
  answered: an immutable binding holds an immutable **value**, deeply.
  Nothing reachable from it can change, except through a `ref`.
- **Exclusivity** is required so that a `mut` argument can't alias another
  argument (e.g. `f(mut x, x)`). The rule for such calls is OQ-032.
- Storage sharing plus uniqueness at last use enables in-place reuse
  (ADR 0012 track 3), and needs reclamation for shared buffers (the GC
  design picks between reference counting, tracing, or both).
- Performance transparency: the report must show where copy-on-write
  actually copies (OQ-033).
- Resource types (files, sockets, locks) need deterministic cleanup despite
  the GC (OQ-034).

## Rejected alternatives

- **Reference semantics:** pervasive aliasing works against AI reasoning,
  optimisation and race freedom.
- **Rust-style lifetimes:** complexity and token cost.
