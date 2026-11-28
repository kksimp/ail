# 1274. `compress.zstd` and `compress.brotli` write valid frames without a match search

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** Standard library, compression
- **Spec rules:** [std.compress.15, std.compress.16](../spec/17-std.md)
- **Number:** the integrator assigns it; this file is renamed then.

## Decision

`compress.zstd(data)` and `compress.brotli(data)` take `bytes` or `str`, never fail, have no effects and give the
same bytes on every platform. They live in `compiler/rt/native/compress_zstd.ail` and `compress_brotli.ail`.

- **zstd:** one RFC 8878 frame of raw blocks, plus RLE blocks for runs of 32 or more equal bytes. No checksum. Single
  segment up to 8 MiB, else a 128 KiB window descriptor.
- **Brotli:** one RFC 7932 stream of uncompressed meta-blocks of up to 65536 bytes, then the empty last meta-block.

Both are O(n) and read by every conforming decoder, including the decoders of ADR 1240 and ADR 1241 once they land.
This gives HTTP and archive code a writer today; ratio is the work of a later level.

## Rejected alternatives

- **Compressed blocks now** (zstd sequences with predefined FSE tables, Brotli with a static Huffman prefix code and
  a hash-chain match finder). Better ratio, but each is a day of work and needs the decoders to check against; the
  frames above are the safe base to build a level on.
- **Content checksum (XXH64)** in the frame. Costs a hash routine for a check no caller asked for; the field is
  optional and can be added without changing the API.
- **A level argument.** There is one level; adding the parameter later is compatible.
- **Writing the stream in the `std/` AIL tree.** `compress` is a built-in module, so its functions are built in.

## Consequences

Output for compressible text is larger than from the reference encoders. Programs that need ratio should use gzip
(level 6) until a matching level lands.
