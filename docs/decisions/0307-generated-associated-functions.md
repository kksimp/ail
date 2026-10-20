# 0307. Generated text may add associated functions of the deriving type

- **Status:** Accepted; amended by [ADR 0350](0350-default-and-generic-fromjson.md) (`FromJson` declares `from_json`, generic types derive it)
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 17 — Metaprogramming
- **Spec rules:** [ct.gen.1](../spec/15-compile-time.md) (amended), [json.derive.2](../spec/std-json.md)
- **Amends:** [ADR 0228](0228-generated-code-is-visible-hir.md) point 1

## Context

ADR 0228 limits generated text to the methods of the deriving `impl`.
`FromJson` must construct a value: `from_json(j Json) Self ! JsonError`
has no `self`, and a trait function without `self` isn't designed
(OQ-039). **Tokens and AI understanding decide**: the use site
`T.from_json(j)` reads like every other constructor.

## Options considered

### Option A — allow `fn T.name(...)` for the deriving type (chosen)
- **Pros:** the associated function is ordinary AIL that `ail api` and the
  checker already know; `FromJson` can be a marker trait with a generator.
- **Cons:** the trait doesn't promise `from_json`, so generic code can't
  call it through a bound until OQ-039 is designed.

### Option B — wait for trait functions without `self` (OQ-039)
- **Pros:** one mechanism.
- **Cons:** no `FromJson` tonight; OQ-039 is a serious decision.

### Option C — a receiver-taking `from_json(self, j Json)`
- **Pros:** fits today's traits.
- **Cons:** needs a dummy value to call; misleading.

## Decision

Generated text holds method definitions for the `impl` and associated
functions `fn T.name(...)` of its type `T` only; the latter are declared at
the top level of the `impl`'s module. Anything else is `ct.gen.1`.

## Consequences

- `FromJson` (ADR 0308) has no methods; when OQ-039 lands, it can declare
  `fn from_json(j Json) Self ! JsonError` and the generated text is already
  right.
- Generated functions keep their stable ids (`method:T.from_json`).

## Rejected alternatives

- **Wait for OQ-039 (B):** blocks the main use of derives.
- **Receiver-taking constructor (C):** a fake receiver.
