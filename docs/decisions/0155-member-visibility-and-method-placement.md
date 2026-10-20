# 0155. Methods live in their type's module; private fields block outside construction

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 12 — Modules
- **Spec rules:** [mod.unit.4](../spec/12-modules.md), [mod.vis.2–3](../spec/12-modules.md)

## Context

ADR 0098 makes fields and top-level declarations private unless `pub`, with
variants and trait methods following their enum or trait. Open points: can a
module add methods to another module's type; what `pub` means on members that
follow their owner; and whether a struct with private fields can be built
elsewhere.

## Decision

1. `fn T.name` is declared in `T`'s module; `fn m.T.name` is an error
   (`mod.unit.4`). `impl Trait for T` may appear in any module.
2. `pub` before a variant, a variant field, a `trait` or `impl` method, or an
   `impl` is an error with the help "remove `pub`" (`mod.vis.2`).
3. A struct literal of a struct with a private field is allowed only in its
   module; reading or assigning a private field and calling a private method
   or associated function elsewhere are errors (`mod.vis.3`).

## Rationale

AI understanding decided. (1) All of a type's methods are found in one
module, so `x.f()` resolves by reading one place. (2) A marker that does
nothing invites the belief that it does something. (3) A private field is an
invariant the module keeps; a literal elsewhere could break it, so the
module's constructor function is the only way in (as in Rust).

## Consequences

- Extending another module's type is done with a trait `impl` or a free
  function.
- Coherence of trait impls across modules (orphan rules) stays open until
  packages exist.

## Rejected alternatives

- **Methods anywhere:** a type's API would be spread across importers.
- **Ignoring redundant `pub`:** silent markers mislead readers.
- **Building structs with private fields anywhere:** defeats the privacy.
