# 0309. `ail api --expanded` lists generated functions under their `impl`; built-in derives by name

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API; 17 — Metaprogramming
- **Spec rules:** [ct.gen.3–4](../spec/15-compile-time.md)
- **Resolves:** OQ-096 (option b)
- **Implements:** [ADR 0228](0228-generated-code-is-visible-hir.md)

## Context

ADR 0228 asks for generated code in `ail api --expanded` with an origin and
its text, and diagnostics in generated code at the `impl` with a note
`gen:impl:T.Trait:LINE:COL`. OQ-096 asks whether the native `Eq`, `Ord` and
`ToStr` derives also print text. **AI understanding decides**, then
**tokens** of the default outline (unchanged).

## Options considered

### Option A — generated functions under the `impl`, with `origin` and `text`; built-in derives by name (chosen)
- **Pros:** one place to look; the default outline is unchanged, so agents
  that don't ask pay nothing; `derived_by` says how every empty `impl` gets
  its methods.
- **Cons:** an associated function `T.from_json` appears under the `impl`,
  not the type.

### Option B — print template text for the built-in derives too (OQ-096 a)
- **Pros:** uniform.
- **Cons:** text that isn't what runs; more to keep in step.

## Decision

1. With `--expanded`, each generated function is an item in its `impl`'s
   `methods`, with `line`/`col` in the generated text, an `origin`
   `{"kind":"derive","impl":"impl:T.Trait","generator":"fn:f","at":"gen:impl:T.Trait:L:C"}`
   and its `text`; `--text` prints the text under the signature.
2. The `impl` gets `"derived_by"`: the generator's id, or `"built-in"` for
   an empty `impl` of `Eq`, `Ord` or `ToStr` (OQ-096 option b).
3. A diagnostic in generated code is reported at the `impl`, with a note
   naming `gen:impl:T.Trait:LINE:COL`.

## Consequences

- Additive keys under `api 1`; no version bump.

## Rejected alternatives

- **Template text for built-ins (B):** shows code that doesn't run.
- **A separate `expansions` list:** two places to look (ADR 0228 B).
