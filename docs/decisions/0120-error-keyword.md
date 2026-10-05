# 0120. Error types are declared with the `error` keyword

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** Designer
- **Topic:** 8 — Error handling
- **Spec rules:** [err.decl.1–3](../spec/08-errors.md), [lex.kw.1](../spec/02-lexical.md)
- **Partly resolves:** OQ-030 (which types may be errors)
- **Amends:** [ADR 0114](0114-reserved-keywords.md) (adds `error` to the reserved keywords)

## Context

`T ! E` and `?` were specified, but not which types may be errors.

## Options considered

Tokens are the mean of five tokenizers on a small parser example. They
differ only once per error type, so on real programs the gap is about 1–2%.

| Option | Tokens | Notes |
|---|---|---|
| **`error enum ParseError` (chosen)** | 77.8 | Explicit at the declaration. Reserves `error` |
| `impl Error for ParseError` | 83.0 | Reuses traits, and allows custom messages. The marking can be far from the declaration |
| Any type, no marking | 76.8 | `return x` is ambiguous when success and error types overlap (`fn f() str ! str`), and `str` errors can't be matched precisely |

## Decision

1. `error struct Name` and `error enum Name` declare **error types**. In
   every other way they are ordinary structs and enums.
2. Only error types can be errors: the `E` in `T ! E`, the members of an
   error union, and the members of an inferred error set.
3. In `T ! E`, `T` must not be an error type. Where a `T ! E` value is
   expected, a value of type `T` is the success value, and a value of an
   error type in `E` is the error. So `return Empty` in a function
   returning `int ! ParseError` returns an error, with no extra marker.
4. `error` is a reserved keyword.

## Consequences

- Readers and tools can list a program's error types from declarations alone.
- Custom error messages come later, with the formatting trait (ADR 0088).
  Until then an error is shown in its default form (ADR 0122).

## Rejected alternatives

- **`impl Error for T`:** six more tokens per error type, and the type's
  declaration doesn't say it is an error.
- **Any type:** ambiguous returns, and errors can't be told apart from data.
