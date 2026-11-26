# 1199. Fix lines the checker can already write: err.type.3's error set, a guard for float patterns, `check --fix`

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 3 — Diagnostics
- **Spec rules:** [err.type.3](../spec/08-errors.md), [expr.match.7](../spec/05-expressions.md), [fix.check.1](../spec/22-tools.md), [trait.op.1](../spec/10-traits.md)
- **Amends:** [ADR 0046](0046-inferred-error-sets.md), [ADR 1047](1047-ail-check-fix.md), [ADR 1175](1175-more-first-draft-certain-fixes.md)

## Context

A hands-on trial of AIL by an agent (2026-11-25) asked for a `fix:` line wherever the checker already knows the
answer. Its example: a `pub fn` written `!T` is rejected with "write `T ! E` with the error types", while the checker
has inferred the exact set (err.type.2) and prints none of it. The agent then reads the body to collect the names
the compiler holds. The same trial matched a float inside a variant, `Lit(1.0) =>`, and was told to use an `if`,
which does not fit inside a `match`.

The second pass of that trial (main 78e27c83) found three more of the same kind. `ail check --fix` wrote none of five
certain fixes while two syntax errors remained, though plain `ail check` wrote them. The native program run alone,
`.native/ail check FILE`, failed with a `JsonError` trace: its `check` read only program dumps, and the launcher
script maps `check` to `check-program`. And `if c then "" else " " + xs.join(" ")` kept its trait.op.1 with no
rewrite, because ADR 1175 left every `+` after `else` alone.

## Decision

- **err.type.3.** After every body is checked, each err.type.3 report gets the inferred set of its function: help
  `its body fails with `Empty | TooLong`: write `! Empty | TooLong` as the return type`, and a fix that writes it.
  `!` gains the names; `!T` becomes `T ! E`. A type declared in the function's module is written bare, another
  module's with its module name (`fs.IoError`). A function whose set is empty keeps the old help.
- **The fix is a guess, not a certain fix (fix.auto.1).** `ail fix` writes it; a build prints it. The error set of a
  public function is part of an API, and ADR 0046 made the author state it.
- **expr.match.7.** The help for a float pattern is `bind the value and test it in a guard: `x if x == 1.5 =>`, or
  `Lit(v) if v == 1.5 =>``.
- **fix.check.1.** `ail check --fix` writes the certain fixes of the errors left after its own rounds, in FILE as
  in the other files. Its rounds stop at a syntax error; the checker still reports the declarations that parsed.
- **native.build.3** says to run the launcher or set `AIL_ROOT`. (`check FILE` in the native program alone landed on
  main separately while this was in flight.)
- **trait.op.1.** The `+` rewrite also starts after an `else` that closes a `then` on its line (the code before it
  has more `then` than `else`, strings not counted). Any other `else` may be the fallback of `a else b` and is left.
  An operand whose braces are all interpolations inside its own string literals (`xs.map(|t| "#\{t}").join(" ")`)
  can now sit in the interpolation; it was refused for holding a brace.

## Options considered

- **A certain fix for err.type.3:** rejected for now. The text is exact, but a build would then set public
  signatures no one read. Revisit if trials show the extra round costs more than the review is worth.
- **Help only, no fix:** rejected; the edit is mechanical and `ail fix` and editors can apply it.
- **Float literal patterns:** a language change (equality on floats in patterns was rejected in expr.match.7).

## Consequences

- `compiler/check/errfix.ail` (new), one line in `compiler/check/body.ail`, one help string in
  `compiler/syntax/parse.ail`, `compiler/explain/fixable.txt`.
- `compiler/main.ail` (`check`, `--fix`), `compiler/fix/rewrite.ail` (`closes_then`).
- Package case `pub_inferred_fix` (err.type.3 fires only in a package with modules, so no mistakes pair); mistakes
  pair `string_plus_after_then_else`.
- Left: the expr.match.3 warnings `ail run` prints.
- Not built in the cloud. The runner builds and regenerates the fix, explain and api goldens.
