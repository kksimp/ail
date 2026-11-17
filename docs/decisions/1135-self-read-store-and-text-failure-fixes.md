# 1135. A `{}` typed by a store that reads it; `T ! str` tail fix and no-result diagnostic

- **Status:** Accepted
- **Date:** 2026-11-17
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer approved every fix of the E14 review on 2026-11-17 ("Yes approve all"), and his rule of no language change to look like other languages (fixes, diagnostics and the card only)
- **Topic:** 3 — Diagnostics; 8 — Error handling
- **Spec rules:** [type.coll.13](../spec/04-types.md), [err.str.1, err.str.4, err.else.5](../spec/08-errors.md)
- **Amends:** [ADR 1129](1129-str-errors.md) (its known gap now has one diagnostic; a `str` tail gets a certain fix), [ADR 1130](1130-certain-fixes-applied-by-check.md) (two more certain fixes)

## Context

In the full E14 run (main 511d5efa), two shapes caused 48 of the 138 failed AIL versions:

- `mut tot := {}` then `tot[key] = (tot.get(key) else d) + v` (18, all one change task). Typing the value read the
  still-untyped local, which reported it (into a discarded quiet pass) and made it unknown, so the store was then
  reported as "a list index must be `int`, found `str`": a message about a list the program never had.
- `T ! str` (30, one task): `fn f() ! str` meant as "no result, fails with text" (a bare `return`, a body ending in a
  statement, `return "syntax"`), which gave one error per `return` and branch; a body or branch ending in a bare
  `str` meant as the failure; `str ! str`; and `f(x) else err` as a statement on a `T ! str` call.

## Decision

1. **Self-reading store (type.coll.13).** In `m[k] = v` where `v` names the pending `{}` local `m`, `v` is checked
   with `m` typed `Map[K, ?]`. When `v`'s type is still unknown, a guess is taken from the part that doesn't read `m`
   (the fallback of `m.get(k) else d`, the default of `m.get_or(k, d)`, the other operand of an arithmetic operator,
   a branch of an `if`) and confirmed by checking `v` again with `m` typed `Map[K, guess]`. A known `V` settles `m`
   to `Map[K, V]`; otherwise `m` stays pending and the store reports type.coll.13.
2. **No list-index message on an unknown receiver** (type.coll.4): it was already reported.
3. **A `{}` reported as untyped** gets the certain fix `Map[K, V]()` when its first store has a known key and value.
4. **`str` tail in `T ! str` (err.str.1).** A `str` ending the body, or a branch of an `if` or `match` in tail
   position, when `T` is not `str`: reported once, with the certain fix that writes `return` before it (the type
   decides it, as ADR 1129 says of `return`). A one-line `if` gets no fix: its branches can't hold a `return`.
5. **`! str` with no result meant (err.str.4).** A bare `return` or a body that ends with a statement reports one
   error at the result type, with help showing an error type (`error struct Bad "{msg}"`, field `msg str`,
   `! Bad`, `return Bad(msg: "...")`) or a result type `T ! str`; the follow-on `expr.block.2` and `expr.if.4` in
   that function are dropped. No fix: which form is wanted is a choice.
6. **`str ! str` (err.str.2)** help shows the error-type form for the function's own name.
7. **Statement `else` on a value result (err.else.5, reported as err.else.4).** The help says the value is unused,
   `_ := ... else ...`; the fix `_ := ` is certain when the `else` part leaves or gives a fallback of type `T`.
   err.else.2's message gains a help naming both ways out.
8. **`!T` and `!` in messages** read "`T` with inferred errors (`!T`)" and "no value, with inferred errors (`!`)", so
   "expected `!`, found `str`" no longer prints.

## Options considered

- **Type `{}` from the first store's value only when it doesn't read the map** (status quo): rejected; it is the
  form agents write for counting and summing, and it failed 18 versions.
- **Give `! str` a meaning "no result, fails with text"** (a spelling change): not decided here; the designer decides
  the spelling later (ADR 1129's known gap stays). Until then, one clear error.
- **Treat a `str` tail as the failure silently**: rejected; the last expression of a body is its result (err.str.1),
  and a rewrite that the file shows keeps that rule visible.
- **Certain `_ := ` for every statement `else`**: rejected; with a block that neither leaves nor gives a `T` the
  rewrite gives err.else.2, a second iteration.

## Consequences

- Checker: `compiler/check/selfread.ail`, `compiler/check/strfail.ail`, hooks in `body.ail`, `call.ail`,
  `resolve.ail`, two state fields in `checker.ail`. Native lowering types the table the same way
  (`compiler/mir/lower_selfread.ail`, one hook in `lower_map.ail`): the guessed part is lowered into a block that
  nothing reaches, so its type is known before the value that reads the table. No language change.
- `coll_op_assign_first_use_errors` now reports only type.coll.13 (no list-index error on an unknown receiver).
- Cases `map_self_read_store`, `str_no_result`, `result_value_unused_block`; `pending_map_value_reads_it` now reports
  type.coll.13; mistakes `str_tail_failure`, `str_branch_failure`, `result_value_unused`, `map_read_before_store`.
- Messages that show an inferred-error result change wording; goldens that hold them are regenerated.
