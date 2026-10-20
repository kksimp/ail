# 0228. Generated code is ordinary HIR with a provenance link, shown by `ail api --expanded`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 17 — Metaprogramming; 19 — AST/IR architecture; 20 — AI semantic API
- **Spec rules:** [ct.gen.1–4](../spec/15-compile-time.md)
- **Design note:** [metaprogramming.md](../design/metaprogramming.md) §4, [ir-architecture.md](../design/ir-architecture.md) §3

## Context

Wishlist 4.5: any generated code must be viewable through the API, so an
agent can see what metaprogramming produced. Generated code also needs
scope rules and a place for its diagnostics. **AI understanding decides**;
tokens are unaffected in source, and the `--expanded` output is opt-in.

## Options considered

### Option A — generated items are HIR items with an origin, printed on request (chosen)
- **Pros:** references, effects, diagnostics and stack traces work on
  generated code with no special cases; the agent reads AIL.
- **Cons:** the API needs an origin key and a way to address lines of
  generated text.

### Option B — a separate "expansion" view, not part of HIR
- **Pros:** the HIR stays source-only.
- **Cons:** two places to look; queries over HIR would miss generated
  methods.

## Decision

1. Generated text holds only the `impl`'s method definitions; it is parsed,
   formatted canonically, and checked as if written in that `impl`
   (`ct.gen.1`).
2. It resolves names in the `impl`'s scope (its module, imports and the
   type's private fields), plus the trait's module by its qualified name
   (`ct.gen.2`).
3. A diagnostic in generated code is reported at the `impl` line, with a
   note at the generated line, addressed `gen:impl:Version.ToJson:LINE:COL`
   (`ct.gen.3`).
4. Each generated item has the usual id and an `origin`
   `{"kind":"derive","impl":"impl:Version.ToJson","generator":"fn:derive_to_json"}`;
   `ail api --expanded` includes generated items and their text
   (an additive key under `api 1`) (`ct.gen.4`).

## Consequences

- **Designer, look first at:** whether `--expanded` should also print the
  built-in derives' text (OQ-096).
- Stack traces and debug info into derived methods point at lines of the
  expanded text, with the origin chain to the `impl`.
- Implemented with ADR 0227 when the bootstrap gains generators.

## Rejected alternatives

- **A separate expansion view (B):** generated methods would be invisible to
  HIR queries.
- **Hiding generated code (as most macro systems do by default):**
  contradicts wishlist 4.5.
