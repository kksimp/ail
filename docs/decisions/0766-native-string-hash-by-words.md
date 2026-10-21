# 0766. Native string keys are hashed eight bytes at a time

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture; performance
- **Spec rules:** type.map.1
- **Builds on:** [ADR 0568](0568-native-map-and-set-layout.md), [native-map-probe report](../design/native-map-probe.md)

## Context

`rt.map_hash_str` was FNV-1a over the bytes: 12 instructions per byte after
code generation, 64 instructions per word for wordfreq (3.4 bytes on average).
It is the largest single item in a string-keyed lookup.

## Decision

Hash a string object's data a word at a time: `h = (h ^ word) * K` per full
eight bytes, the last partial word masked to its bytes with `1 << 8r` minus 1,
the length as the seed, and one final `h ^ (h >> 32)` folded into the low bits
(the index uses the low bits) and cleared to 62 bits.

Reading the whole last word is safe. An object is at least 16-byte rounded
(`alloc`, and 16-aligned literals), so any data end has seven readable bytes
after it; the masked bytes never reach the hash.

The hash only places keys in the index. Iteration order is the entry array's
insertion order (type.map.1), `std.hash.hash64` is separate code, and nothing
else reads this value, so the change is invisible to programs.

## Result (Linux x86-64, callgrind, 100,000 words)

| | before | after |
|---|---|---|
| `map_hash` instructions per word | 64 | 44 |
| wordfreq total instructions | 27,838,487 | 25,654,385 |

## Rejected alternatives

- **Caching the hash in the string object:** no spare header word
  (native-abi.md), and growing the header changes the ABI and every literal.
- **Keeping FNV-1a:** correct and simple, 12 instructions per byte.
- **A stronger mixer per word (shift-xor-multiply):** measured no better than
  one multiply plus the final fold at this table size, and 10 instructions
  slower for short keys.

## Consequences

- `bootstrap/tests/native/run/map_str_keys.ail` runs random string-key map and
  set operations (keys of 0-40 bytes, fresh objects with equal contents,
  removals) and prints sizes, sums and iteration order; native and `ail run`
  agree on 300 seeds of 4,000 operations each.
