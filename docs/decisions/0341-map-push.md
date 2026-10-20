# 0341. `m.push(k, x)` appends to a list-valued map entry

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Collections, token efficiency
- **Spec rules:** `type.map.8`, `type.coll.13`

## Context

`m[k] += 1` starts from zero (ADR 0251), but a list-valued entry had no
equivalent: `m[k] = (m.get(k) else []) + [x]` also needs `+` on lists, which
does not exist. `scan` avoided the construct with a second pass over its list.

## Options considered

### Option A — `m.push(k, x)` (chosen)
- **Pros:** one call; the element type follows from the map; no new operator;
  `scan` drops its second pass and its `shown` set: −4.8 tokens measured.
- **Cons:** one more method on `Map`, valid only for list values.

### Option B — `+` on lists
- **Pros:** general.
- **Cons:** still needs `m.get(k) else []` and a rewrite of the entry, so it
  is longer than A, and `+` on lists invites quadratic copies.

### Option C — leave it
- **Cons:** the two-pass shape stays.

## Decision

On a `Map[K, List[T]]`, `m.push(k, x)` (`mut self`) appends `x` to the list at
`k`, starting an empty list when `k` is missing. On a pending `{}` local
(`type.coll.13`), `m.push(k, x)` fixes its type to `Map[K, List[X]]`. Any
other map value type is an error whose help points to `m[k] = v`.

## Rationale

Tokens decide: `scan` falls from 291.6 to 286.8 tokens, and the program is
clearer (group, then report). The corpus total change is in ADR 0340.

## Consequences

- Resolves OQ-111. The "a list index must be `int`" message on an untyped
  `{}` stays, but `push` on one now types it instead.
- With unique values, the pushed argument moves (`own.uniq.9`).

## Rejected alternatives

- `+` on lists: longer and slower (Option B).
- Leaving it: measured gap (Option C).
