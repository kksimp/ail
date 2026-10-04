# 0102. Unsafe operations require `unsafe` blocks; packages must opt in to unsafe

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 15 — Unsafe semantics
- **Spec rules:** [unsafe.block.1–4](../spec/13-unsafe.md)

## Options considered

- **`unsafe` blocks (chosen):** the exact unsafe lines are marked and
  queryable, giving the smallest unsafe surface. Con: one marker per region.
- **`unsafe` modules:** no markers inside, but coarse, since everything in the
  module looks equally risky.

## Decision

1. Unsafe operations (ADR 0106) are allowed only inside an `unsafe` block.
2. `unsafe fn` declares a function whose callers must uphold conditions the
   compiler can't check. It may be called only inside an `unsafe` block.
3. A package may contain `unsafe` blocks only if its manifest opts in to
   unsafe. Dependencies therefore can't silently contain unsafe code
   (supply-chain safety, [ai-author-wishlist.md 5.1](../design/ai-author-wishlist.md)).
   Single-file programs (ADR 0101) may opt in with a file-level attribute,
   whose exact form is to be decided.
4. The compiler and AI semantic API list every unsafe block and unsafe
   function.

## Consequences

- Manifest syntax for the opt-in comes with the build-system design.
- An `unsafe` block should document its safety reasoning. A
  `## Safety:` doc-comment convention is to be considered with documentation tooling.
