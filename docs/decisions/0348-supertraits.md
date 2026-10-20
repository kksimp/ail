# 0348. Supertraits are written after the trait name, like bounds

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 10 — Traits
- **Spec rules:** `trait.super.1–3`, `trait.std.2`, `trait.auto.5`
- **Resolves:** OQ-039 (supertraits; with [ADR 0349](0349-associated-functions-in-traits.md))

## Context

`Ord` requires `Eq` ([ADR 0127](0127-standard-comparison-and-text-traits.md)) and a `[T Hash]` bound allows `==` ([ADR 0133](0133-automatic-hash.md)), but both were special cases in the checker, and user traits had no way to require another. AIL writes no `:` in declarations ([ADR 0037](0037-signature-syntax.md)).

## Decision

1. `trait Ord Eq` and `trait Key Hash + ToStr`: supertraits follow the name, joined with `+` exactly as in a bound `[T Hash + ToStr]`. A trait with supertraits may have no methods (a named bundle of bounds). A cycle is an error (`trait.super.1`).
2. `impl Sub for T` needs every supertrait implemented for `T` (`trait.super.2`, help: the missing `impl`).
3. A bound `[T Sub]` provides each supertrait, transitively (`trait.super.3`). The prelude now declares `trait Ord Eq` and `trait Hash Eq`; `Hash` stays automatic and unimplementable, so ADR 0133 stays true.

## Rationale

**Tokens decide.** Measured with `tools/tokbench/run.sh` on two declarations (mean over five tokenizers): `trait Ord Eq` 30.4, `trait Ord with Eq` 32.4, `trait Ord requires Eq` 32.4. Supertraits are rare, so the gap is small in a corpus, but there is no counter-argument. **AI understanding** agrees: one spelling for "these traits are required", in bounds and in traits, so nothing new to learn.

## Rejected alternatives

- *`trait Ord: Eq`* (Rust, Swift): contradicts ADR 0037.
- *`trait Ord with Eq` / `requires Eq`*: one more token each, and a keyword that appears nowhere else.
- *Keep the special cases*: user traits still couldn't require one another, and `Hash`/`Ord` would stay unexplained exceptions.
