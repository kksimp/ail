# 0445. DEFLATE, gzip and CRC-32 are written in-house; measured throughput

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture
- **Spec rules:** [std.hash.3](../spec/17-std.md#hashing-stdhash), [std.compress.1–4, 6](../spec/17-std.md#compression-stdcompress)

## Context

ADR 0009 prefers custom tooling. DEFLATE (RFC 1951) and gzip (RFC 1952) are fixed public formats; zlib is the usual library.

## Options considered

### Option A — write from the RFCs (chosen)
- **Pros:** no dependency, no licence question, deterministic output we control, and the system `gzip` is an independent check.
- **Cons:** more code to test; compression ratio and speed must be measured against the incumbent.

### Option B — port zlib
- **Cons:** zlib's licence is permissive but a port adds attribution and a large surface for little gain.

### Option C — link the system libz
- **Cons:** violates ADR 0009 and ADR 0010's spirit; absent on some Windows setups.

## Decision

Option A. `inflate`: canonical Huffman with a 9-bit lookup table and the RFC's counting walk for longer codes; output capped at 2^30 bytes (a zip-bomb guard). `deflate`: LZ77 with 3-byte hash chains and one step of lazy matching (chain length 4 at level 1 up to 4096 at level 9), then per block the smallest of stored, fixed and dynamic Huffman codes, with length-limited complete codes so every decoder accepts them. CRC-32: slicing-by-8 tables built at compile time. Hardware CRC (ARMv8 `crc32`, x86 `pclmulqdq`) is deferred: the tables already run past 1.6 GB/s, below what the compressor needs.

Measured on Linux x86-64 (release build, one core, 32 MiB inputs):

| What | MB/s |
|---|---|
| crc32 | 1639 |
| hash64 | 5377 |
| sha256 / sha512 | 159 / 257 |
| deflate L1 / L6 / L9, mixed data | 59 / 31 / 25 |
| inflate, mixed data | 251–348 |
| inflate, text | about 870 |

On 1.6 MB of this repository's docs at level 6: ours 569,345 bytes in 0.175 s including start-up; system `gzip -6` 570,431 bytes in 0.100 s. Ratio matches; speed is within 1.8× of zlib, a target for a later pass (a faster match finder, 64-bit match compares). Not yet measured on macOS ARM64.

## Consequences

- Tests: RFC 1951 stream vectors, round trips from 0 bytes to 200 kB at levels 0, 1, 4, 6, 9, corrupt-input fuzz without panics, and cross-checks both ways with the system `gzip` (skipped with a note when absent).
- No code or test data was taken from another project.
