# 0168. `assert` and `assert_eq` are prelude functions allowed anywhere

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 28 Testing strategy
- **Spec rules:** `test.assert.1–2`

## Context

Tests need assertions that report both values on mismatch. Assertions are also
useful in ordinary code for stating invariants, and a failed one is a bug,
which AIL answers with a trap (ADR 0035).

## Options considered

### Option A — prelude `assert` and `assert_eq`, usable anywhere (chosen)
- **Pros:** one mechanism; matches `unreachable` (ADR 0084); ordinary code can
  state invariants.
- **Cons:** `assert_eq` is checked by the compiler rather than declared as
  `assert_eq[T Eq ToStr]` in `prelude.ail`, because the bootstrap's prelude
  can't yet trap with a chosen rule or show a value's text form.

### Option B — assertions only inside `@test` functions
- **Pros:** keeps ordinary code free of checks.
- **Cons:** extra rule to check and explain; no benefit to the reader.

## Decision

`assert(cond bool)` and `assert_eq(a, b)` are prelude names. Failure traps with
rule `test.assert.1`; `assert_eq` shows both values' text forms. They are
allowed outside tests, where the trap ends the program.

## Rationale

AI understanding decides: no context-dependent rule, and the message carries
both values so a failure can be read without rerunning.

## Consequences

- Programs can't declare their own `assert` or `assert_eq` (prelude names).
- When the prelude can express trap rules and trait-bounded text, `assert_eq`
  can move there with the signature `assert_eq[T Eq ToStr](a T, b T)`.

## Rejected alternatives

- Tests only: needless restriction with no benefit.
- A macro-style `assert(a == b)` that decomposes the expression: needs
  metaprogramming the language doesn't have yet.
