# 0214. List editing methods: `insert`, `remove_at`, `extend`, `clear`, `reverse`, `first` (resolves OQ-047) (*delegated*)

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Collections
- **Spec rules:** `type.coll.14`, `type.coll.4`

## Context

The editor (ADR 0180) rebuilt its line list with a hand-written `splice`
because `List` had only `push`, `pop` and `swap`. OQ-047 offered methods,
`+` on lists, or a general `splice`. `xs[a..b]` already slices (ADR 0136), so
no `slice` method is added.

## Options considered

### Option A — named in-place methods (chosen)
- **Pros:** one token each; the names agents reach for; in place, no hidden copy.
- **Cons:** several names to learn.

### Option B — `+` on lists
- **Pros:** concise.
- **Cons:** hides an O(n) copy; AIL has no `+` on strings either.

### Option C — one `splice(from, to, items)`
- **Pros:** one method.
- **Cons:** less obvious; every simple insert or removal needs three arguments.

## Decision

Add `insert(i, x)`, `remove_at(i) T`, `extend(ys)`, `clear()`, `reverse()`
(all `mut self`) and `first() T?`. Bad indexes trap under `type.coll.4`;
`insert` accepts `i == len`.

## Rationale

Tokens were a tie (library additions; the editor changed by -0.9% mean
tokens, see ADR 0215). AI understanding decided: common names, no hidden cost.

## Rejected alternatives

- Option B: hidden copies, and no list concatenation operator elsewhere.
- Option C: less discoverable.
- A `slice` method: `xs[a..b]` already covers it.
