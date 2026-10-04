# 0063. Shared mutable identity is expressed with a use-site `ref T`

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 7 — Ownership; 6 — GC; 11 — Concurrency
- **Spec rules:** [own.ref.1–2](../spec/07-ownership.md)

## Options considered

- **Use-site `ref T` (chosen):** one kind of type, sharing visible where it
  happens, and data-race freedom kept. Con: identity-bearing types rely on
  convention to always be used via `ref`.
- **Declaration-site `class` types:** two kinds of types, and `b := a`
  changes meaning based on a distant declaration.
- **Indices and handles only:** fast, but awkward, and stale IDs become a bug class.
- **`ref` plus handles:** flexible, but two ways to model sharing.

## Decision

1. `ref T` is the type of a GC-managed shared cell holding a `T`. Copying a
   `ref` value copies the reference. Every copy refers to the same cell, and
   a mutation through one is visible through all.
2. A `ref` value is confined to the task that created it, unless it is
   wrapped in a synchronised type (to be designed under concurrency).
   This preserves ADR 0025.

```
struct Node
    value int
    children List[ref Node]
```

## Consequences

- `ref` cells are the primary residents of the shared GC heap. The allocation
  ladder (ADR 0012) can still place non-escaping `ref` cells elsewhere.
- Cycles are possible only through `ref`, so the cycle collector or tracing
  GC only needs to consider `ref` cells.
- Open: construction syntax (`ref Node(...)` is illustrative), how fields are
  accessed and mutated through a `ref`, and whether to add standard-library
  handle/arena types later (not excluded).

## Rejected alternatives

- **Class types:** break local reasoning.
- **Handles only:** too restrictive as the sole mechanism.
