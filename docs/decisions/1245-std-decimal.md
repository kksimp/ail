# 1245. `std.decimal`: exact base-10 numbers with explicit scale and rounding

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the std library to be filled out on 2026-11-26 and 2026-11-27
- **Topic:** 17 — Standard library; AI understanding
- **Spec rules:** std.decimal.1 to std.decimal.12 in [std-decimal.md](../spec/std-decimal.md)
- **Extends:** [ADR 1126](1126-std-modules-in-ail-without-use.md), [ADR 1124](1124-arithmetic-operator-traits.md)

## Context

Money and measured quantities need exact decimal arithmetic. `float` gives `0.1 + 0.2 != 0.3`, and `int` cents lose the
scale the moment a price has a tax or a split. Agents write ad-hoc cents code and get rounding wrong (half-up against
banker's rounding, rounding each step instead of the end). The std plan names a decimal type as a backlog module; the
`+ - *` operator traits (ADR 1124) now let it read like a number.

## Decision

A new module written in AIL, `std/decimal/decimal.ail`, on `std.bigint`, in scope without `use` (std.use.4). No syntax
change, no runtime or checker work.

- `Decimal` is a `BigInt` and a scale `>= 0`; the value is `unscaled / 10^scale`. The scale is kept: `12.50` prints with
  its zero. Fields are private, so no value has a negative scale.
- `parse` reads sign, digits, one `.`, and an exponent; `from_int`, `from_parts` build one. Output is plain text, never
  an exponent form.
- `+ - *` and unary `-` are the operator traits and are exact (`+ -` at the larger scale, `*` at the sum). There is **no
  `/`**: a quotient has no exact decimal form in general, so `divide(other, scale, mode)` takes the scale and a
  `Rounding` mode, and rounds the exact quotient once. `round(scale, mode)`, `pow`, `normalize`, `sum`, `as_int(mode)`
  and `as_float` complete it.
- `Rounding` has seven variants (`Down`, `Up`, `Floor`, `Ceiling`, `HalfUp`, `HalfDown`, `HalfEven`). There is no default
  mode: the caller names one.
- `Eq` and `Ord` compare by value (`1.0 == 1.00`). The hand-written `Eq` removes `Hash` (trait.auto.5); a map key is
  `d.normalize().to_text()`.
- Failures are `DecimalError` with an enum `kind` (type.enum.7): `NoDigits`, `BadDigit`, `BadExponent`, `BadScale`,
  `DivideByZero`, `TooLarge`. A scale is limited to 10000 digits so a text such as `1e999999` cannot allocate without bound.
- The module is named `decimal`: no local name in the repository's AIL sources is `decimal`, so std.use.4 hiding does not
  bite there.

## Rejected alternatives

- **`a / b` through `impl Div`.** It would need a hidden scale and mode (Java's `MathContext` default), which is where
  money bugs come from; a missing `impl Div` gives a `trait.op.1` error that points at `divide`.
- **A default rounding mode or default parameters.** A silent mode is the bug this module exists to remove, and defaults
  pass by name only.
- **Normalized values (always strip trailing zeros).** `12.50` would print `12.5`; money output needs the scale. `normalize`
  is the explicit form.
- **Structural `==` (scale-sensitive), as `std.semver` has for builds.** `1.0 == 1.00` being false while `cmp` says equal
  would surprise every reader; unlike a build string, the scale carries no separate data.
- **Public fields.** They would allow a negative scale and a non-canonical zero sign; accessors cost nothing.
- **A fixed-width `int` backing (cents).** Overflows silently on large sums and fixes the scale for the type.
- **A runtime built-in.** Costs checker, effects and lowering work and collides in `check/stdlib.ail`; AIL on `BigInt` is
  fast enough for amounts.
- **A module named `money` or `bigdecimal`.** `money` is too narrow (measurements use it too); `bigdecimal` is a Java
  name and longer.
- **Mixed `Decimal` and `int` operators.** `trait.op.3` forbids implicit conversion; `from_int` is explicit.
