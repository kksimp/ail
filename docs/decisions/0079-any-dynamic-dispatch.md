# 0079. Dynamic dispatch is explicit: `any Trait`

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 10 — Traits; 25 — Optimization
- **Spec rules:** [trait.any.1–2](../spec/10-traits.md)
- **Evidence:** `corpus/r3/d1_any.ail` 10.6, `d2_dyn.ail` 10.6, `d3_implicit.ail` 9.4 (one token, in a rare construct)

## Deciding criterion

**Token usage ties**: one token, in a construct far rarer than static
generics. **AI understanding decides**: with ADR 0078, a bare trait name
means static dispatch, so the dynamic case must look different. `any` makes
the runtime cost visible. `any` and `dyn` tie on tokens, and `any` reads
more naturally.

## Decision

1. `any Trait` is a type whose values can hold any type implementing `Trait`.
   Method calls on it dispatch dynamically.
2. Values of `any Trait` obey value semantics like any other value (ADR 0060).

```
struct Scene
    shapes List[any Shape]
```

## Consequences

- Dynamic-dispatch sites are listed in the performance report, and the
  optimizer may devirtualise them (whole-program optimisation, ADR 0015).
- Which traits can be used with `any` (object safety) is to be specified.
