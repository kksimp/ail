# 1139. Three more agent slips are certain fixes, and four help texts name the working form

- **Status:** Accepted
- **Date:** 2026-11-17
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer approved every fix of the full E14 review (2026-11-17) and holds to no language change to look like other languages (fixes, diagnostics and the card only)
- **Topic:** 3 — Diagnostics
- **Spec rules:** [fix.auto.1](../spec/22-tools.md), [type.tuple.5](../spec/04-types.md), [type.str.9](../spec/04-types.md), [mod.use.2](../spec/12-modules.md), [err.use.1](../spec/08-errors.md), [type.coll.3](../spec/04-types.md), [expr.key.1](../spec/05-expressions.md)
- **Amends:** [ADR 1130](1130-certain-fixes-applied-by-check.md)

## Context

The full E14 run (840 trials) left a tail of small failures that cost an agent one compile each, with a diagnostic
that either named no rewrite or named one that was not certain (`e14-review/full-run-review.md`). Each has one meaning.

## Decision

Three fixes become certain (`fix.auto.1`), so `check`, `build`, `run` and `test` write them:

1. **`for i, x in xs` over a list of non-tuples** (`type.tuple.5`, 5 versions): `for i, x in xs.enumerate()`. Only for
   a list (a `str.chars()` result included), two names, and an iterable that is a name, field, index or call chain.
2. **`n.pad_left(w, c)`, `pad_right`, `center` on an `int`** (`expr.name.1`, 5): `"{n}".pad_left(w, c)`, for a
   receiver that is a plain name. The help also names the format spec, `"{n:02}"`. Other receivers and other unknown
   method names get help only.
3. **`by: key` with a function value of one parameter** (`expr.key.1`, 1): `by: |x| key(x)`.

Four diagnostics change with no fix:

4. **`M.item` where `M` is a module of this program the file does not import** (`expr.name.1`, 5): when one loaded
   module matches, the help names `use M`; with two, it lists them. It was planned as a certain fix, but the Mac
   two-file project check (2026-11-17) never reached it, with or without the module loaded, so it ships as help only.
5. **`r := f()` directly followed by `if t := r`** (`err.use.1`, 4) reports once, with the help "bind and test in one
   step: `if t := f()` ... `else e`". It no longer offers `fn main() ! str` or `r := f()?`, and the follow-on
   `expr.if.5`, `expr.if.6` and unknown `err` are gone: a bound value of unknown type never adds an `else err` error.
6. **`[[] for i in 0..n]` and `[{} for ...]`** (`type.coll.3`, 3) show `[List[T]() for i in 0..n]` (or `Map[K, V]()`,
   `Set[T]()`); the old help, "use `[]` where a `List[...]` is expected", did not apply inside a comprehension.
7. **`opt.map(|x| ...)`** (`expr.name.1`, 3) stays an error; the help shows `v := e else d` then use `v`, `if v := e`
   with a block, and `e?.field else d`.

The `join_swap` fix (ADR 1138) now settles a string with escapes, `", ".join(["\"{t}\"" for t in tags])`, which gave
up before (2 versions); an interpolation that holds a quote is still left alone.

## Options considered

- **A fix for `opt.map`**: rejected; the closure body decides which of the three forms is right.
- **`use M` for a module that is not loaded** (found only on disk): rejected; the checker has no file access, and the
  loader's own help already names the import.
- **Drop only the `fix:` lines of the held result**: rejected; the follow-on errors came from the same bad shape and
  ask for opposite fixes.

## Consequences

- No language change. New mistake-corpus programs (`for_index_element`, `int_pad_left`, `by_function_value`,
  `join_escaped_quotes`) and cases (`diag_held_result`, `diag_comp_empty_literal`, `diag_opt_map_help`,
  `diag_for_enumerate`).
- A `T ! E` held in a name and tested on the next line reports one error.
