# 0122. Error unions are written `A | B`; `?` needs a matching error type; failures must be handled; the default error form

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 8 — Error handling
- **Spec rules:** [err.type.5–6](../spec/08-errors.md), [err.prop.3](../spec/08-errors.md), [err.use.1](../spec/08-errors.md), [err.text.1](../spec/08-errors.md)
- **Partly resolves:** OQ-030 (union syntax, conversion, recursion)

## Deciding criterion

Token usage **ties** on every point. **AI understanding** decides:
- `|` is the union notation AIs know (TypeScript, Python typing, Rust
  patterns).
- No implicit conversion between error types, so `?` never changes what an
  error is.
- An unhandled failure is always a compile error, never silently dropped.

## Decision

1. **Unions.** An explicit set of error types is written with `|`:
   `Config ! IoError | ParseError`. Order doesn't matter.
2. **Propagation.** `e?` in a function with an explicit error type needs
   every error type `e` can fail with to be in that type. Otherwise it's a
   compile error that names the missing types. With an inferred `!T`, they
   are added to the set.
3. **Recursion.** Inferred error sets are the smallest sets consistent with
   every function body: mutually recursive functions share errors, and
   recursion adds nothing by itself.
4. **Handling.** A `T ! E` value must be handled, with `?` or `else`, or
   returned from a function whose result can hold it. It can't be discarded
   or used as a `T`.
5. **Default form.** Until the formatting trait exists, an error value is
   shown as its type, then its variant for an enum, then its fields in
   parentheses: `ParseError.BadNumber(text: "x1")`. Strings are quoted. This
   form appears in error reports and wherever an error is interpolated
   (`"{err}"`).

## Rejected alternatives

- **`(A, B)` or `Either[A, B]`:** reads like a tuple or a two-case enum.
- **Implicit conversion between error types in `?`** (Rust's `From`):
  hidden code at every `?`.
- **Allowing a discarded `T ! E` with a warning:** an ignored failure is
  almost always a bug.
