# 0113. Struct literals name every field exactly once, in any order

- **Status:** Accepted; amended by [ADR 0142](0142-positional-constructor-arguments.md) (positional arguments)
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 4 — Type system
- **Spec rules:** [type.struct.4](../spec/04-types.md)
- **Partly resolves:** the field-order question left open by [ADR 0058](0058-struct-literal-syntax.md)

## Deciding criterion

Token usage **ties**. **AI understanding** decides: every field must be
named, so a missing field is a compile-time error rather than a silent
default. Defaults may come later as an explicit feature. Order is free, so
edits never fail on ordering, and the canonical formatter writes declaration
order.

## Decision

A struct literal lists each field of the struct exactly once, in any order.
Missing, unknown or repeated fields are compile-time errors. Field defaults
and shorthand are still open.

## Rejected alternatives

- Fixed field order in struct literals: rejected because order is free, so edits never fail on ordering, and the formatter writes declaration order.
- Missing fields filled with silent defaults: rejected because a missing field is a compile-time error rather than a silent default. Defaults may come later as an explicit feature.
