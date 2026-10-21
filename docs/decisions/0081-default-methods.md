# 0081. Traits may provide default method implementations

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 10 — Traits
- **Spec rules:** [trait.decl.2](../spec/10-traits.md)

## Deciding criterion

**Token usage:** each implementation of a trait otherwise repeats every
derivable method (e.g. `!=` from `==`, or `contains` from iteration).

## Decision

A method in a `trait` block may have a body. Implementations inherit it,
unless their `impl` block defines the method itself.

```
trait Shape
    fn area(self) f64
    fn is_larger(self, other Self) bool
        self.area() > other.area()
```

## Consequences

- `Self` (the implementing type) is used in trait signatures. Its exact
  rules are to be specified.
- Associated types vs. trait type parameters is still open (OQ-037).

## Rejected alternatives

- No default methods (every implementation spells out each derivable method, such as `!=` from `==`): loses on token usage, the criterion that decides here.
