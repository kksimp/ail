# 1129. `T ! str`: plain-text errors

- **Status:** Accepted
- **Date:** 2026-11-15
- **Decided by:** designer (Kaleb approved on 2026-11-15, "Yes", to add it to 0.2); details by the AI under [ADR 0076](0076-delegated-decisions.md)
- **Topic:** 8 — Error handling; AI understanding
- **Spec rules:** err.str.1-3, [err.decl.2](../spec/08-errors.md) (amended)
- **Amends:** [ADR 0120](0120-error-keyword.md) ("only error types are errors")

## Context

In the E13 agent benchmark agents wrote `fn parse(s str) int ! str` and
`return "bad input"` by habit (Rust's `Result<T, String>`, Go's `error`).
That was `err.decl.2`, a whole fix iteration on a first draft, in a language
where the metric is iterations to working code. The declared fix (an `error
enum` with a message) is right for a library, but a command-line tool wants
one line.

## Options considered

### Option A: `T ! str` means `T ! strerr.StrError` (chosen)
`std.strerr` is a module written in AIL with one `error struct StrError
"{self.why}"`. A loader pass (`compiler/load/strerr.ail`) writes the type for
a function declared `T ! str` and wraps `return "literal"` and `return
"interpolation"` as `StrError(why: ...)`. The module loads when a file has `!
str` (std.use.4).
- **Pros:** no checker or backend change, since everything after the loader
  sees an ordinary error struct; `?`, `else |e|`, traces and `print(e)` work
  unchanged; `ail fmt` keeps the text the user wrote.
- **Cons:** only literals and interpolations are wrapped; a `str` variable
  needs `strerr.StrError(why: v)` (`err.str.3` says so).

### Option B: coerce any `str` value in the checker and the backend
- **Pros:** `return text` works too.
- **Cons:** a type-directed conversion in checker and native lowering, two
  places that must agree; ambiguous when the success type is `str`.

### Option C: keep rejecting it
- **Cons:** the first-draft failure stays; rejected, it is the measured loss.

## Decision

Option A. `T ! str` alone only (not `! str | E`, not `! int`); when `T` is
`str` no return is wrapped (`err.str.2`). `!str` with no success type before
the `!` keeps meaning success type `str` with inferred errors.

## Consequences

The language cards say `T ! str` is allowed; the std index lists `strerr`.
Follow-up if agents still write `return text`: Option B.
