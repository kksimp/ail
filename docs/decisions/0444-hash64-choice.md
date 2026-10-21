# 0444. `hash64` is a wyhash-style multiply-fold hash, written from scratch

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture
- **Spec rules:** [std.hash.4](../spec/17-std.md#hashing-stdhash)

## Context

Programs need a fast non-cryptographic hash for tables, dedup and fingerprints. It must be identical on every platform and version, so stored values stay valid.

## Options considered

### Option A — 64×64→128 multiply-fold over 16-byte chunks (chosen)
- **Pros:** about 5 GB/s measured, passes length and avalanche checks (about 32 of 64 output bits flip per input bit), 40 lines, no table, no licence question since it is written here from the published idea.
- **Cons:** not a vetted function with an external test suite; not for adversarial inputs.

### Option B — xxHash64/XXH3
- **Pros:** widely known, published vectors.
- **Cons:** XXH3 is large; porting copies a BSD-2 project, which needs attribution and a licence review for an Apache-2.0 + Runtime Exception repository. No gain over A for programs.

### Option C — FNV-1a (already used internally, ADR 0175)
- **Cons:** about 1 byte per cycle, weak mixing.

### Option D — SipHash
- **Cons:** slower, and its point is attack resistance that `sha256` serves.

## Decision

Option A (speed, then readability). The output is pinned by tests; changing it needs a new ADR. Documented as not secure.

## Consequences

- Values are `int` (the 64 bits reinterpreted as signed), so they can be negative.
- No code was copied from another project, so no licence attribution applies. Test vectors come from the standards (FIPS 180-4, RFC 4231, RFC 1952's CRC).

## Rejected alternatives

- xxHash64 or XXH3 (option B): rejected because XXH3 is large, and porting copies a BSD-2 project that needs attribution and a licence review for the Runtime Exception.
- FNV-1a, already used internally (option C): rejected because it runs at about one byte per cycle and mixes weakly.
- SipHash (option D): rejected because it is slower, and its point is attack resistance, which `sha256` serves.
