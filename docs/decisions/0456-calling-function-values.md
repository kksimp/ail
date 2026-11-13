# 0456. Calling a function value, and its effects

- **Status:** Accepted; amended by [ADR 1067](1067-closures.md)
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 4 — Type system
- **Spec rules:** [type.fn.3](../spec/04-types.md)
- **Builds on:** [ADR 0455](0455-function-types-and-values.md), [ADR 0164](0164-pure-functions.md), [ADR 0454](0454-named-arguments.md)

## Context

A call through a value has a callee the checker can't name, so its effects
can't come from the call.

## Options considered

### Option A — charge effects where the function is named (chosen)
- **Pros:** exact for what a program names; no effect types on `fn(...)`;
  `@pure fn compute()` that passes `log` to `apply` is rejected with the
  chain `compute -> log -> print`.
- **Cons:** a function value received from a caller carries no effect inside
  `apply` itself; the effect shows at the caller that names it.

### Option B — effect-annotated function types, `fn(int) int io`
- **Cons:** a second type language, and a cost in every signature.

### Option C — assume the worst for any call through a value
- **Cons:** makes `@pure` useless wherever a callback is used.

## Decision

Option A. `f(x)` and `(s.run)(x)` call by position (`type.fn.3`); named
arguments (ADR 0454) don't apply, because a value has no parameter names.
`mut` arguments are errors. Calling a value adds no effect of its own; a
function that names another as a value has that function's effects.

## Consequences

- Surprise: `apply(log, 1)` inside a `@pure` function is rejected, but a
  `@pure fn apply(f fn(int) int, x int)` is accepted, since only its callers
  name the effect.

## Rejected alternatives

- Effect-annotated function types such as `fn(int) int io` (option B): rejected because it is a second type language and a cost in every signature.
- Assuming the worst for any call through a value (option C): rejected because it makes `@pure` useless wherever a callback is used.
