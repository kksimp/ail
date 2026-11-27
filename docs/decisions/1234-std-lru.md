# 1234. `std.lru`: least recently used and expiring caches

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the std library to be filled out on 2026-11-26
- **Topic:** 17 — Standard library; AI understanding, speed
- **Spec rules:** std.lru.1 to std.lru.9 in [std-lru.md](../spec/std-lru.md)
- **Extends:** [ADR 1126](1126-std-modules-in-ail-without-use.md)
- **Number:** the integrator assigns it and renames this file in the chain.

## Context

Tasks in the corpus and in ail-train (`lru_cache`, `lru_ttl_cache`, `lfu`/`ttl` caches) make every agent write a cache by
hand. The first draft is usually a `Map` plus a `List` of keys with `remove_at(0)` on every hit (O(n)), or a node
struct per entry. The Opus trial measured a hand-written heap LRU at about six times C's memory (a follow-up in the
project notes). The std plan (`std-gap-plan.md`, lane 23) asks for a lean one in std.

## Decision

A new module written in AIL, `std/lru/lru.ail`, in scope without `use` (std.use.4). No syntax change, no runtime or
checker work.

- `lru.Cache[K Hash, V]`: `new(capacity)`, `get` (`V?`, marks used), `peek`, `put`, `contains`, `remove`, `len`,
  `capacity`, `clear`, `keys` (newest first). A full cache evicts the least recently used entry on a new key.
- `lru.Ttl[K Hash, V]`: the same plus a lifetime in milliseconds, lazy expiry on lookup and `purge`, the clock
  `time.mono_ns()`, and `get_at`/`put_at`/`purge_at` taking the reading so tests need no sleep. `Ttl` wraps a `Cache`
  and shares its code.
- Layout: one `Map[K, int]` from key to slot, and parallel `List`s for keys, values, previous and next slot numbers
  (and the expiry for `Ttl`). Freed slots are reused through a free list. No node struct and no per-entry box.
- No failure paths: a missing key is `none` or `false`; a capacity or lifetime below 1 is 1.

The plan sketched `lru.Cache[K, V](capacity)`; AIL has no constructor taking a positional capacity for a struct with
internal fields, so the call is `lru.Cache[K, V].new(capacity)` (the `Stack[T].new()` form of the card).

## Rejected alternatives

- **Built-in runtime module.** Costs checker, effects and lowering work and collides in shared files; the structure
  is a `Map` and arrays, which the compiler already makes fast.
- **A node struct per entry with `Option` links.** One allocation per entry and a pointer per link: the memory
  profile this module exists to avoid.
- **An ordered `Map` as the recency list** (remove and re-insert on each hit). Depends on the `Map`'s tombstone
  handling for every `get`; the explicit links make each hit constant time.
- **One type with an optional lifetime.** Puts `get_at`, `purge` and a clock behind every cache; `Cache` stays pure.
- **A background thread that expires entries.** A task per cache, nondeterministic tests, and no gain over lazy expiry.
- **Eager per-key timers or a heap of expiry times.** More memory per entry for a `purge` that a program rarely calls.

## Consequences

- `purge` and `keys` are O(n). A program that purges often should use a short `purge_at` interval.
- Written without a build in this lane (cloud container); the first Mac gate is the first compile. A speed and
  memory row for `tools/perfbench` is a follow-up once the module builds.
