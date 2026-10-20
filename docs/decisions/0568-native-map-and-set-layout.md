# 0568. Native `Map` and `Set`: one insertion-ordered table

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** type.map.1, type.map.3, type.map.4, type.map.6, type.coll.1, own.val.1
- **Builds on:** [ADR 0404](0404-native-calling-convention-and-value-layouts.md) (layouts), [ADR 0407](0407-bump-allocator-no-header.md) (allocation)

## Context

`Map` and `Set` iterate in insertion order (type.map.1): an assignment to an
existing key keeps its place, `remove` keeps the others' order, and a key
added again goes last. The native subset (native.subset.1) had neither. The
self-compiling compiler uses both in most files, and `wordfreq` and `loops`
of the drag race need them.

## Decision

A `Map[K, V]` or `Set[T]` is a pointer to an 88-byte **table object** (heap
kind 7 for a map, 8 for a set; both use the layout, a set ignores the value
word). Two arrays hang off it, as in CPython's dict:

```text
+0   len     live entries              +8   used    array slots used, live or dead
+16  ecap    array capacity            +24  mask    index size - 1 (a power of two)
+32  kind    key kind: 0 int, 1 str, 2 bool
+40  ents    ecap * 24 bytes: hash (-1 when dead), key, value
+48  index   (mask + 1) * 8 bytes: 0 empty, else entry number + 1
+56  vmode   copy mode of values: 0 plain, 1 lists, 2 maps and sets
+64  vinfo   vmode 1: nesting depth * 256 + element bytes
+72  lastkey the last key found or added   +80 laste  its entry, or 0
```

- **Order is the array.** Insertion appends; an existing key's value is
  replaced in place; `keys()`, `values()` and set iteration walk the array
  skipping dead entries. This is type.map.1 with no extra bookkeeping.
- **Removal marks the entry dead** (hash -1, key and value cleared) and
  decrements `len`. The index keeps pointing at it, so probing needs no
  tombstones of its own, and the hash of a live entry is never -1.
- **Rebuild when the array is full:** the array and index are reallocated
  at double the capacity, or at the same capacity when at least half the
  slots are dead, and the live entries are re-indexed in order. The index
  has twice the array's capacity, so it is at most half full; probing is
  linear.
- **Hashing.** An `int` or `bool` key: multiply by a 64-bit odd constant and
  fold the high bits down; a `str` key: FNV-1a over its bytes, folded the
  same way. Hashes are kept non-negative (62 bits). A string key is
  compared by pointer, then length, then bytes.
- **Values are one 8-byte word:** a `float` as its bits, a `bool` as 0 or 1,
  anything else as the value (a pointer for a `str`, list or table).
- **Value semantics (own.val.1).** Binding or assigning a table from a
  place calls `rt.map_copy`, which copies the object, both arrays and
  (by `vmode`) every list and table value inside, as `rt.list_copy_deep`
  does for lists. A `mut` parameter shares the table, like a list.
- **Keys** are `int`, `bool` or `str` for now; other keys are reported under
  native.subset.1. Values may be any native type, tables included.

Runtime: `compiler/rt/native/map.ail` (`rt.map_new`, `map_set`, `map_find`,
`map_at`, `map_add`, `map_has`, `map_remove`, `map_keys`, `map_values`,
`map_copy`, `out_map`, `out_set`). A table is allocated through the
allocator as a plain block until the heap header of lane 171 lands; kind
bytes 7 and 8 are reserved for it.

## Consequences

- `m[k]`, `m[k] = v` and `contains` cost one hash and one or two probes.
  The `wordfreq` race program (10 million `str` keys through
  `get` and assignment) takes 0.36 s native against 1.9 s for CPython and
  0.055 s for `gcc -O2`; hashing the key twice per iteration is most of the
  gap (a cached string hash is a later option).
- A removed key's slot is reclaimed at the next rebuild, not at once, so a
  table with many removals and no insertions keeps its array.
- The collector (lane 171) sees one object with two interior arrays; it
  traces `ents` through `used` slots, skipping dead ones, and `index` not
  at all (it holds no pointers).

## Rejected alternatives

- **Robin Hood or chained buckets with the order in a linked list.** Faster
  removal, but every entry carries two more words and iteration chases
  pointers; the array keeps iteration a linear scan and the object small.
- **Sort keys on iteration.** Wrong: the order is insertion order, not key
  order.
- **A separate set layout with 16-byte entries.** Saves a word per element;
  the second code path is not worth it while sets are small in the compiler.
- **Boxing floats and bools in map values.** An allocation per store for no
  gain, because every value already fits a word.
