# 1125. The collection methods agents guess

- **Status:** Accepted
- **Date:** 2026-11-15
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); slice S14 of the E13 follow-up plan, which the designer approved
- **Topic:** 4 — Type system; 22 — Tools
- **Spec rules:** [type.coll.33](../spec/04-types.md), [type.str.13](../spec/04-types.md) (amended), [type.coll.14](../spec/04-types.md) (amended)
- **Amends:** [ADR 1099](1099-collection-vocabulary.md) (adds to the shared vocabulary)

## Context

In the E13 agent benchmark, first versions of programs used method names from
other languages that AIL rejected: `s.keys()` and `s.to_list()` on a set,
`xs.slice(a, b)` on a list, `x.unwrap()` and `x.unwrap_or(d)` on an optional,
`n.to_string()`, and `m.insert(k, v)` on a map. Each cost a compile round.
Some had help, but agents ran `ail fix` in 3 of 288 trials, and help that
names no exact rewrite still costs a lookup. Where a guessed name has one
meaning, accepting it is cheaper for every reader than an error.

## Options considered

### Option A — accept the names whose meaning is unambiguous, fix the rest (chosen)
- `xs.slice(a, b)` and `xs.slice(a)` on a list, the same as `xs[a..b]` and
  `xs[a..]`, so a list slices as a `str` does; `s.slice(a)` on a `str` for the
  same reason.
- `s.to_list()` on a set: the elements in insertion order, as
  `d.to_list()` gives on a `Deque`.
- `s.keys()` and `s.values()` on a set stay errors (a set has no keys), with
  the fix `to_list`.
- `x.unwrap_or(d)`, `x.unwrap()` and `x.expect(m)` on an optional stay errors
  (an optional has no methods, ADR 0139), now with the fixes `(x else d)` and
  `(x else unreachable(...))`. The parentheses make the rewrite exact wherever
  the call stood, since `else` binds looser than a call. The fix is offered
  only when `x` is a name, field chain or short call.
- `m.set`, `m.insert` and `m.put` on a map stay errors whose help writes the
  line `m[k] = v` with the map's name. No fix: an assignment is a statement and
  the checker can't see whether the call's value is used (fix.edit.1).
- `n.to_string()` already had the fix `to_str` (ADR 0262).
- **Pros:** the first version compiles more often; two spellings of slicing
  read the same; every fix is certain.
- **Cons:** two ways to slice a list.

### Option B — help text only, no new methods
- **Pros:** one way to slice.
- **Cons:** the E13 errors remain; agents that don't run `ail fix` still pay
  a round.

### Option C — also accept `unwrap`/`unwrap_or` as methods of an optional
- **Rejected:** `unwrap` hides a trap behind a common word, and two ways to
  give a fallback (`else` and `unwrap_or`) split the vocabulary for no gain in
  meaning (ADR 0076 criterion 1).

## Decision

Option A. Token cost: none on code that doesn't use the new names; the
spellings are as long as the ones they stand for, so the difference is a tie
by ADR 0076.

## Consequences

- `tools/collection-vocab-audit.py` is unchanged: `slice` and `to_list` are
  not in its shared list.
- `tests/mistakes/rs_unwrap_or` now has a `.fixed` file; its frozen golden
  rows are listed in `tests/golden/known.txt`.
