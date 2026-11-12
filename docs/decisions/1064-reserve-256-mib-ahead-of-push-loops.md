# 1064. Native: reserve up to 256 MiB of list elements ahead of a push loop

- **Status:** Accepted
- **Date:** 2026-11-11
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 23 — Implementation (native optimizer)
- **Spec rules:** [native.bce.3](../spec/23-native.md)
- **Amends:** [ADR 0816](0816-native-codegen-entry-addressing-constants.md) item 3 (the 8 MiB reservation limit)

## Context

On the sieve of 150,000,000 (Mac, main ed20486b), AIL took 865 ms against C's
645 ms. `AIL_OPT_OFF=reserve` changed nothing (878 ms). The disassembly showed
that the strike loop has no checks and the count loop matches C. The time went
elsewhere. `rt.list_new_for` clamps a reservation to `list_most`, 2^23 bytes.
The versioned fill loop's guard `n <= cap - len` therefore fails, and the
plain push loop regrows the list 8 → 16 → 32 → 64 → 128 → 256 MB. That copies
about 248 MB and touches 248 MB of extra fresh pages.

## Decision

`list_most` is 256 MiB of elements for every element width: 2^25 words, or
2^28 bytes for a narrower list. A list's capacity is not observable
(native.bce.3), and `alloc_raw` does not zero memory. So a reservation that
goes unused costs address space and collector accounting only.

Result (Mac): sieve 150M went from 847.9 ms to 709.1 ms, and peak RSS from
393 MB to 145 MB.

## Rejected alternatives

- **1 GiB.** Loops that leave early would reserve far more than they fill.
- **A different growth factor.** The spec says doubling, and any factor still copies.

## Not covered

The nbody gap is not addressed. ARM64 has no base + index + immediate
addressing, so `s[bi + k]` costs an `add` per access.
