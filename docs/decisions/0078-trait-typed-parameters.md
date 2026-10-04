# 0078. A trait used as a parameter's type means an anonymous generic (static dispatch)

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 9 — Generics; 10 — Traits
- **Spec rules:** [gen.anon.1–2](../spec/09-generics.md)
- **Evidence:** `corpus/r3/p1_named_generic.ail` vs `p2_trait_param_sugar.ail`: 20.4 vs **17.4** (−15%)

## Deciding criterion

**Token usage:** `fn draw(s Shape, ...)` is 15% cheaper than
`fn draw[T Shape](s T, ...)`. On AI understanding it ties: semantics are
identical, because under value semantics only performance could differ, and
both forms dispatch statically.

## Decision

1. When a parameter's type is written as a trait name, it declares an
   anonymous type parameter constrained by that trait:
   `fn draw(s Shape)` means `fn draw[T Shape](s T)`.
2. This applies only to a parameter's top-level type. Elsewhere (inside
   `List[...]`, in fields, return types or variables), a bare trait name is
   an error, and the diagnostic suggests a named type parameter or `any Trait`
   (ADR 0079).
3. Canonical form: the anonymous form when the type parameter is used exactly
   once. Otherwise the named form.

## Alternatives not chosen

- **Named generics only:** +15% tokens on the most common generic signature.
- **Trait name means dynamic dispatch (Go/Java style):** hidden runtime cost,
  and inconsistent with rule 2.
