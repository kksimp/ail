# 1148. Map lookups are compiled into the caller, and a table's slot is the hash's top bits

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked on 2026-11-17 for lanes on the speed gaps still open
- **Topic:** 25 — Optimization strategy
- **Spec rules:** [native.map.1](../spec/23-native.md)
- **Builds on:** [ADR 0568](0568-native-map-and-set-layout.md) (the table), [ADR 0936](0936-native-map-key-cache.md) (key cache, kept hash of a miss), [ADR 0391](0391-native-backend-optimisations.md) and [ADR 0940](0940-inliner-cost-model.md) (what goes inline)

## Context

The round 2 measurements on ail-linux (2026-11-17, main 855a6a78) put the
counting update `c := m.get(k) else 0` then `m[k] = c + 1` on an `int` key at
4.36 times a table written by hand in C (`mapinc`: 0.532 s against 0.122 s
for 20 million updates over 50,000 keys). The table itself already avoids a
second probe: `rt.map_find` leaves the entry it found in the table and
`rt.map_set` of the same key stores through it. The cost is around it. An
update makes three calls (`rt.map_find`, `rt.map_probe` inside it, and
`rt.map_set`), because the runtime's own functions go inline only when they
call nothing (ADR 0391, ADR 0940) and each of these calls another; each call
reads the key kind and the remembered key again. The C loop has no call.

Removing the calls alone gained 7% on the Mac (2026-11-18, `mapinc` 222 ms to
206 ms, C 63.5 ms). The rest is the hash. The keys of `mapinc` are the
integers below 50,000; the table took the low 17 bits of a mixed product as
the slot, which scatters them like random keys: one lookup in six finds
another key in its first slot (simulated: 83.3% first-slot hits, 1.24 probes
a lookup). The C table takes the top bits of `k * 2^64 / phi`, which puts
keys that follow one another in different slots: every lookup hits at once.
A probe branch that is wrong one time in six, and is only known after two
dependent memory reads, costs far more than the instructions around it.

## Options considered

### Option A — the lowering writes the hit path into the caller (chosen)

`compiler/mir/lower_mapfast.ail` emits, for a word key, the hash, the table
reads and the first two index slots as MIR; anything else calls
`rt.map_probe_from` with the hash already computed. For a `str` key it emits
the key cache check and calls `rt.map_find` otherwise. A store reuses the
entry of the lookup before it when no call lies between them.

- **Pros:** no call on a hit; the key kind is known at compile time, so the
  kind tests go; the runtime and the table layout are unchanged; one knob
  (`AIL_OPT_OFF=mapfast`) gives the old code for comparison.
- **Cons:** the lowering now reads the table's layout (offsets 24, 40, 48,
  72, 80 and 104), which `map.ail` documents and a change there must follow;
  about 35 instructions per lookup site instead of a call.

### Option B — small runtime functions that call nothing, left to the inliner

- **Pros:** the layout stays in one file.
- **Cons:** the hit path with its constants is at or over the 40-instruction
  limit of ADR 0391 before folding, so whether it goes inline would depend on
  that count; a leaf cannot call the slow path, so the lowering needs the
  branch anyway.

### Option C — let runtime functions that call others go inline (lift ADR 0940's rule)

- **Pros:** helps every runtime call.
- **Cons:** `expand_wide` keeps runtime functions out for a reason given
  there (result objects of `sret_convert`); a far wider change than this gap needs.

### Option D — a new table layout (values in the index, or 4-byte index slots)

- **Pros:** one cache line per hit instead of two.
- Still open after this ADR.
- **Cons:** iteration in insertion order (type.map.1) needs the entries
  array; every reader of the layout changes. Not needed to remove the calls;
  left for a measurement after this.

### Option E — keep the low-bit slot (rejected)

- **Pros:** no layout change.
- **Cons:** keys in a row, the most common integer keys (ids, indexes,
  counters, multiples of a step), collide as often as random keys do.

## Decision

1. A lookup of a key stored as a word hashes it as `rt.map_hash_int` does and
   probes the first two index slots in the caller. A hit needs the entry's
   hash and key to equal the lookup's (a removed entry's hash is -1). Any
   other outcome calls `rt.map_probe_from(m, k, hash, first slot)`, which
   finishes the probe and records a miss for the insert that may follow.
2. A lookup of a `str` or `bytes` key reads the key cache slot of the key
   object and trusts it when its entry holds that object, as `rt.map_find`
   does; otherwise it calls `rt.map_find`.
3. A hit that a plain store may follow (`get`, `m[k]`, `get_or`, `contains`
   on a map) is written to the table as the last key found, as before, so a
   later `rt.map_set` of the key stores without a probe.
4. `m[k] op= v` stores through the entry its own lookup returned.
5. `m[k] = v` after `x := m.get(k) else ...` stores through that lookup's
   entry when the two are in one basic block, name the same table and key
   values, and no call was emitted between them. A missing key (entry 0)
   calls `rt.map_set`.
6. The runtime's own code and composite keys (ADR 0866) keep the calls.
7. A table's index slot is the top bits of the 62-bit hash: `hash >> shift`,
   with `shift` a new header word at +104 (the `str` key cache moves to
   +112). `rt.map_hash_int` is the top 62 bits of `(k ^ (k >> 32)) * C`, with
   `C` the constant already used (2^64 / phi, odd). Simulated on a 2^17-slot
   table with 50,000 keys: keys 0..49999, multiples of 4096, and keys that
   differ only above bit 40 all hit their first slot every time; random
   40-bit keys are unchanged (81.4% against 81.6%). The `str` and composite
   hashes end in a multiply, so their top bits depend on every input bit.
   Iteration order is insertion order and does not change (type.map.1).

## Consequences

- No call on a hit for the counting idioms; the miss and insert paths are
  the runtime's, unchanged.
- A change to the table's header layout must be made in `lower_mapfast.ail` too.
- `tests/native/run/map_fast_paths.ail` checks word, `str`, `bool` and `char`
  keys against a list kept beside the map, and that a call between a lookup
  and a store cancels the reuse.
