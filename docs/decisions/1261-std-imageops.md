# 1261. `std.image.ops`: resize, crop, rotate, flip, grayscale, compositing and drawing

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for a fuller std on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [std.imageops.1-10](../spec/std-imageops.md)
- **Number:** the integrator assigns it; rename this file and the links when it does.

## Context

`std.image` holds an RGBA8 `Image` and `std.image.png` reads and writes it, but a program cannot yet make a thumbnail,
crop a region, rotate a photo, convert to gray or put one image on another without writing the loops itself.

## Decision

- One module, `std/image/ops.ail`, named `imageops` (a bare `ops` would collide with user names, std.use.4). It is pure
  AIL: no runtime, checker or lowering work.
- Every function takes an `Image` and returns a new one; images are values (`pixels` is immutable `bytes`). Errors are
  `image.ImageError`, so one `else err` handles a decode and an edit alike.
- `resize` has three filters in an enum: `Nearest`, `Bilinear`, `Lanczos3`. Weighted filters are separable (a horizontal pass
  to a float buffer, then a vertical pass), widen the kernel by the scale when shrinking so no source pixel is skipped,
  and work on colors multiplied by alpha so transparent pixels leave no fringe. Output is rounded to the nearest byte.
- Rows are computed with `[f(y) par for y in ...]`: the resize passes, grayscale, rotations, flips, `composite` and
  `fill_rect` use all cores and give the sequential result. Lines are inherently sequential and run on one core.
- Compositing is Porter-Duff source-over on straight alpha, in integers, so the result is identical on every host.
- Drawing is rectangles and Bresenham lines with blending; `rgba` is a `0xRRGGBBAA` int, the same as `image.solid`.

## Rejected alternatives

- **In-place (`mut`) operations.** A copy per call costs one memory pass; value semantics keep the `Image` shareable and
  `==` meaningful. A batch drawing API can follow when a program needs thousands of shapes (each `draw_line` copies the
  image once).
- **Mitchell, Catmull-Rom, box, and an area-average filter now.** Three filters cover fast, smooth and sharp; others add
  names without changing what a program can do. The enum can grow.
- **Floating-point or premultiplied pixel storage.** It would change `Image`, which the codecs share.
- **Doing the weighted resize in the runtime.** AIL with `par` is fast enough to start; a native kernel can replace the
  loops behind the same API once the Mac gate measures them.
- **Resize in `std.image` itself.** The module is the type the codecs depend on; a codec should not pull in the filters.
- **A `Canvas` type with drawing methods.** More surface for the same operations; free functions on `Image` read the
  same as the PNG functions.
