# 1168. A collection whose first use only reads it: certain fix from a later use

- **Status:** Accepted
- **Date:** 2026-11-19
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer's rule of no syntax change, with fixes, diagnostics and the card only
- **Topic:** 3 — Diagnostics
- **Spec rules:** [type.coll.13](../spec/04-types.md)
- **Amends:** [ADR 1135](1135-self-read-store-and-text-failure-fixes.md) (its certain fix for `{}` covered a later store only)

## Context

In fresh first drafts of the ledger task (2 of 8), the first use of an untyped local only read it, and a later use fixed
its type: `mut sums := []`, then `idx[k] = sums.len()`, then `sums.push(bigint.from_int(0))`; and `mut m := {}`, then
`cur := m.get(key) else bigint.from_int(0)`, then `m[key] = cur + ...`. Each was one type.coll.13 error, fixed by hand
in a second version. The same family caused 18 failed versions in the E14 run.

## Decision

Keep the rule (a first use that doesn't fix the type is an error) and make the report carry a certain fix
([ADR 1130](1130-certain-fixes-applied-by-check.md)) when a later use gives every type:

- `[]` becomes `List[T]()` at a later `x.push(e)` (`T` the type of `e`).
- `{}` becomes `Set[T]()` at a later `x.add(e)`, `Map[K, List[E]]()` at `x.push(k, e)`, and (ADR 1135) `Map[K, V]()` at a
  later `x[k] = v`.
- `{}` becomes `Map[K, V]()` from `m.get(k) else d` itself: `K` is the key's type and `V` the default's.

Every type must be known (`T`, `K` and `V` without unknowns; `K` and set elements hashable); otherwise the error stays
unchanged. Only the first fix is kept. Checker only: the program is rewritten before it is lowered, so no native
lowering changes.

## Options considered

- **Infer the type in the checker from a later use** (look ahead, or settle on the first read): rejected; the reading
  statements are checked before the fixing one, so the checker would need a second pass or a retyped body, plus matching
  native lowering for each case. The certain fix writes the type into the file, which keeps it visible to readers (rank 1
  and 2 of ADR 1077).
- **A fix for every first read, with the type guessed from the later use even when it isn't certain**: rejected; a wrong
  type gives a second error.

## Consequences

- `compiler/check/lateknow.ail`; `OpenBrace` gains `list`, and `open_brace` is recorded for `[]` too (`selfread.ail`,
  `body.ail`); hooks in `call.ail` (`method`) and `expr.ail` (`OrElse`).
- Mistakes pairs `list_len_before_push`, `list_is_empty_before_push`, `list_contains_before_push`, `set_len_before_add`,
  `map_len_before_push_list`, `map_get_else_before_store`.
- Not gated yet: nothing was built in the cloud.
