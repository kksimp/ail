# 0073. Types implement traits explicitly with `impl Trait for Type`

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 10 — Traits/interfaces
- **Spec rules:** [trait.decl.1, trait.impl.1–3](../spec/10-traits.md)
- **Evidence:** [traits-generics-options.md §2](../design/traits-generics-options.md#2-how-a-type-declares-that-it-satisfies-a-trait) (explicit 44.4, structural 40.0, declared on the type 42.2)

## Options considered

- **Explicit `impl Trait for Type` (chosen):** visible intent, completeness
  checked in one place, no accidental conformance, and works retroactively.
  Con: ~10% more tokens than structural for the conformance.
- **Structural (Go-style):** fewest tokens, but accidental conformance and
  errors far from the type.
- **Declared on the type:** can't be added to existing types.

## Decision

1. A trait is declared with `trait Name` and an indented list of method
   signatures.
2. A type implements a trait only through an `impl Trait for Type` block
   containing definitions of the trait's methods. There is no structural
   (implicit) conformance.
3. A missing or mismatched method in an `impl` is a compile-time error,
   reported at the `impl` and listing what's missing.

```
trait Shape
    fn area(self) f64

impl Shape for Circle
    fn area(self) f64
        pi * self.r * self.r
```

## Consequences

- The semantic API's "what implements this trait?" query can answer from
  declarations directly.
- Coherence (which packages may write `impl Trait for Type`, to prevent
  conflicting implementations) is part of OQ-036.
- Default methods, associated types, and traits for operators are still to
  be decided (next question batch).

## Rejected alternatives

- **Structural:** accidental conformance, and errors far from intent.
- **Declared on the type:** no retroactive conformance.
