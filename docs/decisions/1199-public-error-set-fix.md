# 1199. err.type.3 names the inferred errors and offers the fix; float patterns point to a guard

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 3 — Diagnostics
- **Spec rules:** [err.type.3](../spec/08-errors.md), [expr.match.7](../spec/05-expressions.md)
- **Amends:** [ADR 0046](0046-inferred-error-sets.md)

## Context

A hands-on trial of AIL by an agent (2026-11-25) asked for a `fix:` line wherever the checker already knows the
answer. Its example: a `pub fn` written `!T` is rejected with "write `T ! E` with the error types", while the checker
has inferred the exact set (err.type.2) and prints none of it. The agent then reads the body to collect the names
the compiler holds. The same trial matched a float inside a variant, `Lit(1.0) =>`, and was told to use an `if`,
which does not fit inside a `match`.

## Decision

- **err.type.3.** After every body is checked, each err.type.3 report gets the inferred set of its function: help
  `its body fails with `Empty | TooLong`: write `! Empty | TooLong` as the return type`, and a fix that writes it.
  `!` gains the names; `!T` becomes `T ! E`. A type declared in the function's module is written bare, another
  module's with its module name (`fs.IoError`). A function whose set is empty keeps the old help.
- **The fix is a guess, not a certain fix (fix.auto.1).** `ail fix` writes it; a build prints it. The error set of a
  public function is part of an API, and ADR 0046 made the author state it.
- **expr.match.7.** The help for a float pattern is `bind the value and test it in a guard: `x if x == 1.5 =>`, or
  `Lit(v) if v == 1.5 =>``.

## Options considered

- **A certain fix for err.type.3:** rejected for now. The text is exact, but a build would then set public
  signatures no one read. Revisit if trials show the extra round costs more than the review is worth.
- **Help only, no fix:** rejected; the edit is mechanical and `ail fix` and editors can apply it.
- **Float literal patterns:** a language change (equality on floats in patterns was rejected in expr.match.7).

## Consequences

- `compiler/check/errfix.ail` (new), one line in `compiler/check/body.ail`, one help string in
  `compiler/syntax/parse.ail`, `compiler/explain/fixable.txt`.
- Mistakes pairs `pub_error_set_value`, `pub_error_set_unit`.
- Not built in the cloud. The runner builds and regenerates the fix, explain and api goldens.
