# 0131. Maps and sets keep insertion order; the collection API, places and loops

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** Designer (insertion order for iteration, `keys()`, `values()`, `sorted()` and text form)
- **Details decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)): the API, the place rules and the loop rules
- **Topic:** 4 — Type system; 21 — Standard library
- **Spec rules:** [type.coll.8, type.map.1–6](../spec/04-types.md)
- **Extends:** [ADR 0090](0090-collections.md) (type.coll.1–3 stand; the empty constructors it mentions only in a diagnostic become real)
- **Amends:** [ADR 0117](0117-typing-rules.md) (own.place.1: an entry of a map place is a place; expr.loop.4: `for x in s` accepts a `Set`)
- **Evidence:** `examples/interp.ail`, `examples/json.ail`, `examples/sheet.ail` (outputs identical to the Python versions); tokbench, mean of five tokenizers

## Context

ADR 0090 gave `Map[K, V]` and `Set[T]` literals and nothing else: no
iteration order, no methods, no rule for `m[k] = v`, no equality or text form.
All three example programs use them: scopes and function tables keyed by
`str`, word counts, and a spreadsheet keyed by `Pos` with `Set[Pos]` values.
Their output has to match the Python originals, whose dicts keep insertion
order.

## Options considered

Iteration order:

| Option | Output depends on the hash function | Key needs |
|---|---|---|
| **Insertion order (chosen)** | no | `Hash` |
| Hash order | yes | `Hash` |
| Sorted by key | no | `Ord`, O(log n) |

`for k in m`, over the five map loops in `sheet.ail` (tokbench):

| Option | The five loop headers | Whole file |
|---|---|---|
| **`for k in m.keys()` (chosen)** | 100.8 | 6019.4 |
| `for k in m` meaning keys | 91.8 | 6009.8 (−0.16%) |

## Deciding criterion

The designer chose insertion order, which also settles that output never
depends on the hash function (ADR 0133 hides hash values too).

For the rest, token usage **ties**: the bare loop saves 1.8 tokens per loop
(−0.16% of the file), under the ~2% threshold. **AI understanding** decides.
`for k in m` means keys in Python and pairs in Rust, so either reading is a
guess; `m.keys()` isn't, and the sheet's author wrote `.keys()` five times out
of five without being told. For the place rules, the one that never creates
an entry by accident wins: only a whole-entry assignment inserts.

## Decision

1. **Empty collections (type.coll.8).** `List[T]()`, `Map[K, V]()` and
   `Set[T]()` give an empty collection anywhere, without an expected type. `{}`
   with an expected `Map` or `Set` type is empty; with none it's type.coll.3's
   error, whose help suggests these.
2. **Order (type.map.1).** Iteration, `keys()`, `values()`, `sorted()` and the
   text form follow insertion order. Assigning to an existing key keeps its
   position; `remove` keeps the order of the others; a re-added key goes last.
3. **Hash (type.map.2).** Keys and set elements must be `Hash` (ADR 0133).
   This is checked where the type is written or inferred: `Map[float, int]`
   is an error naming the cause.
4. **Indexing (type.map.3).** `m[k]` reads the value and traps if `k` is
   missing; the message shows the key's text form. An entry `m[k]` of a map
   place is a place, but only when it is the whole target of `=` does it
   insert or replace. Every other use traps if `k` is missing: `op=` targets,
   a step before the last in a longer place (`m[k].f = v`, `m[k][j] = v`;
   `m[a][b] = v` needs `m[a]` and inserts `b`), the receiver of a `mut self`
   method (`m[k].push(x)`), a `mut` argument, `for mut x in m[k]`. `k` is
   evaluated once, before the right-hand side.
5. **Equality and text (type.map.4).** Maps are `==` when they have the same
   keys with equal values (`V` must be `Eq`), sets when they have the same
   elements. Order is ignored, so equal maps can print differently. Maps and
   sets aren't `Ord` or `Hash`. The text form is `{"a": 1, "b": 2}` and
   `{1, 2}`; an empty map or set is `{}`.
6. **Loops (type.map.5).** `for x in s` iterates a set in insertion order, over
   the value at loop start (expr.loop.4). `for k in m` on a map is an error
   with the help "iterate `m.keys()`". `for mut` over a set, or over
   `m.keys()` or `m.values()`, is an error.
7. **API (type.map.6).** Map: `len()`, `get(k) -> V?`, `contains(k) -> bool`,
   `remove(k) -> V?` (`mut`), `keys() -> List[K]`, `values() -> List[V]`.
   Set: `len()`, `add(x) -> bool` (`mut`; whether it was added), `remove(x) ->
   bool` (`mut`; whether it was there), `contains(x)`, `sorted() -> List[T]`
   (`T` is `Ord`), `is_subset(t) -> bool`. Helps: `m.get(k, d)` becomes
   `v := m.get(k) else d`; `contains_key` becomes `contains`; `items()`
   becomes iterating `keys()` and indexing.
8. **Literals.** Duplicate keys or elements that are equal constants are a
   compile-time error. Otherwise a later duplicate replaces the value and keeps
   the first position.

```ail
mut seen := Map[str, int]()
seen["a"] = 1                 # inserts
seen["a"] += 1                # the key exists: fine
for k in seen.keys()
    print("{k}: {seen[k]}")   # a: 2
# seen["b"] += 1              # would trap: no key "b"
mut words := Set[str]()
if words.add("x")             # true: it was added
    print(words)              # {"x"}
```

## Consequences

- Output is the same on every run and platform.
- `a == b` can hold while `"{a}" != "{b}"` and `a.keys() != b.keys()`; empty
  maps and sets print the same. A map with NaN values isn't equal to itself.
- An implementation can use an entries array plus an index table (Python's
  dict layout). A swap-remove would reorder the others and isn't allowed.
- `for k in m.keys()` SHOULD iterate the map directly rather than build the
  list (values are copy-on-write, so this is safe); the type stays `List[K]`.
- `m.get(k)` on a `Map[K, V?]` has type `V??`, which needs a rule for `none`
  (OQ-044 in [open-questions.md](../open-questions.md)).
- Set algebra (union, intersection, difference) isn't designed; `is_subset`
  is the only set-to-set method.

## Rejected alternatives

- **Hash order:** output would depend on the hash function.
- **Sorted maps:** O(log n) operations, and `K` would need `Ord`.
- **`for k in m` meaning keys (Python) or pairs (Rust):** a token tie, and
  each reading surprises users of the other; `m.keys()` is explicit.
- **Auto-inserting a default on `m[k] op= v`:** AIL has no default values.
  The same goes for auto-creating `m[k]` in any longer place.
- **`remove` as swap-remove (indexmap's `remove`):** reorders the other keys.
- **`m[k]` returning `V?` (Swift, Kotlin) or a zero value (Go):** an optional
  burdens every successful read, and a zero value hides missing keys.
  `m.get(k)` is the optional form.
- **`for mut` over a set or its keys:** changing an element would break its
  hash.
