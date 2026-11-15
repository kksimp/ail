# 1124. Arithmetic operator traits, and operators on `BigInt`

- **Status:** Accepted
- **Date:** 2026-11-15
- **Decided by:** Designer (2026-11-15: "operator traits for every type", over a
  `BigInt`-only special case)
- **Topic:** 10 — Traits; 17 — Standard library
- **Spec rules:** [trait.op.1](../spec/10-traits.md), [std.bigint.3, .4, .10](../spec/17-std.md)
- **Amends:** [ADR 0077](0077-operator-traits.md) (names its traits),
  [ADR 0902](0902-std-bigint.md) (operators, `cmp` returns `Ordering`)

## Context

ADR 0077 decided that `+ - * / %` and unary `-` may be implemented for user
types through standard traits, "names to be fixed with the standard library".
They were never named: the checker rejected `impl Add` as `trait.op.2`. In
the E13 agent benchmark, programs that summed past 64 bits had to write
`a.add(b)` chains on `BigInt`, could not compare with `<`, and printed the
struct's fields with `"{b}"`. The approved answer to "BigInt operators or a
128-bit integer" was operators, and no `i128` in 0.2.

## Decision

1. The prelude declares `Add`, `Sub`, `Mul`, `Div`, `Rem`, each
   `fn NAME(self, other Self) Self`, and `Neg` with `fn neg(self) Self`.
   `a + b` on two values of a struct or enum type `T` that implements `Add`
   calls `a.add(b)` and has type `T`; likewise the others. `a += b` is
   `a = a + b`. A bound `[T Add]` gives `+` in generic code.
2. Both operands have the same type (trait.op.3, ADR 0077). The built-in
   numbers keep their own operators and instructions and implement none of
   these traits, so generated code for `int` and `float` is unchanged.
3. The six names leave the `trait.op.2` list. A user type with one of these
   names is a `mod.unit.1` error with a rename fix, as for prelude types; a
   user trait with one is the existing "already a built-in trait" error.
4. Lowering calls the type's method (`compiler/mir/lower_opimpl.ail`, ADR 0823).
5. `std.bigint`: `BigInt` implements the six traits (the methods `add`, `sub`
   and `mul` move into the impls and keep working as methods), `Ord` (so
   `cmp` now returns `Ordering`, not -1, 0 or 1) and `ToStr` (the decimal
   text). `a / b` and `a % b` trap on a zero divisor, as `int` does;
   `divmod` still returns the error.

## Consequences

- `sum += big`, `a < b`, `print(a * b)` and `"{b}"` work on `BigInt`.
- Breaking for 0.1 code: `BigInt.cmp` returns `Ordering`; a type named `Add`,
  `Sub`, `Mul`, `Div`, `Rem` or `Neg` must be renamed (garlic's
  `css.Neg` must be renamed when it moves its `.ail-version` past this).
- `std.bigint` needs a compiler with the new prelude: the seed compiler cannot
  build programs that use it until a seed carries this change.

## Rejected alternatives

- **Special-case `BigInt` in the checker, as `std.tensor` is.** Smaller, but
  a one-off rule an agent has to learn, and user types (vectors, money) still
  get no operators. Kaleb chose the general form.
- **A 128-bit integer (`i128`).** Still overflows, adds a sized type to every
  backend, and does not answer arbitrary sizes. Not in 0.2.
- **Mixed operands (`BigInt + int`).** Would need a trait parameter for the
  right operand and implicit conversion, which ADR 0077 rules out; write
  `bigint.from_int(n)`.
- **An indexing trait (`a[i]`).** Not needed for this change; left open.
