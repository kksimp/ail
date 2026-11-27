# TBD. `std.image.png`: APNG animation decode and encode

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for a fuller std on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [std.png.11-14](../spec/std-png.md)
- **Number:** the integrator assigns it; rename this file and the links when it does.
- **Depends on:** [1250](1250-std-image-png.md) (the PNG module); the std.webp animation ADR (PR #1579) names APNG as its follow-up and sets the frame model.

## Context

APNG (the Animated PNG extension: `acTL`, `fcTL`, `fdAT`) is the PNG form of animation that browsers, chat tools and
screenshot recorders produce. `png.decode` already reads an APNG file as its default image; programs that need the frames
had no way to. `std.gif` and `std.webp` give frames as `Animation { width, height, loops, frames }` with whole-canvas
`Frame { img, delay_ms }`, and PNG should match so one program can handle all three.

## Decision

- `png.decode_animation(data) Animation ! PngError` and `png.encode_animation(anim) bytes ! PngError`, with `png.Frame`,
  `png.Animation` of the same shape as `webp`'s (`loops` 0 repeats forever; a file with no `acTL` is one frame, loops 1,
  delay 0) and `png.is_animated(data) bool`. `decode` and `info` are unchanged and give the default image.
- Decoding follows the APNG specification: `acTL` before the image data, `fcTL` and `fdAT` with consecutive sequence numbers
  (one count over both chunk kinds), frame rectangles inside the canvas, dispose ops none, background and previous (previous
  on the first frame acts as background), blend ops source and over, a default image that is the first frame or is not
  part of the animation. Each frame's pixels go through the same decoder as a still image (all color types and depths,
  `PLTE`, `tRNS`, Adam7), so the file's color type and interlace apply to every frame.
- Every frame is returned as the whole canvas as it looks when shown. The canvas starts transparent black. The delay is
  `delay_num * 1000 / delay_den` ms (denominator 0 means 100).
- Blend over is non-premultiplied with exact integer arithmetic rounded to the nearest: alpha `(sa * 255 + da * (255 - sa) + 127) / 255`,
  color `(sc * sa * 255 + dc * da * (255 - sa) + t / 2) / t` with `t = sa * 255 + da * (255 - sa)`; a source alpha of 0 keeps the
  canvas pixel and 255 replaces it. This equals the exact-fraction form of the specification's equation rounded to nearest.
- `encode_animation` writes every frame as the whole canvas to a viewer: the first frame is also the default image, so a
  viewer without APNG shows it; each later frame is cropped to the smallest rectangle that differs from the frame before
  it (a 1 x 1 rectangle when nothing does) with blend op source and dispose op none, so the pixels are exact for any alpha.
  The file is 8-bit RGB when every pixel of every frame is opaque and RGBA otherwise; the delay is written as milliseconds
  over 1000. `decode_animation(encode_animation(a)) == a` for every valid animation.
- Failures are `PngError` as for a still image: `BadChunk` for a wrong frame count, sequence, size, rectangle or op, a first
  frame that does not cover the canvas, or a frame with no data; `Corrupt` for invalid zlib; never a trap. Work is bounded:
  frames times canvas pixels is at most `image.MAX_PIXELS`.
- Chunks after the first `IDAT` do not start an animation: an `acTL` there is ignored and the file is a still image.
- No syntax changes, no runtime or checker work; the still-image decoder and encoder were split into helpers with the same behavior.
- Checking: a Python mirror of the same code structure was compared with Pillow (libpng chunks): the frames of seven
  Pillow-written APNG files (sub-rectangles, dispose background and previous, blend over, RGB, palette, a hidden default
  image) match the frames Pillow was given; 30 random animations round-trip through the mirror's encoder and Pillow's reader;
  the blend equals an exact-fraction implementation of the specification on 100,000 random pixel pairs. Hand-assembled files
  cover translucent blends, Adam7 frames, a palette with `tRNS` and a gray image with a hidden default image. The goldens
  come from Pillow and that reference, never from the AIL. Pillow's own reader differs from the specification on translucent
  blends (it interpolates the alpha channel), on a hidden default image (it draws the next frames over it) and fails on
  interlaced APNG, so those fixtures use the exact-fraction reference. The first Mac gate is the first AIL compile.

## Rejected alternatives

- **A separate `std.image.apng` module.** The chunk reader, header checks, filter and Adam7 code are the PNG module's own
  private functions; a second module would copy them. `std.webp` and `std.gif` already hold animation in the format's module.
- **Frames as sub-rectangles with ops (the file's own model).** Every consumer would redo the compositing; whole-canvas
  frames are what `gif.decode` and `webp.decode_animation` give.
- **Premultiplied or Pillow-style blending.** The specification asks for non-premultiplied over; Pillow's mask paste does
  not blend the alpha channel; browsers round differently from each other, so the tests use an exact definition.
- **Encoder options (dispose and blend ops, palettes, rectangles) now.** Cropping to the changed rectangle with source
  blending is exact for every input and small for typical animations; options can come as defaulted parameters.
- **Dropping the default image when it is not part of the animation.** `decode` still returns it, as `libpng` does.
