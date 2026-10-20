# 0164. `@pure` rejects a function with I/O, `unsafe`, `spawn` or a capability

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** [eff.pure.1](../spec/18-effects.md)
- **Builds on:** [ADR 0083](0083-attribute-syntax.md) (attributes), [ADR 0163](0163-inferred-effect-sets.md)

## Context

Inferred effects (ADR 0163) tell a reader what a function does; `@pure` lets
an author state what it must not do, so a later edit that adds I/O deep in a
callee is caught where the promise was made. The deciding criterion is **AI
understanding**: the diagnostic must say *where* the effect comes from.

## Options considered

### Option A — forbid `io`, `unsafe`, `spawn` and capabilities; allow `fail` and `mutates` (chosen)
- **Pros:** "pure" means "touches nothing outside its arguments", which is
  what a caller or a memoiser needs; failure and `mut` parameters are already
  visible in the signature and at the call site.
- **Cons:** differs from the strictest definition of purity.

### Option B — forbid every effect
- **Pros:** strictest.
- **Cons:** rules out `T ! E` parsers and `mut`-parameter helpers, which
  are deterministic and local; authors would drop `@pure` instead.

## Decision

`@pure fn f(...)` is a compile-time error (`eff.pure.1`) if `f` has `io`,
`unsafe`, `spawn` or a capability. One diagnostic per function, at the
attribute, names the shortest call chain for each forbidden effect, grouping
effects that share a chain:
``f` is `@pure` but has effects `fs` and `io` via f -> save -> fs.write_text`.

## Rationale

The chain turns a whole-program fact into a local fix: the AI sees which
callee to change without searching. Attribute parsing comes from lane 7
(ADR 0167); `pure` is one of its known attributes.

## Consequences

- `@pure` applies to top-level functions and methods (`fn Type.m`); the
  bootstrap doesn't accept attributes inside `trait` or `impl` blocks yet.
- A call through `any Trait` makes a `@pure` function impure if any
  implementation has an effect (ADR 0163).

## Rejected alternatives

- **Forbid everything (B):** `fail` and `mutates` are already explicit, so
  forbidding them adds no information and loses most useful pure functions.
- **A warning instead of an error:** a promise that isn't enforced isn't one.
