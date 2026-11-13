# 0455. Function types `fn(A) R`, and declared functions as values

- **Status:** Accepted; amended by [ADR 1067](1067-closures.md)
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 4 — Type system
- **Spec rules:** [type.fn.1-2](../spec/04-types.md)
- **Builds on:** [ADR 0037](0037-signature-syntax.md), [ADR 0250](0250-key-functions-and-take.md) (key functions are the one place a function-like argument exists)

## Context

Agents reach for `apply(f, x)`, dispatch tables and callbacks. Today the
only way is `match` on a name, and `f := double` was reported as
unsupported. The brief assumed `fn` types existed; they don't, so this ADR
adds the type and the value together. The deciding criteria are **AI
understanding** (a type that says what can be passed) and **tokens**.

## Options considered

### Option A — `fn(A, B) R` types and declared functions as values (chosen)
- **Pros:** the type mirrors a declaration, `fn name(a A, b B) R`, minus the
  names; no new keyword. Needs no closure machinery: a value is an index, so
  copying is free and the checker needs no capture analysis. Tokens: this
  is a capability, not a saving. A name-to-operation `pick(name)(a, b)` costs
  101 tokens on cl100k against 90 for `match name` with direct calls, so
  agents should reach for it for callbacks (`apply(f, x)`), not for dispatch.
- **Cons:** no anonymous functions; a callback that needs a variable is a
  struct plus a function, as before.

### Option B — closures `fn(x) x + n` as well
- **Cons:** captures need a model (copy or share, by mutable value
  semantics, ADRs 0060-0063), and `fn(m) key` was already rejected for key
  functions (ADR 0250) because it weakens `fn` as a declaration anchor. A
  separate decision for later.

### Option C — `Fn[A, B, R]` as a generic type
- **Cons:** a `fn` that is not the declaration keyword; longer.

## Decision

Option A. `fn(A, B) R` is a type (`type.fn.1`); the result follows `)` as in
a declaration, with no result meaning `Unit`. A function value is only a
declared top-level function that is not generic, not a test, has no `mut` or
`move` parameter, and cannot fail (`type.fn.2`), so every function value has
one fixed positional signature and calls are checked with no inference. A
result type is an optional type at most (`fn(int) int?`); a function that
can fail is not a value. Naming a function as a value counts as calling it
for the effect set, which keeps `@pure` and `ail effects` sound.

## Consequences

- Function types have no `==`, hash, `to_str` or generated test values.
- Methods and associated functions are not values yet.
- Generic functions are not values; `apply[T]` taking a `fn(T) T` is allowed,
  since its parameter is a type, not a value of a generic function.
- `ail fmt` writes `fn(int) int` with no space after `fn`.

## Rejected alternatives

- Closures such as `fn(x) x + n` (option B): a separate decision for later. Captures need a model under mutable value semantics (ADRs 0060-0063), and `fn(m) key` was already rejected for key functions (ADR 0250) because it weakens `fn` as a declaration anchor.
- `Fn[A, B, R]` as a generic type (option C): rejected because it is a `fn` that is not the declaration keyword, and it is longer.
