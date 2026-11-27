# TBD. `std.image.gif` and `std.image.webp`: GIF decode with frames and lossless WebP decode

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for a fuller std on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [std.gif.1-10](../spec/std-gif.md), [std.webp.1-10](../spec/std-webp.md)
- **Number:** the integrator assigns it; rename this file and the links when it does.
- **Depends on:** [TBD-std-image-png.md](TBD-std-image-png.md) (`std.image.Image`); merge after it.

## Context

`std.image` holds an RGBA8 `Image` and `std.image.png` decodes and encodes PNG. The other two formats a program meets
most are GIF (stills, and animations in chat and web content) and WebP (the lossless variant is what tools export for
graphics with transparency). Both are small enough to write in AIL, as PNG was.

## Decision

- `std.image.gif` (`std/image/gif.ail`, module `gif`): `decode` gives an `Animation { width, height, loops, frames }`; each
  `Frame { img, delay_ms }` is the whole canvas as it looks when the frame is shown, so a caller needs no compositing.
  Disposal methods 0 to 3 are applied between frames (2 clears to transparent, 3 restores the previous canvas), a frame
  rectangle is clipped to the canvas, transparent indexes keep the canvas pixel, interlace and local tables are read, and
  `loops` is the NETSCAPE2.0 count (1 when absent). `first_frame` is the still picture; `info` reads the header only.
- `std.image.webp` (`std/image/webp.ail`, module `webp`): `decode` reads a RIFF container with a `VP8L` chunk (with or without
  `VP8X`) and implements the whole lossless bitstream: simple and normal prefix codes, LZ77 with the 120 distance codes,
  the color cache and meta prefix images, and the predictor (all 14 modes), cross-color, subtract-green and color-indexing
  transforms. `info` also reads lossy and animated files' sizes, which `decode` refuses with `Unsupported`.
- Both give `image.Image` (RGBA8), return `GifError` / `WebpError` with an enum `kind` (type.enum.7), never trap on bad
  input, and bound the work by `image.MAX_PIXELS` (and, for GIF, by that many pixels in all the frames).
- Both are implicit modules (std.use.4) under the names `gif` and `webp`; no checker, runtime or lowering work, no syntax change.
- Checking: the algorithms were written first as a Python mirror and compared with Pillow's GIF reader and libwebp over
  more than 480 generated images (all predictor modes, the color cache, meta prefix images, three palette packings, 109
  of the 120 distance codes); the AIL is a line-by-line port. The AIL itself is first compiled by the Mac gate.

## Rejected alternatives

- **Frames as raw sub-rectangles with offsets and disposal flags.** The caller would redo the compositing, which is the
  subtle part (disposal 3, transparency, clipping); a viewer that wants the rectangles can be given them later.
- **Lossy WebP (VP8) now.** A VP8 decoder (boolean decoder, intra prediction, DCT, loop filter, and the YUV to RGB step) is
  a second module of the size of the rest of this one; `Unsupported` with an exact reason and `info` for its size cover
  what a program needs to decide.
- **Animated WebP now.** It needs the ANMF frame and blend rules over both lossy and lossless frames; the lossless-only
  case can be added on top of `decode` when asked.
- **A GIF or WebP encoder.** Decoding is what programs lack; an encoder (LZW is easy, VP8L needs a real optimiser to compete
  with libwebp) can follow when a program asks.
- **Table-driven prefix decoding in `webp` now.** The bit-at-a-time canonical decode is simple and checked against libwebp;
  a lookup table is a speed change to make with a benchmark at the gate.
- **Failing on a GIF with a short image stream.** Browsers draw what is there; this module reports `Truncated` instead, so a
  caller knows the file is cut (a lenient mode can be added if needed).
