# 0828. Native `std.compress`: the Rust codec ported to the runtime, byte for byte

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** std.compress.1-4, std.compress.7-8, native.subset.1

## Context

`examples/compress.ail` and `examples/gz_check.ail` were outside the native
subset because `std.compress` had no native implementation. The interpreter
runs the codec in Rust (`bootstrap/src/codec`).

## Decision

1. **Where:** `compiler/rt/native/compress.ail`, Runtime Code in AIL
   (ADR 0510), with no new global slots, trap kinds or instructions. Each
   function is a row of the table in `lower_os.ail` (`compress.gzip` to
   `rt.compress_gzip`), as `path` is (ADR 0614).
2. **Same bytes:** `gzip`, `zlib` and `deflate` are a step-for-step port of
   `deflate.rs`: hash chains of 128 candidates, one step of lazy matching,
   blocks of 32768 tokens, and per block the smallest of stored, fixed and
   dynamic Huffman, with the same tie-breaks in the code-length repair. The
   output equals the interpreter's byte for byte (std.compress.1: the same
   input gives the same output on every platform).
3. **Same failures:** `inflate`, `gunzip` and `unzlib` port `inflate.rs`,
   `gzip.rs` and `zlib.rs`, with the same messages, kinds and checks
   (std.compress.2-4, 7-8). A failure sets the error slot to
   `compress.CompressError(kind: "K", message: "M")`, the interpreter's
   display text, as `fs.IoError` does; binding `err.kind` waits for the
   error-value model.
4. **Buffers:** the compressor sizes its output once, from the input length
   (a block costs at most five bytes over its input), and uses atomic scratch
   blocks, so no pointer lives only inside a block the collector skips. The
   decompressor grows its output by doubling a string object.
5. **Arguments:** `bytes` and `str` share one layout, so `gzip`, `zlib` and
   `deflate` take either (std.compress.1); the others take `bytes`.
6. **Evidence:** `compress_basic`, `compress_stream`, `compress_corrupt` and
   `compress_fail_*` in `bootstrap/tests/native/run` compare the executable
   with `ail run` on x86-64 and under qemu on ARM64. A scratch comparison on
   11 inputs up to 8 MB found no difference; on three compressions of 8 MB
   the executable took 0.89 s against 2.4 s for `ail run`.

Not here: `gzip_file`, `gunzip_file`, `zip_dir`, `unzip`, levels other than
6 (the language fixes level 6), and `main() ! compress.CompressError`
(the error-value model).

## Rejected

- **A faster, different compressor (a fixed Huffman table only):** the output
  would differ from the interpreter's, and the spec promises one output per
  input.
- **Growing the compressor's output by doubling:** the bound is cheap and
  keeps one allocation; the cost is a slightly larger buffer than needed.
- **A fast decode table (the Rust 9-bit table):** the bit-by-bit walk is a
  few times slower on large inputs but a quarter of the code; the table is
  the follow-up if a benchmark asks for it.
