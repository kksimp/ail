# 0254. `Json` gets typed field getters and `set`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Standard library (`Json`); token efficiency
- **Spec rules:** json.query.1 (amended), json.query.2–3

## Context

Reading a typed member was `j.get("id")?.as_str()`, and changing one member
meant rebuilding the object's member list in a loop (`tasks`'s
`with_status`, nine lines). Python writes `j.get("id")` and
`{**j, "status": s}`.

## Options considered

| Form | tasks | build |
|---|---|---|
| `j.str("id")` (chosen) | 742.6 | 456.2 |
| `j["id"]?.as_str()` | 760.2 | 465.2 |
| `j.get("id")?.as_str()` | 768.6 | 469.4 |

### Option A — `str`, `num`, `bool`, `arr` getters and `set` (chosen)
- **Pros:** getters −39.2 tokens (−2.2% of the corpus); `set` −43.2
  (−2.4%); plain prelude methods, no new syntax.
- **Cons:** five more `Json` methods; `str` and `bool` are also type names.

### Option B — indexing `j["k"]` giving `Json?`
- **Cons:** less than half the saving, and `[]` on a `Map` traps on a
  missing key, so the same syntax would mean two things.

### Option C — keep `get`/`as_*`
- **Cons:** the tokens above.

## Decision

`j.str(k)`, `j.num(k)`, `j.bool(k)` and `j.arr(k)` give member `k` as that
variant or `none`. `j.set(k, v)` (`mut self`) replaces the first member
named `k` or appends it, and traps unless `j` is an object.

## Rationale

Both changes clear the 2% threshold on **tokens**. The getter names are the
variant names models already see in `Json.Str` and `as_str`.

## Consequences

- No `obj(k)` getter: `j.get(k)` already gives the `Json`.

## Rejected alternatives

- Indexing: smaller saving, conflicting meaning.
- Keeping `get`/`as_*` only: tokens.
