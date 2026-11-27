# 1227. Integer and float math helpers

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 04 Types (numbers)
- **Spec rules:** [type.num.17, type.num.18](../spec/04-types.md)

## Context

The std gap plan (lane 13) found agents writing `gcd`, integer square roots and NaN tests by hand, often wrongly
(overflowing `lcm`, `x != x`).

## Decision

- `int` gains `a.gcd(b)`, `a.lcm(b)`, `n.isqrt()`, `n.is_pow2()` and `n.log2()` (floor). A result that does not fit
  traps under `expr.arith.1`: `gcd` of `int.MIN` with 0, an overflowing `lcm`, `isqrt` of a negative, `log2` of 0 or less.
- `float` gains `x.log2()` (exact for powers of two), `is_nan()`, `is_finite()`, `is_infinite()` and the constants
  `float.PI`, `E`, `TAU`, `INF` and `NAN`.
- They are built-in methods (`check/builtins.ail`), lowered in their own `mir/lower_intmath.ail` (ADR 0823) with runtime
  functions in `rt/native/math.ail`.

## Rejected alternatives

- A `std.math` module: the methods sit beside `abs`, `pow` and `sqrt`, where an agent already looks.
- Returning `int?` for the domain errors: these are programming errors, and every other integer overflow traps.
- Sized ints and `f32` in the same change: left for a later ADR, to keep the change to one type each.
