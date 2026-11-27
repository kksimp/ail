# Standard library: `std.ordcoll`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR 1260).

Ordered collections: `OrdMap` and `OrdSet` keep their keys in `Ord` order
(first and last, floor and ceiling, ranges, in-order lists), and `IndexHeap` is
a priority queue over integer ids with update and decrease-key. The built-in
`Map` and `Set` keep insertion order and `Heap` has no way to change a queued
item; these types fill that gap. Nothing here touches the OS, so every
function is pure (`eff.infer.1`) and needs no capability. The module is
written in AIL and is in scope without `use` (`std.use.4`); `ordcoll.OrdMap`
names it.

Each entry costs its key and its value and a share of two list headers per
chunk of up to 128 entries. There is no node or pointer per entry.

## Ordered map

**std.ordcoll.1** — `ordcoll.OrdMap[K, V].new()` is an empty map whose key type
`K` implements `Ord`. Keys compare with `<`, so two keys that neither
is below the other are one key. Details: [below](#more-on-stdordcoll1). *(ADR 1260)*

```ail tool=run,FILE expect=0,true rules=std.ordcoll.1
fn main()
    m := ordcoll.OrdMap[str, int].new()
    print("{m.len()},{m.is_empty()}")
```

**std.ordcoll.2** — `m.put(k, v)` sets the value of `k` and adds the entry
when `k` is new. Details: [below](#more-on-stdordcoll2). *(ADR 1260)*

```ail tool=run,FILE expect=b1,none,2 rules=std.ordcoll.2
fn main()
    mut m := ordcoll.OrdMap[str, str].new()
    m.put("b", "b1")
    m.put("a", "a1")
    print("{m.get("b") else "none"},{m.get("z") else "none"},{m.len()}")
```

**std.ordcoll.3** — `first()` and `last()` give the entries with the smallest
and the largest key as `(K, V)?`; `pop_first()` and `pop_last()` also remove
them. All are `none` for an empty map. *(ADR 1260)*

```ail tool=run,FILE expect=1,a rules=std.ordcoll.3
fn main()
    mut m := ordcoll.OrdMap[int, str].new()
    m.put(2, "b")
    m.put(1, "a")
    e := m.pop_first() else
        return
    print("{e.0},{e.1}")
```

**std.ordcoll.4** — `floor(k)` is the entry with the largest key `<= k`,
`ceiling(k)` the one with the smallest key `>= k`, `lower(k)` the largest key
`< k` and `higher(k)` the smallest key `> k`; each is `(K, V)?`. *(ADR 1260)*

```ail tool=run,FILE expect=10,30 rules=std.ordcoll.4
fn main()
    mut m := ordcoll.OrdMap[int, bool].new()
    m.put(10, true)
    m.put(30, true)
    f := m.floor(25) else
        return
    c := m.ceiling(25) else
        return
    print("{f.0},{c.0}")
```

**std.ordcoll.5** — `range(lo, hi)` is the entries with `lo <= key < hi` in key
order, and `from_key(lo, limit)` the first `limit` entries from the first key
`>= lo`. *(ADR 1260)*

```ail tool=run,FILE expect=2,3 rules=std.ordcoll.5
fn main()
    mut m := ordcoll.OrdMap[int, bool].new()
    for k in [1, 2, 3, 4]
        m.put(k, true)
    print(["{e.0}" for e in m.range(2, 4)].join(","))
```

**std.ordcoll.6** — `keys()`, `values()` and `items()` are lists in key order.
*(ADR 1260)*

```ail tool=run,FILE expect=a,b rules=std.ordcoll.6
fn main()
    mut m := ordcoll.OrdMap[str, int].new()
    m.put("b", 2)
    m.put("a", 1)
    print(m.keys().join(","))
```

**std.ordcoll.7** — `rank(k)` is how many keys are `< k`, and `nth(i)` the
entry with `i` keys below it, `none` outside `0..len()`. Both walk the chunks
(at most `len() / 64` of them) and not the entries. *(ADR 1260)*

```ail tool=run,FILE expect=1 rules=std.ordcoll.7
fn main()
    mut m := ordcoll.OrdMap[int, bool].new()
    m.put(5, true)
    m.put(9, true)
    print(m.rank(9))
```

## Ordered set

**std.ordcoll.8** — `ordcoll.OrdSet[K].new()` is an empty set of `Ord` elements.
`add(x)` is true when `x` was not in the set, `remove(x)` true when it was,
`contains(x)`, `len()` and `is_empty()` as for a map. A set is an `OrdMap[K,
bool]`. *(ADR 1260)*

```ail tool=run,FILE expect=true,false rules=std.ordcoll.8
fn main()
    mut s := ordcoll.OrdSet[int].new()
    print("{s.add(3)},{s.add(3)}")
```

**std.ordcoll.9** — `first()`, `last()`, `pop_first()`, `pop_last()`, `floor`,
`ceiling`, `lower` and `higher` are as for a map and give `K?`. *(ADR 1260)*

```ail tool=run,FILE expect=4 rules=std.ordcoll.9
fn main()
    mut s := ordcoll.OrdSet[int].new()
    _ := s.add(4)
    _ := s.add(8)
    print(s.floor(7) else -1)
```

**std.ordcoll.10** — `range(lo, hi)` is the elements with `lo <= x < hi`,
`to_list()` every element in order, and `rank(x)` how many are `< x`. *(ADR 1260)*

```ail tool=run,FILE expect=1,2 rules=std.ordcoll.10
fn main()
    mut s := ordcoll.OrdSet[int].new()
    for x in [3, 1, 2]
        _ := s.add(x)
    print(s.range(1, 3).join(","))
```

## Indexed priority queue

**std.ordcoll.11** — `ordcoll.IndexHeap[P].new()` is an empty queue of integer
ids, each with a priority `P` (`Ord`), the smallest priority served first;
`new_max()` serves the largest first. Details: [below](#more-on-stdordcoll11). *(ADR 1260)*

```ail tool=run,FILE expect=true rules=std.ordcoll.11
fn main()
    q := ordcoll.IndexHeap[int].new()
    print(q.is_empty())
```

**std.ordcoll.12** — `push(id, p)` queues `id`, or gives a queued `id` the
priority `p`, better or worse. Details: [below](#more-on-stdordcoll12). *(ADR 1260)*

```ail tool=run,FILE expect=true,false rules=std.ordcoll.12
fn main()
    mut q := ordcoll.IndexHeap[int].new()
    q.push(1, 10)
    print("{q.decrease_key(1, 5)},{q.decrease_key(1, 8)}")
```

**std.ordcoll.13** — `peek()` and `pop()` give the next `(id, priority)` or
`none`, `pop` also removing it. `contains(id)` and `priority(id)` (`P?`) look an
id up, and `remove(id)` takes any queued id out and gives its priority.
*(ADR 1260)*

```ail tool=run,FILE expect=2,5 rules=std.ordcoll.13
fn main()
    mut q := ordcoll.IndexHeap[int].new()
    q.push(1, 9)
    q.push(2, 5)
    e := q.pop() else
        return
    print("{e.0},{e.1}")
```

## Rejected alternatives

- **A B-tree or red-black tree of nodes.** Per-node headers and pointers
  cost memory the heap-LRU benchmark already spends too much of, and AIL
  lowers a list of keys to a flat array. Chunked sorted arrays have the same
  search cost and no per-entry overhead.
- **A skip list.** Random levels need a runtime random source and per-entry
  pointers.
- **`m[k] = v` and `for k in m`.** An index assignment and iteration on a
  library type are language features; `put` and `keys()` need none.
- **Extending the built-in `Heap` with decrease-key.** It would add a position
  table every `Heap` pays for; the indexed queue is a separate type.

### `std.ordcoll` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on std.ordcoll.1

`len()` and `is_empty()` give the entry count. `for` over an `OrdMap` is not defined: use `keys()`, `values()` or `items()` (`std.ordcoll.6`).

#### More on std.ordcoll.11

Ids are `0, 1, 2, ...`, such as node numbers: the queue keeps a slot per id up to the largest one pushed, so a sparse id like 4000000000 costs memory (use a `Heap` there). `len()` and `is_empty()` as above.

#### More on std.ordcoll.12

`decrease_key(id, p)` changes a queued `id` only when `p` is served before its present priority, and returns true when it did; it is false for an id that is not queued. A negative id traps (`type.index.1`). Each is O(log n).

#### More on std.ordcoll.2

`m.get(k)` is the value or `none`, `m.contains(k)` a `bool`, and `m.remove(k)` takes the entry out and gives its value, or `none`. Each costs a binary search over the chunks, one inside a chunk, and a shift of at most 128 entries.
