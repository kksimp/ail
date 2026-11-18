# 1159. `sort()` of strings sorts (first eight bytes, string) pairs

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked on 2026-11-17 for lanes on the open slowdowns against C, runtime speed only, no syntax change
- **Topic:** 25 — Optimization strategy
- **Spec rules:** none changed ([type.coll.11](../spec/04-types.md): the order is the same)
- **Builds on:** the radix sort of integers and floats (`compiler/rt/native/sort_radix.ail`)

## Context

Sorting a million short strings took 1.37 times `qsort` with `strcmp`
(Linux, `micro2-linux-8a636a55.txt`: 0.589 s against 0.431 s). `list_sort`
merges runs of string words and calls `list_word_lt` for each comparison,
which reads both strings byte by byte wherever they lie in the heap: two
cache misses and a byte loop per comparison.

## Options considered

### Option A — merge (key, string) pairs (chosen)

`compiler/rt/native/sort_str.ail`. Each string gets a key: its first eight
bytes as one big-endian word, zeros after a shorter string, top bit flipped
so signed order is byte order. The same stable bottom-up merge runs on the
pairs. Keys that differ decide the order without reading a string (a
string that is a prefix of another has a key no greater, so byte order and
"shorter first" both hold); equal keys fall back to the full comparison.

- **Pros:** most comparisons read two neighbouring words; the order and
  stability are those of the merge sort it replaces; 60 lines.
- **Cons:** 32 bytes of scratch an element instead of 8; strings that share
  their first eight bytes gain nothing.

### Option B — most-significant-byte radix sort on the strings

- **Pros:** no comparisons for short keys.
- **Cons:** recursion per bucket, small-bucket fallbacks, more code to get
  right; the gain over A is unmeasured.

### Option C — cache the key inside the string object

- **Cons:** changes the string layout (native-abi.md) for one operation.

## Decision

Option A, from 64 elements (below that the plain merge sort stays). The
language rule does not change: `sort()` orders strings by their bytes and
is stable.

ADR 0076 order: no source or diagnostic change; speed decides. Both
targets run the same runtime code.

## Consequences

- Mac arm64, 3 million short strings (`hotpaths-job-mac-98407a6fcp.txt`):
  0.765 s to 0.394 s; the C twin (`qsort`, `strcmp`) 0.740 s. Linux: `docs/status.md`.
- `sorted(by:)` and lists of lists keep the comparison sort.

## Rejected alternatives

- **Option B:** more code, unmeasured extra gain; can follow if the
  equal-prefix case matters.
- **Option C:** an ABI change.
