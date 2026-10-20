# 0454. Named arguments at call sites: `f(a, dy: 2, dx: 1)`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 3 — Expressions and control flow
- **Spec rules:** [expr.call.3](../spec/05-expressions.md)
- **Builds on:** [ADR 0058](0058-struct-literal-syntax.md), [ADR 0142](0142-positional-constructor-arguments.md) (constructors already take labels)

## Context

A call such as `shift(p, 2, 5)` hides which number is `dx`. Constructors
already take `field: value`; functions don't, and the checker rejected labels
there. The deciding criterion is **AI understanding**: a call that names its
arguments can be checked and read without the declaration. Tokens are a cost:
three calls cost 39 tokens positional and 51 named on cl100k (+3 per name),
so naming is opt-in.

## Options considered

### Option A — `name: value` after any positionals, any order (chosen)
- **Pros:** the constructor syntax, nothing new to learn; a reordered call
  stays checkable; mixes with positional use.
- **Cons:** +3 tokens per named argument.

### Option B — all-or-nothing named calls
- **Cons:** forbids the common `f(p, dx: 2, dy: 5)`.

### Option C — default parameter values with named arguments
- **Cons:** a separate design (OQ for defaults); keeping the two apart lets
  this ship now. There are no defaults yet, so every parameter is given.

## Decision

Option A (`expr.call.3`). Positionals first, then named; each parameter given
exactly once; unknown, repeated and already-filled names, and a positional
after a named one, are errors, with a did-you-mean for a near name.
Arguments are evaluated in **parameter order**, which keeps the interpreter
and the bytecode tier free of per-call reordering: after checking, a pass
(`named_args.rs`) puts the arguments of such calls in parameter order and
drops the labels. Built-ins have no parameter names and reject labels
(`expr.call.3`). Constructors keep their own rule (`type.struct.3`).

## Consequences

- A call through a trait uses the trait declaration's parameter names.
- Function values (ADR 0456) are called positionally.
- Surprise: `f(b: g(), a: h())` runs `h` before `g`.
