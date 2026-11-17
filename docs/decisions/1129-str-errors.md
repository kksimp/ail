# 1129. `T ! str`: plain-text errors

- **Status:** Accepted
- **Date:** 2026-11-15
- **Decided by:** designer (Kaleb approved on 2026-11-15, "Yes", to add it to 0.2); details by the AI under [ADR 0076](0076-delegated-decisions.md)
- **Topic:** 8 — Error handling; AI understanding
- **Spec rules:** err.str.1-3, [err.decl.2](../spec/08-errors.md) (amended)
- **Amends:** [ADR 0120](0120-error-keyword.md) ("only error types are errors")
- **Amended by:** [ADR 1135](1135-self-read-store-and-text-failure-fixes.md) (a `str` tail's certain fix; one diagnostic for the known gap), [ADR 1140](1140-standard-text-failure.md) (the known gap's form: the standard `Fail`)

## Context

In the E13 agent benchmark agents wrote `fn parse(s str) int ! str` and
`return "bad input"` by habit (Rust's `Result<T, String>`, Go's `error`).
That was `err.decl.2`, a whole fix iteration on a first draft, in a language
where the metric is iterations to working code. An `error enum` with a
message is right for a library, but a tool wants one line.

## Options considered

### Option A: `str` is a failure type; the checker and backend decide by type (chosen)
In a function declared `T ! str`, `return V` with `V` of type `str` is the
failure, any other `return` is the success value. The error has no error
type: a handler binds `e` as a `str` (the text slot the backend already has
for errors of no single type), and `?` passes any error on as its text.
- **Pros:** one rule, decided by type, so `return "x"`, `return why` and
  `return e.to_str()` all work; no new std module; matches what agents write.
- **Cons:** the last expression of a body is never a failure (`return` is
  needed); `T` must be written (`!str` alone keeps its meaning, a `str`
  result with inferred errors); the error loses its type, so it can't be
  matched on.

### Option B: a std `StrError` struct, with literals wrapped by the loader
- **Pros:** typed error value.
- **Cons:** wraps only literals and interpolations, so `return why` fails
  and agents trip on the difference (rejected after review).

### Option C: keep rejecting it
- **Cons:** the measured first-draft failure stays.

## Decision

Option A. `T ! str` alone (not `! str | E`, not `! int`); `str ! str` is
`err.str.2`. A function declared `T ! str` accepts any error with `?` (err.str.3).

## Consequences

Changes: `Checker.resolve_ret`, `return_stmt`, `add_errors`; native
`lower_strerr.ail` and `error_ty_of`. Cards say `T ! str` is allowed.
Known gap: a failing function with no success value can't be written `! str`
(it parses as a `str` result); use an error type there.
