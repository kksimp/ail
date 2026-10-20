# 0198. Run a test with parameters as a property

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Testing, [ADR 0167](0167-test-functions-and-ail-test.md)
- **Spec rules:** `test.prop.1`, `test.prop.2`, `test.prop.3`

## Context

An AI author writing a test usually knows an invariant ("addition commutes") but not good inputs. Example tests cover the cases the author thought of; generated inputs cover the rest. Lane 7 made `@test` take no parameters (`test.attr.2`), so a parameter is free syntax to give this meaning.

## Options considered

### Option A — parameters make a property (chosen)
- **Pros:** zero new syntax; the parameter list is the generator spec; a failing input pastes straight into a plain test. Tokens: `@test fn commutes(a int, b int)` costs the same as an example test plus the types.
- **Cons:** the checker must know which parameter types can be generated.

### Option B — a `property(...)` helper called inside a test
- **Pros:** no checker change.
- **Cons:** needs closures with typed generator arguments, which the bootstrap lacks; far more tokens per property.

### Option C — a separate `@prop` attribute
- **Pros:** explicit.
- **Cons:** a second attribute for what the parameters already say (+1 token per test, no information).

## Decision

A `@test` with parameters is a property. `ail test` runs it on generated inputs, 100 by default (`--cases N`), from a seed that is printed and can be replayed (`--seed S`). Parameter types must be `int`, `float`, `bool`, `char`, `str`, `List`, `Map`, `Set`, optionals, and non-generic structs and enums made of those; anything else is `test.prop.2` with the part that can't be generated.

## Rationale

ADR 0076 criterion 1 (tokens) and 2 (AI understanding): option A is the shortest and has no second way to write a test. Checking the types at compile time makes a bad parameter an error in one pass, not a failure at run time.

## Consequences

- A test file can mix example tests and properties.
- Generated floats are finite and ints stay within about ±2^31, so arithmetic properties don't fail on overflow noise; a property that needs extremes writes them as example tests.
- Generic structs and `any Trait` parameters can't be generated yet.

## Rejected alternatives

- Option B: needs language features that do not exist yet and costs more tokens.
- Option C: redundant.
