# 1140. The standard text failure `Fail`

- **Status:** Accepted
- **Date:** 2026-11-17
- **Decided by:** Designer (a standard error type for failing with text, no new syntax: decision card, 2026-11-17);
  the name, the `?` rule and the details by AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Amended by:** [ADR 1147](1147-calc-miss-fixes.md) (two gaps in the certain fix), [ADR 1181](1181-optional-try-fails-with-text.md) (`?` on an optional fails with text)
- **Topic:** 8 — Error handling; 3 — Diagnostics
- **Spec rules:** [err.fail.1-3, err.str.2, err.str.4](../spec/08-errors.md)
- **Amends:** [ADR 1129](1129-str-errors.md) (its known gap: a function with no result that fails with text now has a
  form), [ADR 1135](1135-self-read-store-and-text-failure-fixes.md) (err.str.4 gets a certain fix and a one-line help),
  [ADR 1130](1130-certain-fixes-applied-by-check.md) (one more certain fix)

## Context

The E14 confirmation run (main 1636ca45, `e14-review/confirm-run-review.md` sections 2 and 4) left the calculator
task at 2.33 iterations against C's 1.17. About 20 of its 34 failed AIL versions still wrote `T ! str` where there is
no result: `fn expr(mut self) ! str` with `return "syntax"` (about 11), and `str ! str` (about 6). ADR 1135 made each
one error, with a help that teaches a four-line `error struct Bad` declaration; agents read it and still needed a
second iteration. The designer chose a standard error type, with no new syntax.

## Decision

1. **The type (err.fail.1).** The prelude declares `error struct Fail "{msg}"` with the one field `msg str`, in its
   own prelude module (`## @module fail`), read when the program names `Fail`. It needs no `use`. `fn f() ! Fail`,
   `fn g() str ! Fail`, `return Fail(msg: "...")`; a handler's `e` prints as the text and has `e.msg`. It is an
   ordinary error type: union member, `match` arm, `fn main() ! Fail`. Native lowering adds the module to the
   program's sources as it does `json` and `seq`, so it builds, returns, propagates and prints on the three hosts
   with no lowering code of its own.
2. **A program's own `Fail` wins.** A program that declares a struct, enum or error type named `Fail`, in any of its
   modules, gets no prelude `Fail`: the checker reads the module only when no file declares the name, and native
   lowering drops the prelude module when any module declares `Fail` (and, as before, any prelude module whose type
   the root module declares, now counting `error struct` and `error enum` lines too). So a project module's own
   `Fail` is not visible unqualified in the root file, in both.
3. **The prelude's types yield to a bare variant of the expected enum.** Where an enum with a variant `Fail` is
   expected, `Fail(...)` builds the variant, as native lowering already resolved it (`call_variant` tries the
   expected enum first). Without this, a program with `enum Value` / `Fail(err E)` would stop checking.
4. **`?` turns text into a `Fail` (err.fail.3).** In a function whose declared errors name `Fail` and not `str`, `?`
   on a call whose undeclared error is text (a `T ! str` call, or an inferred set holding `str`) passes it on as
   `Fail(msg: TEXT)`; the display text and the trace stay, the error value and tag become the `Fail`'s. Lowering
   (`compiler/mir/lower_failconv.ail`, one hook in `propagate_result`) tests the tag on the failing path: a tag of
   none of the declared types (text has tag 0; an error a `T ! str` function passed on keeps its own) is rewrapped.
   Other undeclared errors stay err.prop.3, whose help adds `v := ... else e`, then `return Fail(msg: "{e}")`.
   The review of the confirmation trials found the calculator's no-result functions all call `T ! str` functions
   with `?` (`toks := tokenize(line)?`), the mix the card teaches; strict `?` turned the fixed function into three
   err.prop.3. The other direction already holds (err.str.3).
