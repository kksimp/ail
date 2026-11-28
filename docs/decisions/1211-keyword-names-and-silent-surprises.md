# 1211. Certain keyword renames, `to_int()` used as an `int`, and `x ^ 2`

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)), at the designer's request to smooth keyword clashes and silent surprises. Diagnostics and fixes only, no syntax change and no keyword renamed
- **Topic:** 22 — Tools
- **Spec rules:** [lex.kw.1](../spec/02-lexical.md), [type.opt.5](../spec/04-types.md), [expr.bit.3](../spec/05-expressions.md), [fix.auto.1](../spec/22-tools.md)
- **Amends:** [ADR 1085](1085-keywords-as-member-names.md) (the rename fix is certain inside a function), [ADR 1203](1203-certain-first-draft-fixes.md)

## Context

Agents name variables and parameters `error`, `par`, `any`, `with`, `unique` and `todo`. The checker already names the
keyword and offers a rename, but the fix was a guess, so plain `ail check` left it for the agent to apply. Two other
first-draft habits stayed loud but vague or silent: `s.to_int()` used as an `int` (the help was the general
"may be absent" text, with no fix), and `x ^ 2` for a square, which is a bit xor and compiles.

## Decision

- **lex.kw.1 is certain inside a function.** A keyword declared in a function's parameters or body is renamed, with
  every later use through the end of the function, to the replacement the help names (`error` to `err`, `any` to
  `some`, `with` to `using`, `par` to `parallel`, `unique` to `distinct`, `todo` to `pending`). The name is made fresh:
  if any word of the file already spells it, `_` is appended until none does, so the rename never merges two
  bindings. A label before `:` inside a call's parentheses (`Outcome(error: e)`) is the field's name and is left alone.
  A second declaration of the same word in the function is covered by the first fix and has no report of its own, so
  two fixes never edit one span. A keyword declared in an item (a module constant, an `impl` or `trait` body) stays a
  guess: a rename there can't reach every use. The fix is marked by the title ending " in this function" (`sure.ail`).
- **`s.to_int()` where an `int` is wanted.** In an argument, a return or block value, a binding or struct field of type
  `int`, and as an operand of arithmetic or ordering beside an `int`, a direct `.to_int()` call on text gets help that
  says it returns an optional because the text may not be a number, and shows `else 0`, `else return` and `?`. The fix
  `(s.to_int() else 0)` is a guess, not certain: the default is the author's choice and a silent 0 can hide bad input.
- **`x ^ 2` and `x ^ 3`** on a variable draw a warning under `expr.bit.3` that names `x * x` and `x.pow(2)`.
  `@allow("expr.bit.3")` silences it. Literal on both sides is not reported.

## Options considered

- **Rename the keyword to another word (`any_`):** rejected; the existing replacements read as words and are stable in
  goldens. Only a clash adds `_`.
- **Make the `else 0` fix certain:** rejected; it changes failure into a value the author did not choose.
- **A new rule id for the `^` warning:** rejected; `expr.bit.3` already defines `^`, so no new rule enters the
  coverage lists.
- **Warn on every `^`:** rejected; xor with a mask or `1` is common and correct.
- **Make the rename certain for item-level keywords:** rejected; callers in other functions would still use the old
  name.

## Consequences

- `compiler/syntax/parse.ail` (`keyword_as_name`, `function_head`, `fresh_name`, `rename_uses`), `compiler/fix/sure.ail`,
  `compiler/check/toint.ail` and `xorpow.ail` (new), one line each in `expr.ail` and `call.ail`, three call sites changed
  from `mismatch` to `mismatch_value`.
- Mistakes pairs `keyword_param_error`, `keyword_error_label` and `to_int_as_int`; checks in the gate case `auto_fix_habits`.
- The runner regenerates the explain, fix and api goldens (`keyword_local_*` fix titles change) and builds and runs the
  gate; nothing was built in the cloud.
