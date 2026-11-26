# 0279. `?` on an optional fails with `Absent` where errors are inferred, and tests can fail

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Errors; testing; token efficiency
- **Spec rules:** err.prop.2, err.prop.5, test.attr.4
- **Amended by:** [ADR 1181](1181-optional-try-fails-with-text.md) (`?` on an optional fails with text in a `T ! str` or `Fail` function)

## Context

The brief asked for a shorter form of `x := opt else return`, the "or give up"
case. Its commonest home is a test: `e := parse_line(...) else return` makes a
test pass silently when the value is missing, which hides the very bug the
test checks. In `main() !` and other `!T` functions, the same give-up needs
`else return SomeError(...)` or a helper, where Python raises `KeyError` on
`d["k"]`. `?` on an optional worked only in functions returning optionals
(`err.prop.2`).

## Options considered

### Option A — `?` on an optional fails with `Absent` in inferred-error functions; tests are `!` (chosen)
- **Pros:** one familiar symbol, `?`, means "or give up" everywhere it can; a
  missing value in a test fails it, with the position, instead of passing;
  −8.8 tokens on the agent corpus from tests alone, −16.8 with `main`.
- **Cons:** an error that carries no data beyond its trace and context.

### Option B — `else fail` or `else return` sugar
- **Pros:** explicit.
- **Cons:** a new keyword form for what `?` already says; still silent in
  tests unless written with care.

### Option C — keep `else return`
- **Cons:** tests that pass on the path they don't check.

## Decision

In a function whose error set is inferred (`!T` or `!`) and that doesn't return
an optional, `e?` on an optional fails with the prelude's
`error struct Absent "a value is absent"`, which joins the inferred set;
`e? "context"` attaches context. Functions that state their errors still need
`else`. A `@test` function written without a return type is checked as `!`, so
`?` works in it; calling a test stays an error (`test.attr.3`).

## Rationale

Tokens are a tie (−0.5% to −0.7%), so AI understanding decides (criterion 2):
a test can no longer pass by leaving early, and the program fails at the `?`
with its position and context, which is what an agent needs to fix it. Stated
error sets are unchanged, so a library's contract never gains `Absent`
silently.

## Consequences

- `ail test` reports `error: a value is absent at LINE:COL` for a missing
  value.
- Examples and the agent corpus use `?` in tests and in `main`.

## Rejected alternatives

- `else fail` / `else return` sugar (B): a second spelling of `?`.
- `Absent` in stated error sets too: would let a declared `! ParseError`
  function fail with a type its callers don't expect.
- Keeping `else return` in tests (C): silent passes.
