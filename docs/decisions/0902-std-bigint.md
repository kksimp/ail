# 0902. `std.bigint`: integers of any size

- **Status:** Accepted; amended by [ADR 1124](1124-arithmetic-operator-traits.md)
- **Date:** 2026-10-25
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library
- **Spec rules:** std.bigint.1-9 ([17-std](../spec/17-std.md))

## Context

The adoption-gaps review (2026-10-24) listed bigint as missing. Python's `int`
is unbounded, so ports of hashing, number-theory, finance and puzzle code from
Python hit the 64-bit `int` limit silently or with a trap.

## Decision

`std.bigint`, written in AIL (`std/bigint/bigint.ail`), imported with
`use std.bigint`: `BigInt` (sign and base-10^9 limbs), `from_int`, `parse`,
`to_text`, `as_int`, `add`, `sub`, `mul`, `divmod` (toward zero), `pow`,
`factorial`, `cmp`, `negate`, `abs`, `is_zero`, `is_negative` and
`BigIntError` (std.bigint.1-9). Base 10^9 makes `to_text` and `parse` linear
and keeps a limb product inside a 64-bit `int`. Division is schoolbook with a
bisected quotient limb. The module is pure AIL: no lowering, runtime or Rust
change (ADR 0873).

## Rejected alternatives

- **Operator overloading (`a + b`).** Not part of the language today; methods
  need none, and operators can be added when the language gets them.
- **Base 2^32 limbs.** Faster arithmetic, but `to_text` needs repeated
  division. Printing is the common use; speed can follow with a native kernel.
- **A native runtime type.** Needs lowering and runtime work in both backends
  (ADR 0008). Start in AIL; move hot loops down if measurements ask.
- **Rationals and decimals in the same ADR.** Separate modules, later.
