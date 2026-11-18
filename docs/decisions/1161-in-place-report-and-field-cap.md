# 1161. `ail perf --explain` says which structs lists keep in place; the cap is 16 fields

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked on 2026-11-17 to keep closing the speed gaps to C without syntax changes
- **Topic:** 25 — Optimization strategy
- **Spec rules:** [tool.perf.1](../spec/22-tools.md), [native.flat.1](../spec/23-native.md)
- **Amends:** [ADR 1149](1149-structs-kept-in-place-in-lists.md) and [ADR 1156](1156-structs-with-pointers-kept-in-place.md) (the field cap), [ADR 1021](1021-perf-explain.md) (the report)

## Context

Whether a `List[S]` keeps its elements in place is decided for the whole
program (native.flat.1): one use anywhere that needs objects boxes every list
of `S`. Two things about that were left open.

1. **Nothing said what was decided.** A `pop()` in one function halves the
   speed of a loop in another, and neither the programmer nor an agent could
   see it without reading MIR.
2. **The cap of eight fields was a guess.** ADR 1149 chose it without a
   measurement.

## Decision

1. `ail perf --explain FN FILE` prints, before the loops of `FN`, one line
   for each struct the program has a list of:
   - ``lists of `S` keep their elements in place (native.flat.1)`` (or
     `native.flat.2` when a field is a pointer), or
   - ``lists of `S` hold one object per element: REASON (native.flat.1)``,
     where `REASON` is the first one found: the struct's shape (its field
     count against the cap, a field type not kept in place), or the function
     and the use that needs objects (a list method, a list literal, `==`,
     text).

   The lines are the same for every `FN`; they are facts of the program.
   Nothing is recorded unless the report was asked for.
2. A struct is kept in place when it has one to **16** fields (was eight).
   Nothing else in native.flat.1 and native.flat.2 changes.

## Measurements

Mac (arm64, 8 cores), main 7661fb5a against the same tree with the cap at
31, 400,000 elements, best of 5 interleaved runs. `n` is a struct of floats,
`t` has one text field. `scan` builds the list, reads two fields 40 times
and every field 8 times; `swap` builds it and exchanges elements end for end
8 times. Wall ms, then peak memory in MB.

| fields | scan, objects | scan, in place | swap, objects | swap, in place |
|---|---|---|---|---|
| 12 n | 45.4, 50 | 42.8, 38 | 79.8, 189 | 15.5, 38 |
| 12 t | 44.8, 50 | 44.4, 38 | 81.1, 189 | 18.6, 38 |
| 16 n | 56.6, 61 | 69.9, 50 | 127.9, 237 | 20.0, 50 |
| 16 t | 93.1, 61 | 75.2, 50 | 128.9, 237 | 25.7, 50 |
| 24 n | 110.3, 85 | 92.2, 154 | 151.3, 335 | 67.2, 295 |
| 24 t | 113.1, 85 | 91.4, 154 | 151.4, 335 | 76.0, 295 |
| 31 n | 109.0, 104 | 124.7, 194 | 160.8, 409 | 88.3, 381 |
| 31 t | 108.0, 104 | 123.2, 194 | 160.7, 409 | 98.8, 381 |

Up to 16 fields the in-place list uses less memory in every row and a write
of an element is five to six times faster. Reads are level, except `scan` of
16 floats, which is 24% slower in place; the cause is not known (the same
loop over 16 fields with a text is 19% faster), and it is small beside the
write and memory gains. From 24 fields the in-place list uses nearly twice
the memory in `scan` and the write gain falls to about two times; at 31
fields reads are 14% slower. The compiler builds itself in the same time
under either cap (32.5 s and 32.8 s), and the fixed point holds at 31.

## Consequences

- An agent or a programmer asks one command why a list of structs is slow
  and gets the function and use to change.
- Structs of 9 to 16 fields in lists lose an allocation and a pointer per
  element.
- `tests/native/run/flat_refs.ail` covers a 16-field struct with a text and
  a list; the tool.perf.1 example checks the report line.
- The 24% `scan` row is open: find why a read-only loop over a 128-byte
  element is slower in place.

## Rejected alternatives

- **A warning or note from `ail build`:** boxing is not a mistake, and most
  programs would print it on every build. The report is for whoever asks
  about speed, and `ail perf --explain` is where they ask.
- **A line in `ail mir`:** MIR is not a stable or documented output, and
  agents are told to use `ail perf --explain`.
- **Keep eight:** costs five to six times on writes and 12 MB to 190 MB in
  the rows measured, for structs of 9 to 16 fields.
- **Cap at 24 or 31 (the most the block kind can hold):** peak memory of a
  read-mostly program nearly doubles and the gain on writes is under half of
  what 16 gives.
- **A cap in bytes:** every field is 8 bytes in a struct object and in
  place, so bytes and fields are the same measure.