5. **err.str.4's certain fix (err.fail.2).** `! str` becomes `! Fail`, and each `return TEXT` of the function's own
   body (not a local function or function literal) becomes `return Fail(msg: TEXT)`. Certain because the body
   shows no result was meant, so each text returned is the failure. It is withheld when any `return` gives a
   non-`str` value or a value whose end the fix can't find on its line, the body ends with a `str`, the function
   passes on an error other than text or `Fail`, it is used as a function value, a call uses its `str` result
   (`x := f() else ...`, `f()?` inside an expression, a handler that reads its error other than as a whole `{e}` of
   a string: `return e`, `errs.push(e)`), or the program declares its own `Fail`. A statement `else` on a call of
   such a function is part of the same mistake: its err.else.4 (whose `_ := ` fix would fight this one) is dropped.
   Calls are recorded for every `! str` function as they are checked, and the decision is made once all bodies are
   checked, so a caller above its callee, or two mutually recursive functions, get the same result.
6. **Helps.** err.str.4 and err.str.2 each show one line: `! Fail` (or `str ! Fail`) and `return Fail(msg: "...")`.
7. **Card.** The four-line `error struct Bad` example becomes one line in the full card, the core card and the
   how-to "fail with text": with a result other than `str`, `T ! str` and `return TEXT`; otherwise `Fail`.
8. `T ! str` (ADR 1129) and `! str` meaning "a `str` result with inferred errors" are unchanged. There is no
   positional `Fail("x")`; the existing type.struct.7 fix names the field when an agent writes it.

## Name

Measured with `tools/tokbench` on a representative program (a method `! X`, a `str ! X` function, a handler, three
`return X(msg: ...)`), mean of five tokenizers:

| Name | Tokens | vs `Fail` |
|---|---|---|
| `Fail` | 104.8 | — |
| `Failure` | 105.8 | +1.0% |
| `TextError` | 109.8 | +4.8% |

`Fail` and `Failure` tie on tokens (under 2%). `Fail` reads as the declaration says it: `! Fail`, "fails with",
matching the card's and diagnostics' wording ("fail with text"), and it is the working name the designer saw.
No clash: no std item, keyword or prelude name is `Fail`; the effect `fail` is lowercase and never written
(eff.infer.2). `TextError` is longer and suggests a parse error of text.

## Options considered

- **New syntax** (`! str` meaning "no result" when nothing precedes `!`): rejected by the designer; no syntax change.
- **A `Fail` declared in a std module** (`use std.err`): rejected; the point is a form an agent writes without
  knowing more, and a missing `use` is another iteration.
- **`?` stays strict for text too** (option a, this ADR's first draft): rejected after review, see 4; it made the
  certain fix's result fail to check in the calculator's own shape.
- **`?` converts every error to `Fail`**: rejected; text loses nothing as a `Fail`, but a typed error would lose
  its type, which a union (`! Fail | IoError`) is written to keep.
- **The prelude type always beats a variant** (as user types do): rejected; it breaks existing programs with a
  `Fail` variant (the tokbench corpus has one) and disagrees with native lowering.
- **Certain fix without looking at the calls**: rejected; `print(f() else "x")` would stop checking after it.

## Consequences

- Prelude: `tools/native/prelude.ail` (module `fail`), regenerated `compiler/mir/prelude_embed.ail`; loaded by
  `compiler/load/load.ail`, `compiler/mir/lower_prelude.ail`, `compiler/main.ail` (`mentions_prelude`).
- Checker: `compiler/check/textfail.ail`, hooks in `strfail.ail`, `body.ail`, `call.ail`, `expr.ail`, one state
  field in `checker.ail`. Native: `compiler/mir/lower_failconv.ail`, one line in `lower_err.ail`, `rt.err_as_fail` in
  `compiler/rt/native/err.ail`, the `Fail` rule in `prelude_shadow.ail`.
- Tests: `tests/native/run/fail_type.ail`, `fail_main.ail`, `fail_own.ail`, `fail_variant.ail`, `fail_text_mix.ail`;
  `tests/cases/fail_prop_errors.ail`; mistakes `str_no_result_method`, `str_no_result_nested`,
  `str_no_result_local_fn`, `str_no_result_mixed`, `str_no_result_callers_first` (callers first, mutual recursion),
  `str_no_result_error_used` (no certain fix).
- The explain golden for err.str.2, err.str.4 and the new rules needs refreezing, and llms-full.txt regenerating.
