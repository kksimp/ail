# 0117. Typing rules for fields, assignment, `match`, loops and places

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 4 — Type system; 2 — Core syntax (expressions); 7 — Ownership
- **Spec rules:** [type.struct.5–6](../spec/04-types.md), [type.enum.5](../spec/04-types.md), [expr.bind.6](../spec/05-expressions.md), [expr.block.3](../spec/05-expressions.md), [expr.if.4](../spec/05-expressions.md), [expr.match.4–5](../spec/05-expressions.md), [expr.loop.4–5](../spec/05-expressions.md), [own.place.1](../spec/07-ownership.md)

## Context

Implementing structs, enums, assignment, `match` and loops needs typing
rules the spec implied but didn't state. Most follow directly from "no
implicit conversions" ([ADR 0108](0108-call-rules-and-syntax-errors.md)).
The term "mutable place" was used by several rules but never defined.

## Deciding criterion

Token usage **ties** throughout. **AI understanding** decides each point: one
explicit rule per error, and no unreachable or ambiguous code accepted
silently.

## Decision

1. **Fields and variants.** A struct's field names are distinct, and so are
   an enum's variant names. Each value in a struct literal has its field's
   type. `e.f` requires `e` to be a struct with a field `f`.
2. **Places.** A place is a local binding, parameter or `self`, or a field
   (`p.f`) or element (`p[i]`) of a place. It's mutable when its root is
   a `mut` binding, a `mut` parameter or `mut self`.
3. **Assignment.** `p = e` and `p op= e` need a mutable place `p`, and
   `e` must have `p`'s type.
4. **Blocks that leave.** A block ending in `return`, `break` or
   `continue` never produces a value, so it fits wherever any value is
   expected.
5. **Using a value.** An `if` or `match` whose value is used (bound,
   passed, returned, or ending a block whose value is used) must produce a
   value on every path. An `if` therefore needs an `else`. All branches or
   arms must have the same type.
6. **Patterns.** Until general patterns are designed, a pattern is `_` or a
   variant of the scrutinee's enum, written bare or qualified. An arm that
   earlier arms already cover is an error, since it can never run.
7. **Loops.** `for x in e` needs `e` to be a `List[T]` (`x` is a `T`)
   or a range of `int` (`x` is an `int`). `x` is an immutable binding
   scoped to the loop body. The loop iterates the value `e` had when the
   loop started (`own.val.1`). `break` and `continue` outside a loop are
   errors.

## Rejected alternatives

- **Warning instead of an error for an unreachable arm** (Rust, Swift): a
  duplicate arm is almost always a mistake, and AIL already rejects
  non-exhaustive matches (ADR 0056).
- **Treating a sibling branch's type as unknown:** after the first branch of
  an `if` or the first element of a list, its type is the expected type of
  the rest, so `[Grade.A, B]` works as `type.enum.3` intends.
