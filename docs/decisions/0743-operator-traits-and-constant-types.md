# 0743. `trait.op.2` and `ct.const.4` get their own diagnostics

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 10 — Traits; 15 — Compile-time execution
- **Spec rules:** trait.op.2, ct.const.4
- **Builds on:** [ADR 0077](0077-operator-traits.md), [ADR 0225](0225-constants-call-compile-time-safe-functions.md)

## Context

Lane 225 found two rules the checker never emitted. `impl Pow for T` was reported as
`expr.name.1` (unknown trait), which says nothing of why no such trait will appear.
A constant of a `unique` type was accepted, and a `ref` in a constant's value was
reported under `mod.const.2` only when written as a `ref` expression, never when a
function returned the cell.

## Decision

- `trait.op.2`: `impl X for T`, where `X` names an operator trait of another language
  that AIL has no operator for (`Pow`, `Not`, `BitAnd`, `BitOr`, `BitXor`, `Shl`, `Shr`,
  the `...Assign` traits, `Deref`, `DerefMut`, `Call`, `Assign`, `Concat`), reports
  `trait.op.2` and suggests a named method. Other unknown names stay `expr.name.1`.
  `Add`, `Index` and the like are `trait.op.1`'s, whose traits the bootstrap doesn't
  have yet; they stay `expr.name.1`.
- `ct.const.4`: a constant whose type is or holds a `ref` or a `unique` type is an error
  at its value. A literal `ref e` stays `mod.const.2` (`own.ref.6` names that rule); the
  type check covers a cell that comes from a call, a list, or a struct field. Where both
  would report the same place, only the first does.
- The self-hosted checker mirrors `trait.op.2`. `ct.const.4` needs constant types, which it
  does not compute yet; its `coverage.ail` does not list the rule.
- `tools/garlic/js/vm/gen.ail` held `GO := Stop(...)`, a constant of a type that holds a
  `ref` (through `Value`). It is now `fn go()`, one line and no change in behaviour.

## Rejected alternatives

- **Recurse into fields only for a `ref` that the value really holds.** The rule is on the
  constant's type, and a value-level check would make `GO` pass until someone stores a
  cell in it.
- **Report `ct.const.4` for a literal `ref` too.** `own.ref.6` and the existing case
  `own_ref_misuse` name `mod.const.2` for it; two rules for one mistake would split the
  guidance.
