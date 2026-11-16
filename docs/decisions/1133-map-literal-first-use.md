# 1133. `mut m := {}` is a map from `get_or`, and `contains` waits for the store

- **Status:** Accepted
- **Date:** 2026-11-15
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** Collections; AI understanding
- **Spec rules:** type.coll.13

## Context

type.coll.13 types `x := {}` by its first use, but only a plain store
`m[k] = v`, `m[k] op= n`, `m.add(e)` or `m.push(k, e)` did. In the E13
benchmark (2 of 12 plain stock trials, 5 core-card versions) agents wrote
`mut qty := {}` then `qty[k] = qty.get_or(k, 0) + q`, or `if not
items.contains(sku)` first. The store's value reads the map before its type is
known, so the local was left without a type and the errors that followed
talked about a list index. The card already promises "typed by first use".
No language change: only which first uses the checker understands.

## Options considered

### Option A: two more first uses (chosen)
`m.get_or(k, d)` types the map `Map[K, D]` from the key and the default, so
the store around it works. `m.contains(k)` before the first store is only a
test: it checks the key and leaves the type to the store that follows.
- **Pros:** the two forms agents write work; each rule is local to one call.
- **Cons:** `m.get(k)` and `m.remove(k)` as the first use still need the type
  (their value type can't be known from the call); the error says so.

### Option B: look ahead for the first store anywhere in the function
- **Pros:** covers every order.
- **Cons:** a second pass of the checker over the block; the type of a local
  would depend on code that follows it, against local reasoning (criterion 1).

### Option C: only improve the diagnostic
- **Cons:** the first build still fails.

## Decision

Option A. The help for a pending `{}` stays: declare it with `Map[K, V]()`.
