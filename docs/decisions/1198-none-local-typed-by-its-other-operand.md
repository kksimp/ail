# 1198. A `none` local used as an operand takes its type from the other operand

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 4 — Types; AI understanding
- **Spec rules:** [type.opt.9](../spec/04-types.md) (extended)
- **Amends:** [ADR 0468](0468-optional-none-locals-and-if-branches.md) (what fixes the type of a `none` local)

## Context

The running-best loop an agent writes first is `mut best := none`, then `if best == none or v > best` and
`best = v` in the block. Since ADR 1173 the comparison with `none` leaves `best` waiting for its type, and since
[ADR 1182](1182-tested-optional-reads-as-its-value.md) the right side of that `or` reads a tested optional as
its value. The program still failed: `v > best` is read before the assignment, so it was type.opt.9, "the type of
`best` isn't known yet". A typed start (`mut best := xs.first()`) already worked.

## Decision

- **type.opt.9, extended:** a `none` local with no type yet, written as one operand of a binary operator other
  than `and` / `or`, takes the type `T?` when the other operand has a type `T` (`T?` stays `T?`) and does not
  read the local. This is the type its first assignment `x = e` would give for an `e` of that type.
- The operator is then checked as usual with `x` of type `T?`. In `best == none or v > best` type.opt.12 reads
  `best` as its value; in a bare `v > best` the error is the usual one for an optional operand, whose help shows
  the test or `else`.
- A later assignment of another type is the ordinary mismatch error against `T?`.
- The native lowering does the same where it lowers the operator (`compiler/mir/lower_optoperand.ail`), with
  the function that types the variable at its first assignment.

## Options considered

- **Keep type.opt.9 as it was and improve its help:** rejected. The program is clear to a reader and the type is
  not in doubt; an error here costs an iteration for nothing (criterion 1).
- **Look ahead to the first assignment:** rejected. It makes a read depend on code below it, and the lowering
  would need the same search. The other operand is local to the expression.
- **Also take the type from a call argument's parameter or a `return`:** not now. Those have an expected type
  already and no probe failed there.

## Consequences

- `compiler/check/optpending.ail` (`settle_by_operand`), one line in `expr.ail`; `compiler/mir/lower_optoperand.ail`
  and three lines in `lower_ops.ail`. Nothing in `lower.ail` (ADR 0823).
- When the local is the left operand, the lowering lowers the right operand first. The right operand does not
  read the local, so the order can't be observed through it.
- Cases `optional_none_operand`, and a line in `optional_none_errors`.
- More programs are accepted; none changes meaning.
- Not built in the cloud (ADR 0842); gated on the Mac.
