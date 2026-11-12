# 1086. `Heap[T]` and `Deque[T]` in the prelude

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 04 — Types; standard library
- **Spec rules:** [type.heap.1-5, type.deque.1-5](../spec/04-types.md)
- **Builds on:** [ADR 0937](0937-grid-flat-2d-list.md) (prelude types), [ADR 1076](1076-struct-field-defaults.md) (field defaults)

## Context

A priority queue and a double-ended queue are the two collections agents
reach for after `List`, `Map` and `Set` (Dijkstra, BFS, sliding windows). AIL
had neither, so every program wrote its own heap (the review's `p2_route`
does). An agent writes `Heap[int]()` unprompted, so the names must work
with no `use`.

## Decision

Both are prelude modules written in AIL, read when a program names `Heap` or
`Deque` and doesn't declare that name itself (a program's own `Heap` still
wins, as with `Grid`).

- `Heap[T Ord]`: `Heap[T]()` is min-first; `Heap[T](max_first: true)` is
  max-first; `Heap.from(xs)` and `Heap.from_max(xs)`; `push`, `pop() T?`,
  `peek() T?`, `len`, `is_empty`, `to_sorted`. A binary heap over a list.
- `Deque[T]`: `Deque[T]()`, `Deque.from(xs)`; `push_front`, `push_back`,
  `pop_front() T?`, `pop_back() T?`, `first`, `last`, `len`, `is_empty`,
  `at(i)`, `to_list`. Two lists (front reversed, back) that rebalance by
  halves, so every operation is O(1) amortised and `at` is O(1).
- Constructors use struct field defaults (ADR 1076), so no new syntax.
- Method names follow `List` and `Map` (`len`, `is_empty`, `first`, `last`,
  `push`, `pop`) for the one-collection-vocabulary item.

`d[i]` and `for x in d` on a deque are a second step (checker and lowering
hooks like `Grid`'s `g[i, j]`); until then `d.at(i)` and `for x in d.to_list()`.

## Rejected alternatives

- **`use std.heap`**: agents would have to know the import; the prelude is
  read on the name.
- **`Heap.max[T]()`**: a call with no argument can't infer `T`; `max_first:`
  reads at the construction site and needs no new form.
- **A comparator parameter**: closures exist, but a second way to order
  duplicates `Ord`; a max-first flag covers the common case.
- **A ring buffer**: needs a filler for empty slots of any `T`; two lists
  need none and have the same bounds.
