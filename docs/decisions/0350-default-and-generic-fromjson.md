# 0350. `Default` and generic `FromJson` through associated functions

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 17 — Standard library
- **Spec rules:** `trait.assoc.1`, `json.derive.2`
- **Amends:** [ADR 0307](0307-generated-associated-functions.md), [ADR 0343](0343-json-derives-maps-optionals-enum-payloads.md)

## Context

`FromJson` had no methods: its derive generated an inherent `fn T.from_json` (ADR 0307), so generic code couldn't call it and a generic type's derive was a `ct.derive.4` error until OQ-039. ADRs 0348 and 0349 now allow trait functions without `self`.

## Decision

1. `trait FromJson` declares `fn from_json(j Json) Self ! JsonError`. Its derive fills the empty `impl` with it, like `ToJson`'s. `str`, `int`, `float`, `bool`, `Json` and `List[T FromJson]` implement it. Call sites don't change: `Todo.from_json(j)`.
2. `impl FromJson for Pair[A FromJson, B FromJson]` derives a `from_json` that reads a field of type `A` with `A.from_json`. Without the bounds, the generated call is a `trait.assoc.2` error naming the bound to add.
3. A prelude trait `Default` with `fn default() Self`, in an optional prelude part read when a program names `Default`. An empty `impl Default for T` derives a struct of each field's default (`0`, `0.0`, `""`, `false`, empty collections, `none`, or the field type's `default()`); an enum is a `ct.derive.4` error, since no variant is the obvious default. `int`, `float`, `str`, `bool` and `List[T]` implement it.

## Rationale

**Tokens decide** for the name `default` over `zero` (a struct of strings and lists isn't a zero; both are one token). **AI understanding**: one way to read a value from JSON, the same for generic and plain types, and one trait an agent reaches for to make a blank value.

## Rejected alternatives

- *Keep the inherent `fn T.from_json`* and special-case generic types: two mechanisms for one trait.
- *`Default` for enums picking the first variant*: silent and order-dependent; the `impl` writes one line instead.
- *`Default` always in the prelude*: every program would load it; the optional part keeps start-up unchanged.
