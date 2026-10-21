# 0767. The native map lookup tries one probe without saving registers

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture; performance
- **Spec rules:** type.map.1
- **Builds on:** [ADR 0766](0766-native-string-hash-by-words.md)

## Context

`rt.map_find_h` held the map, key, hash, mask, index, entries and key kind live
across the inlined key comparison, so every call saved five callee-saved
registers and spilled four values to the stack: about 25 of its 69
instructions on the common first-probe hit.

## Decision

`map_find_h` does the first probe only: the slot is empty (not found), or its
entry has the same hash and the same key object (found). Anything else calls
`map_probe(m, k, h, i)`, the full loop from that slot with the content
comparison. The fast path calls nothing but that final tail call, so it saves no
registers.

## Result (callgrind, 100,000 words)

| | before | after |
|---|---|---|
| `map_find_h` instructions per word | 69 | 43 plus 22 in `map_probe` on collisions |
| wordfreq total instructions | 25,654,385 | 25,479,264 |

Together with ADR 0766: 27,838,487 to 25,479,264 (-8.5%). Wall-clock on the
shared box moved within noise (best of 21: 0.039 s before, 0.041 s after; median
0.053 s and 0.047 s). The gain here is small because the first probe collides
for about a quarter of the 12 keys, and keys built at run time (equal content,
different object) always take the second function.

## Rejected alternatives

- **Inlining the content compare in the fast path:** brings back the saved
  registers it exists to avoid.
- **A bigger index (lower load factor) to cut collisions:** costs memory for
  every map; the index is already twice the entry capacity.

## Consequences

- No change to behaviour; same native tests as ADR 0766.
- The remaining per-call cost in `map_find`, `map_set` and `main` is code
  generation (argument shuffles at every function entry, loop-invariant
  constants rebuilt in loops); see the report for lane 120.

## Update (2026-10-21)

`map_find_h` and `map_probe` are gone: `map_find` computes the hash and runs the
probe loop itself, so a lookup is one call (plus `map_key_eq` for keys that are
not the same object). Strings of 1 to 8 bytes hash in one word without the loop
(same value as the general path). `map_ops`: 279.7M to 213.6M instructions;
`wordfreq` 1M: about 217M to 203.0M.
