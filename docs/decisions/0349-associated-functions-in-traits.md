# 0349. Associated functions in traits, called as `T.name()`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 10 — Traits
- **Spec rules:** `trait.assoc.1–4`
- **Resolves:** OQ-039 (associated functions; with [ADR 0348](0348-supertraits.md))

## Context

Generic code couldn't construct a value of a type parameter: no `T.zero()`, no `Default`, no generic `FromJson` ([ADR 0343](0343-json-derives-maps-optionals-enum-payloads.md)). A trait function without `self` has no value to dispatch on, so the implementation must know the type at the call ([ADR 0080](0080-generic-compilation-strategy.md)).

## Decision

1. A trait function without `self`, `fn zero() Self`, is an associated function of the trait. An `impl` defines it with `Self` replaced, or inherits a default body, which may call `Self.zero()` (`trait.assoc.1`).
2. `T.zero()` on a type parameter calls the function of a trait among `T`'s bounds and their supertraits; `Type.zero()` and `Type[A].zero()` call it on an implementing type that declares no function of that name itself (`trait.assoc.2`, `trait.assoc.4`). `x.zero()` on a value is an error with the help `Type.zero(...)`.
3. `T` is the type of the running call: inferred from the arguments or the expected type, or written (`zeros[int](3)`) (`trait.assoc.3`).
4. The bootstrap passes types at run time only where needed: the checker records the type arguments of calls whose type parameters are bound by a trait with associated functions, and the interpreter keeps a stack of those bindings for the callee. Programs that don't use the feature pay one empty-map check per call.

## Rationale

**Tokens decide**: `T.zero()` reuses the associated-function call form (`Type.name()`, [ADR 0082](0082-method-placement-coherence.md)), with no new syntax; the agent corpus is unchanged. **AI understanding**: the call names the type it runs for, and the checker states which bound provides it. **Speed**: a compiler can monomorphise or pass witness tables; the interpreter's cost is limited to the calls that need it.

## Rejected alternatives

- *A `zero` value argument (`zeros(n, 0)`)*: pushes the work onto every caller and doesn't extend to `from_json`.
- *Static-only `Type.name()` with no calls through a type parameter*: leaves `Default` and generic `FromJson` impossible.
- *Pass every generic call's type arguments at run time*: costs every generic call in the interpreter's hot path for a rare feature.
