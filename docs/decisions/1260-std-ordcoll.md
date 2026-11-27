# 1260. `std.ordcoll`: ordered map, ordered set and indexed priority queue

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the std library to be filled out on 2026-11-26
- **Topic:** 17 — Standard library; AI understanding, speed
- **Spec rules:** std.ordcoll.1 to std.ordcoll.13 in [std-ordcoll.md](../spec/std-ordcoll.md)
- **Extends:** [ADR 1126](1126-std-modules-in-ail-without-use.md)

## Context

`Map` and `Set` keep insertion order and give no floor, ceiling, range or in-order walk, so agents sort keys on each
query or write a tree by hand (schedulers, interval lookups, leaderboards, time-keyed logs). The built-in `Heap` has no
decrease-key, so Dijkstra and Prim re-push stale entries. The std plan (`std-gap-plan.md`) and the overnight refill list
ask for an ordered map and a priority queue. The heap-LRU benchmark already uses about 6x C's memory, so entry overhead
matters.

## Decision

A new module written in AIL, `std/ordcoll/ordcoll.ail`, in scope without `use` (std.use.4); no syntax, checker or
runtime change. The name `ordcoll` has no local-name clash in the tree (the likelier `ordered`, `sorted` and `indexed`
appear in 11 to 31 files).

- `OrdMap[K Ord, V]`: sorted chunks of at most 128 keys, a parallel chunk list of values, an entry count. `put`, `get`,
  `contains`, `remove`, `first`, `last`, `pop_first`, `pop_last`, `floor`, `ceiling`, `lower`, `higher`, `range(lo, hi)`
  (half open), `from_key(lo, limit)`, `keys`, `values`, `items`, `rank`, `nth`. Search is a binary search over the chunks'
  last keys, then inside one chunk; an insert shifts at most 128 entries and a full chunk splits in two. An emptied chunk
  is dropped; chunks do not merge when they shrink.
- `OrdSet[K Ord]` is an `OrdMap[K, bool]` with the same query set.
- `IndexHeap[P Ord]`: a binary heap of integer ids with a position table, so `push` on a queued id updates its
  priority, `decrease_key` only improves it, and `remove(id)` is O(log n). `new()` is smallest first, `new_max()`
  largest first. Ids are dense non-negative integers (node numbers); the table grows to the largest id pushed.
- Keys compare with `<` only, so a type needs `Ord` and no `Eq` or `Hash`.

Memory: an entry is its key and value plus a share of two list headers per chunk (about 1.5 words per 64 entries on
average). No node header, no child or parent pointer. A B-tree of boxed nodes costs two or three words more per entry.

## Rejected alternatives

- **B-tree or red-black tree of nodes.** More memory per entry and more allocation, no better search cost for the
  sizes agents use; the chunked array is also much shorter to get right.
- **Skip list.** Needs a random source and per-entry pointers.
- **One flat sorted list.** Insert and remove shift the whole list: quadratic for 100000 entries. Chunks cap the shift
  at 128.
- **`m[k] = v` and `for k in m` for `OrdMap`.** These are language features (index assignment on a library type,
  an iteration protocol); `put` and `keys()` need none, and the designer's rule is no syntax change for the std library.
- **A decrease-key on the built-in `Heap`.** Every `Heap` would carry a position table; a separate type costs only the
  programs that use it.
- **Hash-set based `OrdSet`.** Needs `Hash` on the element and cannot answer floor or range.

## Consequences

- The module was written and tested without a build in the lane's container; the first Mac gate is its first compile.
- Follow-ups: merge small chunks after many removals; a `for` form when the language gets an iteration protocol;
  `OrdMap` as the store behind a time-keyed `std.log` or a scheduler.
