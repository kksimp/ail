# 0082. Associated functions; methods live in the type's package; coherence rules; no ambiguous calls

- **Status:** Accepted; amended by [ADR 0137](0137-integer-methods-and-min-max.md) (inherent and built-in methods take precedence over standard-trait defaults)
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 10 — Traits; 12 — Modules
- **Spec rules:** [trait.method.4–5, trait.impl.4, trait.res.1](../spec/10-traits.md)
- **Resolves:** OQ-036

## Deciding criterion

Token usage **ties** on all four parts. **AI understanding decides**: every
rule makes "what can `x.f()` mean?" answerable from a single place.

## Decision

1. **Associated functions:** `fn Type.name(...)` without `self` declares a
   function called as `Type.name(...)`, e.g. `Circle.new(1.0)`.
2. **Placement:** methods and associated functions of a type must be declared
   in the package that declares the type.
3. **Coherence:** `impl Trait for Type` may appear only in the package that
   declares `Trait` or the package that declares `Type`. Duplicate
   implementations are impossible by construction.
4. **No ambiguous calls:** if `x.f()` could refer to more than one method (an
   inherent method and a trait method, or two traits' methods), it's a
   compile-time error. The diagnostic offers qualified forms such as
   `Shape.area(x)`.

## Consequences

- Rules 2 and 3 can be relaxed later without breaking existing code. They
  start strict on purpose.
- Wrapping a foreign type to give it a foreign trait (the newtype pattern)
  becomes the standard workaround. The standard library should make it cheap.
