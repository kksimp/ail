# 0318. Lists and maps may hold unique values; such a collection is unique

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 7 — Ownership; 5 — Types
- **Spec rules:** [own.uniq.5, own.uniq.8–9](../spec/07-ownership.md#unique-values-in-lists-and-maps)

## Context

ADR 0191 kept unique values out of every collection because the bootstrap
copies collection values. A server holding many connections, or a tool
holding many open files, then has no way to keep them (OQ-050).

## Options considered

### Built-in `List` and `Map` values hold unique values (chosen)
`List[T]` and `Map[K, T]` of a unique `T` are themselves unique: they move,
never copy. Methods that put an element in take it with `move`
(`push`, `insert`, `extend`); `pop`, `remove_at` and `m.remove` move one
out; indexing and `for` borrow in place. Literals move their elements in.
- **Pros:** no new type or keyword; the existing move rules and diagnostics
  apply; the borrow model (ADR 0062) already covers `xs[i].f` and
  `for mut x in xs`.
- **Cons:** the element type decides whether a list copies, so a list's
  behaviour depends on `T` (shown by `is unique` in diagnostics).

### A separate `UniqueList[T]`
- **Pros:** explicit at the type.
- **Cons:** a second list type and API to learn, more tokens per use, and
  conversions between them.

### A copyability bound on all generic code (`[T unique]`)
- **Pros:** also opens user generics to unique values.
- **Cons:** a large design touching every generic signature; not mergeable
  tonight. Left open in OQ-050 for user-defined generics.

### Keep the restriction
- **Cons:** blocks the resource-holding programs agents write.

### How an element goes in
Chosen: `xs.push(move x)`, as own.move.2 requires for every `move`
parameter, including a new value. Rejected: unmarked `xs.push(x)` for
unique values only, which gives `push` two call forms depending on `T`.

## Decision

`List[T]` and `Map[K, T]` may hold a unique `T` and are then unique; push,
insert and extend move in, pop, remove_at and remove move out, and indexing
and iteration borrow. Map keys, set elements, `ref` contents and type
arguments still can't be unique.

## Rationale

AI understanding decided (criterion 2): one list type, one set of move rules,
and every misuse is a compile-time error with a help naming the method that
moves. Tokens tie with `UniqueList` on the operations themselves and favour
this option on type names.

## Consequences

- `Checker::is_unique` treats `List`/`Map` of a unique type as unique, so
  a struct field of one makes the struct unique (own.uniq.5).
- The interpreter releases a list's elements (and a map's values) last first
  when the collection is released; `clear` and index assignment release what
  they remove. Detecting a unique list looks at its first present element,
  so lists of plain values pay nothing extra.

## Rejected alternatives

- **`UniqueList[T]`:** a second collection API.
- **Generic copyability bound now:** too large for this round (OQ-050).
- **Unmarked `push` of unique values:** two call forms for one method.
