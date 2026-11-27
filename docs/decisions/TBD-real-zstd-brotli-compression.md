# TBD. `compress.zstd` and `compress.brotli` compress: LZ77, Huffman and FSE, levels 1-9

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked that the std really work, not only compile
- **Topic:** Standard library, compression
- **Spec rules:** [std.compress.15, std.compress.16](../spec/17-std.md)
- **Amends:** [ADR 1274](1274-zstd-brotli-writers.md) (writers with no match search)
- **Number:** the integrator assigns it; this file is renamed then.

## Context

ADR 1274 shipped writers of valid frames with raw and RLE blocks only, so 5 MB of one repeated line stayed 5 MB. A
compression function that does not compress does not meet `std.compress`'s purpose (HTTP bodies, archives, files).

## Decision

`compress.zstd(data, level)` and `compress.brotli(data, level)` take an optional `level`, 1 (fastest) to 9
(smallest), 0 or out of range meaning 3. Both share one match finder, `compiler/rt/native/compress_lz.ail`:
LZ77 over hash chains of four-byte keys with a ring of chains, a 1 MiB window (1 MiB less 16 bytes for Brotli),
greedy at level 1 (one candidate), lazy by one step from level 3, chain depth 2^(level-1).

- **zstd:** blocks of up to 128 KiB, each the smallest of raw, RLE and compressed. Compressed: literals are Huffman
  coded (lengths from the DEFLATE length-limited builder at 11 bits, weights written directly, so only when the
  largest literal byte is at most 128; one stream up to 1 KiB, else four), sequences are FSE coded over the
  predefined tables with offset values of distance + 3 (no repeat offsets). No checksum. Frames over 8 MiB declare
  a 1 MiB window; smaller ones are single segment.
- **Brotli:** meta-blocks of up to 256 KiB, each the smaller of compressed and uncompressed. Compressed: one block
  type per category, one prefix code each for literals, insert-and-copy commands and distances (simple codes for one
  or two symbols, complex codes with run-length coded lengths otherwise), explicit distances only, no context map,
  no static dictionary.
- The `rt` functions are `compress_zstd(data, level)` and `compress_brotli(data, level)`, with `level` an optional
  second argument in the checker and lowering (`compress.zstd(data)` is unchanged).

## Evidence

Round trips of 143 inputs (sizes 1 to 1.1 MB around every block boundary; text, UTF-8, noise, and
mixed runs and copies) at levels 1, 3 and 9, 429 streams per codec: all read back by the reference Python decoders
(`zstandard`, `brotli`) and by `compress.unzstd` and `compress.unbrotli`, no failure. Sizes and times on a 4-core
cloud container, `ail` compiled natively, reference encoders called through Python (C):

| input | codec, level | ours (bytes, ms) | reference (bytes, ms) |
|---|---|---|---|
| 4.0 MB AIL source | zstd 1 | 1,552,961, 66 | 1,173,587, 11 |
| | zstd 3 | 1,264,382, 106 | 1,044,972, 17 |
| | zstd 9 | 1,093,241, 2,285 | 888,338, 66 (level 9) |
| | brotli 3 | 1,034,288, 156 | 1,067,557, 32 (quality 3) |
| | brotli 5 | 967,380, 313 | 913,285, 76 (quality 5) |
| | brotli 9 | 909,899, 1,875 | 845,615, 240 (quality 9) |
| 4.7 MB log text | zstd 3 | 948,549, 80 | 769,033, 10 |
| | brotli 5 | 678,917, 193 | 639,028, 62 |
| 6 MB binary | zstd 3 | 2,410,494, 242 | 1,979,762, 22 |
| | brotli 5 | 1,975,042, 538 | 1,745,831, 134 |
| 5.2 MB one repeated line | zstd 3 | 488, 27 | 6,188 |
| | brotli 3 | 284, 31 | 53 |

Ours are 10 to 25 per cent larger than the reference at a level of similar speed class for zstd (predefined FSE
tables, no repeat offsets, a simple match finder) and 5 to 10 per cent larger for Brotli, and 4 to 10 times slower
(native AIL against tuned C).

## Rejected alternatives

- **Dynamic FSE tables and repeat offsets now.** They close most of the zstd ratio gap but need count normalisation
  and the FSE table description writer; left for a follow-up, the frames above already compress well and every decoder reads them.
- **FSE-coded Huffman weights** (needed for literal bytes above 128): such blocks keep raw literals, which still
  compress through matches; a follow-up can add the weight coder.
- **Block-split and context-modelled Brotli** (several block types, literal context maps, the static dictionary): large
  gains on text but a large encoder; the single-tree meta-block is valid and is within 10 per cent of the reference
  at quality 5.
- **A separate `compress.zstd_fast`**: an optional `level` covers it with one name.
- **Level 10 and up** (optimal parsing): too slow in a function with no streaming interface.

## Consequences

`compress.zstd(data)` and `compress.brotli(data)` now return smaller output than before for compressible data and
never more than a few bytes over for incompressible data. Levels above 5 trade time for size roughly 2 to 8 times
per step pair, as the table shows. Streaming and a dictionary are not offered.
