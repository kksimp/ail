# 1197. `bytes` loops read in place; a joined `List[char]` map key is looked up before the string is made

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer's speed mandate of 2026-11-17 (runtime speed only, no syntax change) and the Mac fast track of 2026-11-25
- **Topic:** Native lowering and runtime
- **Spec rules:** native.map.1 (amended); type.bytes.5 and expr.loop.3 unchanged
- **Builds on:** [ADR 0412](0412-bytes-type.md) (`bytes`), [ADR 0825](0825-native-ascii-join-chars.md) (ASCII join), [ADR 1148](1148-map-lookups-compiled-into-the-caller.md) (map lookups in the caller)

## Context

A hands-on trial wrote nine programs in AIL and four C twins. Two gaps were
in ordinary code:

- A hash over the bytes of each line (`for b in bs` inside a loop of rounds)
  ran 2.6 times as long as C. `for b in bs` over a `bytes` called
  `rt.bytes_list`, which allocates a `List[u8]` and copies the bytes into
  it, every time the loop started. On the base, 60% of the samples were in
  `list_copy`, `list_alloc`, `bytes_list` and the allocator, and the
  collector took most of the rest.
- A word count built each word in a `List[char]` and counted it with
  `counts[cur.join("")] += 1`. The join already has a one-pass ASCII path
  (ADR 0825); what was left was one string allocated for every word of the
  input, hashed, compared and dropped: 431 MB resident for a 107 MB file.

## Options considered

### Byte loops

- **A: read the bytes in place (chosen).** `bytes` is an immutable value, so
  the loop keeps the object's address and length and loads byte `i`
  (`compiler/mir/lower_bytewalk.ail`). An assignment to the variable in the
  body cannot change what the loop reads. `for mut` keeps the copy, which
  its writes go to.
- **B: hoist the copy out of the enclosing loop.** One copy per call is
  still an allocation the loop has no use for.

### Joined keys

- **A: ask the map first (chosen).** For `xs.join("")` written as the key of
  a lookup or store on a `str`-keyed map, `rt.mapkey_chars` hashes the
  characters as `map_hash_str` would hash the joined bytes, probes, and
  returns the map's own equal key, noted as the last key found so the
  lookup that follows hits in the caller. The string is made only for a key
  the map lacks, a character past ASCII, or an empty list. Strings never
  change and have no identity, so the map's key stands for the joined text.
  A hash that disagreed with `map_hash_str` would lose the speed and
  nothing else: the miss joins and takes the ordinary lookup.
- **B: a string builder type.** New surface for programs to learn; the list
  of characters is what agents already write.
- **C: make short-lived strings cheaper in the allocator.** Helps every
  program a little and this one less: the string is still built, hashed and
  compared against the stored key.

## Decision

Both A. No syntax and no observable behaviour changes: results, iteration
order and traps are those of the copy and of the join.
`AIL_OPT_OFF=bytewalk` and `AIL_OPT_OFF=joinkey` keep the old code.

## Rationale

ADR 0076 order: nothing changes for the reader or the diagnostics, so speed
decides, and both remove work the program never asked for.

## Consequences

- Tests `bytes_walk` and `map_join_key`, each also built with its fast path
  off.
- Mac (Apple Silicon, 2026-11-26, `bytejoin-mac-96054b10.txt`, branch
  against main 78e27c83, best of 5, C twins with `cc -O2`):

  | Program | main | branch | C | branch MB | main MB |
  |---|---|---|---|---|---|
  | byte hash, 60,000 lines, 200 rounds | 0.887 s | 0.670 s | 0.669 s | 19 | 39 |
  | the same through a pool of 1 worker | 1.503 s | 0.958 s | | 26 | 43 |
  | the same through a pool of 4 workers | 2.321 s | 0.266 s | | 26 | 44 |
  | word count, 107 MB | 0.731 s | 0.533 s | 0.492 s | 116 | 431 |

  The byte hash runs 4,679 M instructions against 14,097 M before (C:
  2,932 M) in the same cycles as C: the loop is bound by the multiply.
  The pool of 4 was slower than 1 because every worker allocated in the
  loop and the collector stopped them all.
- Not covered: `for i, b in bs.enumerate()` and comprehensions over a
  `bytes` still copy; a key joined in an earlier statement
  (`w := cur.join("")`, then `counts[w]`) is still made.

## Rejected alternatives

- **Hoisting the copy:** keeps an allocation with no use.
- **A string builder:** new surface where the existing form can be fast.
- **Cheaper short-lived strings alone:** leaves the build, hash and compare.
