# 0126. Every type gets `==` and a text form automatically

- **Status:** Accepted; amended by [ADR 0133](0133-automatic-hash.md) (`Hash` is a third automatic trait)
- **Date:** 2026-10-05
- **Decided by:** Designer
- **Topic:** 10 — Traits; 4 — Types
- **Spec rules:** [trait.auto.1–4](../spec/10-traits.md), [err.text.1](../spec/08-errors.md)
- **Amends:** [ADR 0073](0073-explicit-trait-impls.md) (an exception for two built-in traits), [ADR 0122](0122-error-unions-and-reports.md) (the default error form becomes every value's default form)
- **Evidence:** `tools/tokbench/corpus/r7/e*.ail`, `t*.ail`

## Context

Enums had no `==` ("compare them with `match`"), and only `int`, `bool`,
`str` and error values could be printed or interpolated. Comparing a
direction or printing a point took a `match` or hand-written text. AI agents
expect `c == Red` and `print(p)` to work, as they do in Swift (enums), Go
(`==` on structs, `%v`) and Python dataclasses.

## Options considered

Tokens are the mean of five tokenizers. "Equality" compares an enum and a
struct; "text" prints a struct and an enum.

| Option | Equality | Text |
|---|---|---|
| Today: `match` and field-by-field code | 200.2 | 90.8 |
| **Automatic (chosen)** | **191.4** (−4%) | **51.8** (−43%) |
| Opt in with an empty `impl Eq for T` | 202.2 | 64.6 |
| Opt in with `@derive(Eq)` | 202.2 | 63.8 |

## Decision

1. `==` and `!=` work on every type whose parts support them: `int`, `bool`,
   `str`, lists and optionals of such types, and structs and enums (error
   types included) whose fields all do. Equality is structural: the same
   variant with equal fields, and lists element by element.
2. Every value has a text form, used by `print`, interpolation and error
   reports. A struct prints as `Point(x: 1, y: 2)`, a variant as `Dir.North`
   or `Shape.Circle(r: 1)`, a list as `[1, 2]`, and an absent optional as
   `none`. Strings inside another value are quoted.
3. These are the standard traits `Eq` and `ToStr` (ADR 0127), implemented
   automatically. Writing `impl Eq for T` or `impl ToStr for T` replaces the
   automatic implementation. A type parameter bounded by `Eq` accepts any
   type that supports `==`.
4. Ordering (`<`) isn't automatic: `impl Ord for T` opts in (ADR 0127).

## Rationale

Fewest tokens on common code, and the behaviour AI agents already expect.
The exception to ADR 0073 is limited to two built-in traits whose meaning is
fixed by the language, so `x == y` and `"{x}"` never depend on which
declarations happen to exist. Ordering stays explicit because it isn't always
meaningful.

## Consequences

- `print` and interpolation accept any value.
- A type that declares its own `eq` or `to_str` method outside an `impl` is
  an error, with help pointing at the `impl` form (`trait.auto.4`).
- Future types that can't be compared (such as functions) will lack `Eq`.

## Rejected alternatives

- **Empty `impl` per type:** a line per type for behaviour nearly every type
  wants, and `c == Red` fails until it's added.
- **`@derive(...)`:** the same cost, plus an attribute to learn.
- **Status quo:** the most tokens, and a common AI first-attempt failure.
