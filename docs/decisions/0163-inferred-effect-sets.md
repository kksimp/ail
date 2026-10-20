# 0163. Every function has an inferred effect set, shown by `ail effects`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API; 21 — Standard library architecture
- **Spec rules:** [eff.infer.1–6](../spec/18-effects.md)
- **Plan:** [ai-first-plan.md](../design/ai-first-plan.md) §3.1

## Context

An AI reasoning about a call wants to know what the callee does to the
machine without reading its body (wishlist 1.1). The plan (§3.1) asks for an
inferred effect set per function, never written in source, shown to tools and
checkable. The deciding criterion is **AI understanding**: tokens are
unaffected because nothing is written in source.

## Options considered

### Option A — inferred sets, whole-program, shown by a command (chosen)
- **Pros:** zero syntax cost; every function gets a set, including ones
  nobody annotated; whole-program analysis (ADR 0015) resolves trait calls
  exactly as the compiler will; one function (`effects_of`) serves
  `ail effects` now and `ail api` after merge.
- **Cons:** a set can change when a distant callee changes; dynamic dispatch
  needs a conservative rule.

### Option B — effects written in signatures (`fn f() uses io`)
- **Pros:** the contract is visible in source and stable across edits.
- **Cons:** costs tokens on most functions, the first criterion; effect
  polymorphism (generic code over effectful callbacks) needs more syntax.

### Option C — only `io` versus pure
- **Pros:** simplest.
- **Cons:** can't sandbox by resource (§3.2) and hides failure and mutation,
  which tools want too.

## Decision

Each function's set is drawn from the effects `io`, `fail`, `mutates`,
`unsafe`, `spawn` and the capabilities `fs`, `env`, `proc`, `clock`,
`net`. `fail` and `mutates` come from the function's own signature;
the rest pass from callee to caller, as the least fixed point over the call
graph. Sources: `print` (`io`), the standard modules of the plan's §4
table, `unsafe` blocks and `spawn`. A trait call whose target the types
determine counts only that target; any other trait call (`any Trait`, a type
parameter, `Trait.f(x)`) counts **every implementation in the program**,
including the trait's default. Operators, interpolation, `print` and
built-in methods count every implementation of `Eq`, `Ord`, `ToStr` and
`Hash`, since they may call them implicitly. `ail effects FILE [--json]`
prints the sets by stable id (`fn:name`, `method:Type.name`).

## Rationale

The union over implementations is sound and needs no annotations: AIL always
compiles whole programs (ADR 0015), so the set of implementations is known.
`fail` and `mutates` are not passed on because the caller either handles a
failure or is itself fallible (and so has `fail` from its own signature),
and a `mut` argument is the caller's own place, already marked at the call
site (ADR 0061).

## Consequences

- `ail api` (lane 2) fills its `effects` field from `effects_of`.
- The checker records the static target of method and associated-function
  calls (`Resolved::targets`), which the query compiler can reuse.
- Standard-module calls are recognised by the receiver's name until lane 4's
  resolution lands; switching to the resolved call is a one-line change in
  `std_effects`.
- `unsafe` and `spawn` have no source until the bootstrap parses them.
- A set may over-approximate where dispatch is dynamic. `alloc` arrives with
  the backend.

## Rejected alternatives

- **Written effects (B):** costs tokens on most signatures; inference gives
  the same information to tools for free. Can be added later for public APIs.
- **Pure/impure only (C):** too coarse for capability sandboxing (ADR 0165).
- **Ignoring implicit standard-trait calls:** unsound; a `ToStr` that prints
  would make `"{x}"` silently impure.
