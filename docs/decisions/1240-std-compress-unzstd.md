# 1240. `compress.unzstd`: an in-house Zstandard decoder

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 — Standard library (`std.compress`)
- **Spec rules:** [std.compress.14](../spec/17-std.md)

## Context

Zstandard (RFC 8878) is the compression of `.zst` files, `.tar.zst` archives, package formats and HTTP
`Content-Encoding: zstd`. `std.compress` read DEFLATE, gzip, zlib and zip only, so a program given zstd data
had no way in. The std gap plan ranked Brotli and zstd decompression together (wave 3); Brotli is a separate
lane in the same module.

## Decision

- `compress.unzstd(data bytes) bytes ! compress.CompressError`, no effects, beside `unzlib` and `gunzip`.
- Every frame in a row is decoded and the outputs appended; skippable frames are passed over. Each frame's
  content size and its XXH64 content checksum are checked when the frame carries them.
- The whole output stays in memory, so the window size needs no check; `std.compress.4`'s 2^30-byte limit
  applies to the total, and a frame that declares a larger content size fails before anything is allocated
  for it.
- Frames that need a dictionary fail with `unsupported`. Failures reuse the module's kinds, and the messages
  are a closed list in the rule, as ADR 0784 does for DEFLATE.
- The decoder is written from the RFC in `compiler/rt/native/zstd.ail` (prefix `zst_`), sharing
  compress.ail's word helpers and failure words. It was first written as a Python model, checked against the
  reference library's output (levels -10 to 22, checksums on and off, content sizes on and off, several
  frames) and against corrupted inputs, then the native file was checked line for line against that model.

## Rejected alternatives

- **Linking libzstd.** No C libraries in the toolchain or the runtime (ADR 0009), and the runtime must build
  the same way on the three hosts.
- **Writing it in AIL under `std/`.** The decoder is byte and bit work in hot loops over raw buffers, the
  case the plan reserves for the runtime ("only what needs the OS or real speed").
- **A compressor too (`compress.zstd`).** Reading is what programs need (downloads, archives, HTTP); an
  encoder that matches the reference's ratios is a much larger job. Left for a later change.
- **Dictionary support.** It needs an API for passing dictionaries and no current use asks for it; such
  frames fail with `unsupported` and a clear message instead of `corrupt`.
- **A streaming API.** The other byte functions take and return whole `bytes`; a stream reader for all
  codecs is its own decision.
- **Advertising `zstd` in the HTTP client's `Accept-Encoding` (ADR 0717).** A follow-up once this lands.
