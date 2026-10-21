# 0769. Native string hash: a second multiply as the finalizer

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture; performance
- **Spec rules:** type.map.1
- **Builds on:** [ADR 0766](0766-native-string-hash-by-words.md)

## Context

ADR 0766 ended the string hash with `h ^ (h >> 32)`. Multiplying by an odd
constant moves information only upward, so keys that differ only in their high
bytes (`key0` to `key19999`: the digits sit in bytes 3 to 7 of one word) land in
a few index slots. Simulated at the map's 32,768-slot index, 20,000 such keys
average 50 probes (longest 423); `map_ops` spent 48% of its 500M instructions in
`rt.map_probe`. Keys of 12 to 20 bytes had the same fault (11 probes on average).

## Decision

Finish the hash with `h = (h ^ (h >> 32)) * K` and `h ^ (h >> 29)`, then clear
to 62 bits. Three more instructions per hash. The same keys then average 1.75
probes (longest 47), the figure for a uniform hash at this load.

Still invisible to programs: the hash only places keys in the index, iteration
is insertion order (type.map.1), and `std.hash.hash64` is separate code.

## Rejected

- A stronger per-word mix (a multiply and a shift per word): slows every word
  to fix the end, which one finalizer already does.
- Probing by a stride instead of linearly: hides the clustering, doesn't remove it.
