# 1138. Four string slips are certain fixes

- **Status:** Accepted
- **Date:** 2026-11-16
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer's 0.2 goal of fewer agent iterations than C, and his rule of no language change to look like other languages (fixes, diagnostics and the card only)
- **Topic:** 3 — Diagnostics
- **Spec rules:** [fix.auto.1](../spec/22-tools.md), [lex.str.1, lex.str.2](../spec/02-lexical.md)
- **Amends:** [ADR 1130](1130-certain-fixes-applied-by-check.md), which left the string-escape rules out of the certain set

## Context

The E14 pilot's held-out task `invjson` is the one task that prints literal `{` and `}` (JSON). All four AIL trials
failed to compile at least once, and all seven failed versions were compile errors: 5 from a literal brace in a string
(`"{"`, `"    {\"sku\": ..."`) or from the `\}` the agent then wrote, one from `", ".join(xs)`, one from
`"\\u00" + k.to_hex().pad_left(2, '0')`. Every trial passed 22 of 22 cases once it built. In each case the checker
already printed the right fix; it was not certain, or its rewrite gave up (analysis in
`e14-review/heldout-losses.md`).

## Decision

Four fixes become certain (`fix.auto.1`), so `check`, `build`, `run` and `test` write them:

1. **A `\{` that only text can follow.** A `{` in a string is a literal brace when `\"` follows it (an interpolation
   never starts with a backslash), or, with no `}` closing it on the line, when the string's own closing `"`, a space
   or `\n` follows. The fix inserts the backslash and the string is lexed on, so its other braces and the errors after
   it are found in the same run. `{name` with no `}` is a forgotten closer and stays a guess; `{" ".repeat(n)}` is
   still an interpolation of a string.
2. **`\}` as `}`.** `}` is never escaped; the rewrite has one meaning.
3. **`sep.join(xs)` as `xs.join(sep)`**, when the receiver is a `str` and the one argument is a list that is a name,
   a call chain or a list literal (anchor `join_swap`).
4. **The `+` on strings rewrite** now settles a plain character literal inside brackets, so the form the card teaches,
   `"x" + s.pad_left(2, '0')`, gets its interpolation fix. An escape, a quote or a brace in the character is still left
   alone.

Certainty of a parser fix is the rule's (`SURE_SYNTAX_RULES`) or, for `lex.str.1` and `lex.str.2`, the fix's title
(`sure_syntax_fix`), because those rules also carry guesses. The core card gains one line with a JSON string and the
rewritten `join` spelling; the help of the brace diagnostic says `}` is never escaped.

## Options considered

- **Accept `\}` as an escape** (chosen against): removes the same failures, but adds a second spelling to the
  language. Rejected: no language changes to look like other languages.
- **Keep every string-escape fix a guess** (the ADR 1130 position): rejected; 5 of the 7 failed versions came from it,
  and each rewrite has one meaning.
- **Make every unclosed `{` certain**: rejected; `"{n"` is a forgotten closer, and a certain wrong rewrite hides it.

## Consequences

- No language change; new `fix.auto.1` cases and mistake-corpus programs (`json_brace_text`, `json_brace_escaped_close`,
  `string_plus_pad_left*`, `join_python_*`), and a gate scenario (`auto-fix string slips`).
- A brace followed by a space or `\n` and never closed is now escaped, never reported as a missing `}`.
