# TBD. `std.image.webp`: lossy (VP8), alpha (ALPH) and animation (ANIM/ANMF) decode

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for a fuller std on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [std.webp.11-20](../spec/std-webp.md)
- **Number:** the integrator assigns it; rename this file and the links when it does.
- **Depends on:** [TBD-std-image-gif-webp.md](TBD-std-image-gif-webp.md) (the lossless decoder and the container scan); merge after it.

## Context

[TBD-std-image-gif-webp.md](TBD-std-image-gif-webp.md) decodes lossless WebP only and refuses lossy and animated files with
`Unsupported`. Most WebP files on the web are lossy (often with an alpha plane) or animated, so `webp.decode` failed on
the files programs meet most. The designer's rule is that a module must work, not only compile.

## Decision

- `decode` also reads a `VP8 ` key frame: the boolean arithmetic decoder, the frame header (segments, loop filter with
  deltas, 1 to 8 token partitions, quantizers, probability updates), intra prediction (16x16, 8x8 chroma and the ten 4x4
  modes), the inverse WHT and DCT, the loop filter run after the frame, and YUV to RGB with libwebp's constants and its
  fancy chroma upsampler. The tables and the structure follow RFC 6386 and the BSD-licensed Go `x/image/vp8` decoder (tables
  reused, code written fresh in AIL).
- `ALPH` (raw or VP8L-coded, with the four prediction filters) gives the alpha of a lossy image; `VP8L` frames keep their own.
- `decode_animation` returns an `Animation { width, height, loops, frames }` with every frame as the whole canvas and its
  `delay_ms`, composed as libwebp's `AnimDecoder` does (dispose to background, no-blend, the key-frame rule, and the
  non-premultiplied blend). `decode` of an animation returns the first frame; a still picture is one frame with delay 0.
  `info` fails with `BadChunk` for `ANMF` without `VP8X`.
- Same errors as before (`WebpKind`), never a trap, work bounded by `image.MAX_PIXELS` (also over all frames).
- Checking: the decoder was first written as a Python mirror and compared with `dwebp`, `anim_dump` and Pillow over more
  than 1,200 generated files (all prediction modes, segments, simple and normal filters, sharpness, deltas, 1 to 8 partitions,
  skip flags, sizes from 1x1, raw and coded alpha with every filter, and animations built with `webpmux`). Then the AIL text
  itself was run through a throwaway AIL-to-Python transpiler and gave the same pixels, and fuzzed (flipped and cut files)
  with no failure other than `WebpError`. The goldens in the tests are libwebp's output, never this decoder's. The first Mac
  gate is the first AIL compile, so type and `mut` errors are possible until it runs.
- APNG is not added: `std.image.png` (PR #1515) has no `acTL`/`fcTL`/`fdAT` handling, and the PNG file belongs to its own lane.
  An `std.image.apng` module on the PNG module's internals is a follow-up once that lane's file is merged.

## Rejected alternatives

- **Lossy only, no animation.** Animated WebP is common in chat content, and it reuses the lossless and lossy paths; stopping
  at stills would leave `decode_animation` for a later change with the same container code.
- **Premultiplied or "fast" blending.** libwebp's `AnimDecoder` blends not premultiplied, and the tests compare with it.
- **The simple (nearest) chroma upsampler.** `dwebp` and browsers use the fancy one, so pixels would differ from libwebp.
- **Inter frames and `show_frame` 0.** WebP uses only key frames; `Corrupt` and `Unsupported` say so.
- **Decoding the loop filter inline per macroblock.** It must run on the finished neighbours as libwebp does; doing it after the
  whole frame is simpler and gives the same pixels.
- **A speed pass now.** The decoder is written for clarity (lists of ints); a benchmark at the gate decides what to tune.
