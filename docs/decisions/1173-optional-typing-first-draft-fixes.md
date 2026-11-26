# 1173. Optional typing and string fixes for first drafts

- **Status:** Accepted
- **Date:** 2026-11-21
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer's rule of no syntax change, with the checker, diagnostics and fixes only
- **Topic:** 3 — Diagnostics
- **Spec rules:** [type.opt.9](../spec/04-types.md), [err.prop.2](../spec/08-errors.md), [err.else.3](../spec/08-errors.md), [lex.str.2](../spec/02-lexical.md)
- **Amends:** [ADR 0468](0468-optional-none-locals-and-if-branches.md), [ADR 1172](1172-first-draft-build-fixes.md)

## Context

The $40 rerun's first-draft causes put optional and error typing at 6 of 24 non-green drafts, the one cause that held
its size across both runs. Three of them were `mut x := none` compared with `none` before its first assignment, which
type.opt.9 rejected as a use. Three compiler faults sat beside them: the `fix:` line for `\"` inside an interpolation
said `\{`, the certain fix of err.else.3 removed a name the block still used, and `io.print` hinted `io.eprint`.

## Decision

- **type.opt.9.** `x == none` and `x != none` on a local still waiting for its type are a `bool` and leave it
  waiting. Every other early use is still the error.
- **expr.name.1 on `T?`, err.prop.2.** The help quotes the receiver and writes the line: `(xs.last() else default).len()`,
  `v := x else return`. Help only, since the default is the author's.
- **err.else.3.** The certain fix that drops the name is withheld when the block reads it.
- **lex.str.2.** `\"` inside `{...}` reports that code in the braces writes plain quotes, shows the corrected code, and
  carries a certain fix removing the backslashes up to the closing brace, only when that closes the brace.
- **std.use.2.** `io.print`, `io.println` and `io.puts` are `print`, fixable.

## Rejected alternatives

- **Settle the type from a scan for the first assignment.** More cases (`best == none or v > best`), but the checker is
  one pass and lowering would need the same scan.
- **A certain fix for a method on `T?`.** The default value has no single reading.
- **Card lines instead.** The card already states these forms and the agents read it.
