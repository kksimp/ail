# 1265. `std.kv`: sorted scans and all-or-nothing batches

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library
- **Spec rules:** std.kv.10-14 ([17-std](../spec/17-std.md)); extends [ADR 1098](1098-std-kv.md)
- **Number:** placeholder; the integrator assigns it and adds the index row.

## Context

`std.kv` (ADR 1098) has set, get, remove, compact and a crash-safe log. A
program that stores related keys (an order and its lines) can lose half of a
multi-key change, and cannot list the keys under a prefix in order.

## Decision

- `scan(prefix)` and `range(lo, hi)` return sorted key lists from the in-memory
  map. No second index: sorting on demand is O(n log n) and the store is
  memory-sized by std.kv.8.
- `kv.batch()` builds a `Batch` of `set` and `remove`; `Store.apply` writes it as
  one log record (marker `klen = 0xFFFFFFFF`, body length, changes, one CRC-32).
  Replay already stops at the first record that fails its checksum, so a batch is
  whole or absent with no new recovery code. The last change on a key wins.
- Not a `begin`/`commit` object: a batch holds no reads, so there is nothing to
  isolate, and a `unique` Store already has one writer.

## Consequences

A file that holds a batch record is not readable by the earlier `std.kv`, which
would see an oversized key length and cut the file there. The header stays
`AILKV1\n`; `std.kv` is days old in a public release and no seed writes batches,
so a version byte was not worth the format change.

## Rejected alternatives

- **A commit marker after plain records.** Two writes, and an old reader would
  apply an uncommitted prefix; the single record is atomic by the existing CRC rule.
- **A sorted tree or skip list index.** Faster scans, but more code and memory for
  stores that std.kv.8 already limits to memory size.
- **`begin`/`commit`/`rollback` on Store.** Needs read isolation semantics that a
  single-writer log store does not have; `std.db` covers real transactions.
- **Bump the magic to `AILKV2`.** Breaks every existing store for a feature few use.
