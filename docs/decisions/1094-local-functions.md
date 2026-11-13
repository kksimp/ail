# 1094. Named local functions

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer approved every item of the AI-first review for 0.1 on 2026-11-12)
- **Topic:** 4 — Type system
- **Spec rules:** [type.fn.9-10](../spec/04-types.md)
- **Builds on:** [ADR 1067](1067-closures.md), [ADR 0455](0455-function-types-and-values.md)

## Context

A function literal (ADR 1067) is one expression. A helper that needs a
loop, an early `return`, a few statements or recursion had to move to the top
level and take every local it used as a parameter. Agents write nested
helpers by habit (Python `def` inside `def`, Rust `fn` inside `fn`, Go
closures assigned to a name). The review found the top-level detour costs
parameters on every call and moves the helper away from its only use.

## Options considered

### Option A — `fn name(params) R` statement, a closure with a block body (chosen)
- **Pros:** the same spelling as a top-level function, so nothing new to
  learn; captures, effects and the native object are those of literals, so
  one model covers both; recursion by name; usable as a value.
- **Cons:** parameters can't be `mut` or `move` and calls are positional,
  since the local is a function value.

### Option B — block-bodied literals, `|x| <newline> block`
- **Cons:** a literal that spans lines hides control flow inside an
  argument list; `return` would mean two things; no name to recurse with.

### Option C — top-level functions only
- **Cons:** the status quo the review flagged.

### Sub-choice: captures by reference
- **Cons:** rejected as in ADR 1067: value semantics, no lifetimes; a
  captured `ref` cell shares state explicitly.

### Sub-choice: mutual recursion between local functions
- **Cons:** a name is visible only after its declaration, so the copy rules
  stay simple; two helpers that call each other go to the top level.

## Decision

Option A. The name is declared before the body, so the body can call
itself; outer locals are copied when the declaration is reached
(`type.fn.10`). Native code lowers the body into its own function named
`outer.name`, so traces and trap stacks show where it lives.

## Consequences

- Not generic, not a method, no contracts in 0.1.
- The compiler's own source doesn't use local functions until a seed with
  them is published.
