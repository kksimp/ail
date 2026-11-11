# 0936. Native string-keyed maps cache keys by object, and an insert reuses the missed probe's hash

- **Status:** Accepted
- **Date:** 2026-11-04
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture; performance
- **Spec rules:** type.map.1, type.map.3
- **Builds on:** [ADR 0568](0568-native-map-and-set-layout.md), [ADR 0766](0766-native-string-hash-by-words.md), [ADR 0767](0767-map-first-probe.md), speed-edges proposal P9

## Context

A `Map[str, V]` lookup hashes the key's bytes every time (`rt.map_hash_str`,
44 instructions per word in wordfreq) before the probe. Programs look up the
same string objects again and again: literals, a vocabulary list, names held
in a syntax tree. The proposal (P9) was to cache each string's hash in the
string object. An insert also hashed twice: once in the probe that missed,
once in `map_append`.

## Decision

1. **A key cache per `str`-keyed table.** The table object grows to 360
   bytes and holds 32 entry addresses at +104, slot chosen by the key
   object's address. `rt.map_find` on `str` keys reads the slot first; a
   slot counts only when its entry's key is that very object, which needs no
   hash and no byte compare. The entry keeps the object alive, so no other
   string can have its address, and strings never change, so the hit is
   exact. A removal clears the entry's key (the check fails); a rebuild and a
   copy clear the cache. Every probe hit and every insert fills its slot.
2. **An insert reuses the hash of the probe that just missed.** Every table
   (104 bytes now) keeps the last missed key and its hash at +88 and +96;
   `map_set`, `map_add` and `map_counts` take the hash from there. The table
   holds the key, so an address names one object. Composite keys (kind above
   15) always rehash: their objects are not immutable.

Nothing a program can observe changes: iteration is insertion order
(type.map.1) and the hash only places keys in the index.

## Result

ail-linux, wordfreq 10M words (`tools/perfbench`), min of 7, outputs equal:

| | base | this ADR | C (gcc -O2) | Python |
|---|---:|---:|---:|---:|
| time | 0.27 s | 0.22 s | 0.15 s | 2.50 s |
| instructions | 1.649G | 1.452G | | |

On the Mac (Apple Silicon, same build and protocol): base 0.097 s, this ADR
0.062 s, C 0.150 s.

The first version (16 slots by `address >> 4`, a helper call per lookup)
missed most lookups and ran 9% more instructions; the slot from the
multiplied address and the hit written out in `map_find` fixed both.
Stage-2 self-build 26.2 s to 26.6 s (noise range).

**Amended 2026-11-11 (delegated, ADR 0076):** the slot is now `address >> 5`
modulo 32, not the top five bits of the address times an odd constant. With
the hash, which of a vocabulary's literals collided depended on where the
linker put them: an x86-64 register change (ADR 1049) that only moved code
made wordfreq run 11.7% more instructions, all of it in the probe and hash
paths. String objects are at least 32 bytes and 16-byte aligned, so `>> 5`
puts keys made one after another in different slots until they span 1 KB.
The first version's `>> 4` over 16 slots used every other slot, which is why
it missed. Rejected: keeping the hash (layout decides speed); a hash of the
1 KB block added to `>> 5` (protects keys exactly 1 KB apart, a rare
pattern, at the cost of collisions across a block boundary).

## Rejected alternatives

- **The hash in the string object, in the top 32 bits of `chars`** (the
  proposal's choice): about a hundred runtime sites read `chars` with
  `load64(p + 8)` (and `bytes` objects share the layout), each would need a
  mask, and a missed one gives wrong lengths silently. Literals are in
  read-only data, so the compiler would also have to precompute their hashes
  with the runtime's exact function.
- **A hash word in the string header** (`+16 hash`, data at +24): moves the
  data of every string, every literal, FFI's `rt_str_new` and every
  `p + 16` in the runtime; the ABI change ADR 0766 already rejected.
- **A hash word after the data:** its offset depends on `bytes`, and a
  `bytes` object turned into a string in place has no room for it.
- **Extending the one-entry last-key cache to a list searched in order:**
  linear in its size; the direct-mapped slot is one load and one compare.

## Consequences

- Fresh string objects with equal contents (the words of a split line) still
  hash once per lookup; the cache helps repeated objects only.
- `tests/native/run/map_key_cache.ail` checks lookups of the same and of
  equal objects through growth, removals and copies against a list oracle.
