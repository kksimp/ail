# Standard library: `std.lru`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR TBD).

Bounded caches that give up the entry used longest ago: `Cache`, with a
capacity, and `Ttl`, whose entries also expire. `Cache` is pure
(`eff.infer.1`); `Ttl` reads the monotonic clock (`std.time.3`) in the forms
without `_at`. The module is written in AIL and is in scope without `use`
(`std.use.4`); `lru.Cache[str, int].new(100)` names it.

## The cache

**std.lru.1** — `lru.Cache[K, V]` holds at most `capacity` entries, `K` being
a `Hash` type. `Cache[K, V].new(capacity int)` makes an empty one; a
`capacity` below 1 is 1. `len()` is the number of entries and `capacity()`
the most it holds. *(ADR TBD)*

```ail tool=run,FILE expect=0,2,1 rules=std.lru.1
fn main()
    c := lru.Cache[str, int].new(2)
    one := lru.Cache[int, int].new(-5)
    print("{c.len()},{c.capacity()},{one.capacity()}")
```

**std.lru.2** — `get(mut self, k K) V?` gives the value for `k` and marks `k`
as the most recently used; it is `none` when `k` is not cached.
`peek(k) V?` and `contains(k) bool` read without marking. *(ADR TBD)*

```ail tool=run,FILE expect=1,2,true rules=std.lru.2
fn main()
    mut c := lru.Cache[str, int].new(2)
    c.put("a", 1)
    c.put("b", 2)
    print("{c.get("a") else 0},{c.peek("b") else 0},{c.contains("b")}")
```

**std.lru.3** — `put(mut self, k K, v V)` stores `v` under `k` as the most
recently used entry. A `k` already cached has its value replaced. A new `k`
in a full cache first removes the least recently used entry. *(ADR TBD)*

```ail tool=run,FILE expect=1,false,true rules=std.lru.3
fn main()
    mut c := lru.Cache[str, int].new(2)
    c.put("a", 1)
    c.put("b", 2)
    first := c.get("a") else 0
    c.put("c", 3)
    print("{first},{c.contains("b")},{c.contains("a")}")
```

**std.lru.4** — `remove(mut self, k K) bool` removes `k` and is true when it
was cached. `clear()` removes every entry. *(ADR TBD)*

```ail tool=run,FILE expect=true,false,0 rules=std.lru.4
fn main()
    mut c := lru.Cache[str, int].new(3)
    c.put("a", 1)
    print("{c.remove("a")},{c.remove("a")},{c.len()}")
```

**std.lru.5** — `keys() List[K]` lists the keys from the most recently used
to the least. It does not mark anything. *(ADR TBD)*

```ail tool=run,FILE expect=a,c,b rules=std.lru.5
fn main()
    mut c := lru.Cache[str, int].new(3)
    c.put("a", 1)
    c.put("b", 2)
    c.put("c", 3)
    one := c.get("a") else 0
    print(c.keys().join(","))
```

## Entries that expire

**std.lru.6** — `lru.Ttl[K, V].new(capacity int, ttl_ms int)` is a cache whose
entries live `ttl_ms` milliseconds from the `put` that stored them (a
`ttl_ms` below 1 is 1). It has `get`, `peek`, `put`, `contains`, `remove`,
`len`, `capacity`, `clear` and `keys` as `Cache` does, and `purge() int`.
Expiry is lazy: `get` of an expired key removes it and gives `none`;
`peek` and `contains` treat it as missing and leave it; `purge` removes every
expired entry and gives their number; `keys` purges first; `len` counts the
expired entries not yet removed. A `get` refreshes the recency of an entry but
not its lifetime; a `put` refreshes both. A full cache removes its least
recently used entry, expired or not. *(ADR TBD)*

```ail tool=run,FILE expect=1,true,0 rules=std.lru.6
fn main()
    mut t := lru.Ttl[str, int].new(10, 60_000)
    t.put("a", 1)
    print("{t.get("a") else 0},{t.contains("a")},{t.purge()}")
```

**std.lru.7** — The clock of `Ttl` is `time.mono_ns()`. `get_at(k, now_ns)`,
`put_at(k, v, now_ns)` and `purge_at(now_ns)` take the reading instead, so a
test needs no sleep. An entry stored at `t` with a lifetime of `ttl_ms` has
expired at every reading `>= t + ttl_ms * 1_000_000`. Use one clock for one
cache. *(ADR TBD)*

```ail tool=run,FILE expect=1,-1 rules=std.lru.7
fn main()
    mut t := lru.Ttl[str, int].new(10, 10)
    t.put_at("a", 1, 0)
    print("{t.get_at("a", 9_999_999) else -1},{t.get_at("a", 10_000_000) else -1}")
```

## Layout

**std.lru.8** — A cache keeps one `Map` from key to slot number and parallel
lists for the keys, the values and the two links of a doubly linked list
(slot numbers, not references). A slot freed by `remove`, `purge` or
eviction is reused, so a full cache allocates nothing per `put` beyond what
the `Map` needs. `get`, `put` and `remove` take constant time apart from the
`Map`'s own; `purge` and `keys` take time in the number of entries.
*(ADR TBD)*

**std.lru.9** — No function here fails: a missing key is `none` or `false`,
and a capacity or lifetime below 1 is 1. *(ADR TBD)*

```ail tool=run,FILE expect=-1 rules=std.lru.8,std.lru.9
fn main()
    mut c := lru.Cache[int, int].new(1)
    c.put(1, 10)
    c.put(2, 20)
    print(c.get(1) else -1)
```
