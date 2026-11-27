# 1250. `std.image` and `std.image.png`: a shared RGBA8 image and PNG decode and encode

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for a fuller std on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [std.image.1-4](../spec/std-image.md), [std.png.1-10](../spec/std-png.md)
- **Number:** the integrator assigns it; rename this file and the links when it does.

## Context

Programs that touch pictures (thumbnails, screenshots, game sprites, test fixtures) have no way to read or write the
most common lossless format. `std.screen.pixels` and `std.gfx` draw but hold no image type; the apps (a browser, a
viewer, an editor) each carry their own PNG code. The std plan (`std-gap-plan.md`, wave 3) lists PNG, and the JPEG lane
needs the same buffer, so the buffer is its own module and the codecs are submodules (the shape of `std.audio` and
`std.audio.opus`).

## Decision

- `std.image` (`std/image/image.ail`) holds `Image { width, height, pixels }`: RGBA8, 4 bytes a pixel, rows from the top,
  no padding, `pixels` as immutable `bytes`. `image(w, h, pixels)` validates, `solid(w, h, 0xRRGGBBAA)` fills, `img.pixel(x, y)`
  reads one as `0xRRGGBBAA`. `MAX_PIXELS` is 2^27 so a hostile header cannot ask for gigabytes.
- `std.image.png` (`std/image/png.ail`, module `png`) is written in AIL. `decode` reads every color type and bit depth, `PLTE`,
  `tRNS`, the five filters and Adam7, and checks every CRC-32 and the zlib Adler-32. It uses the built-in `compress.unzlib`
  and `hash.crc32`; it adds no runtime or checker work.
- 16-bit samples keep their high byte and fewer-bit samples scale to 0..255: the pixels a browser shows. Gamma and color
  profiles are not applied.
- `encode` writes 8-bit RGB when every pixel is opaque and RGBA otherwise, picks each row's filter by the smallest
  sum of absolute (signed) values, and deflates with `compress.zlib`. `decode(encode(img)) == img` for every image.
- Failures are `PngError { kind, why }` with an enum `kind` (type.enum.7) and never traps.
- Both modules are implicit (std.use.4), under the names `image` and `png`.
- No syntax changes.

## Rejected alternatives

- **The image type inside `std.png`.** The JPEG decoder and future codecs would depend on a PNG module for a struct.
- **A `Pixel` struct or a `List[int]` of colors.** A 4096x4096 image would be 16 million elements; `bytes` is compact,
  immutable (so `==` and sharing are cheap) and what `std.screen.pixels` and the codecs move.
- **Keeping 16-bit and palette images in their own formats.** Every consumer would handle five layouts; RGBA8 is what
  drawing, resizing and encoding need. A `decode16` can be added when a program asks.
- **Runtime-built PNG (native code, like `compress`).** AIL is fast enough for filtering and unpacking here, and a
  module in AIL needs no checker, effects or lowering work.
- **Encoder options (filters, palettes, interlace) now.** Adaptive filtering with zlib gives a good size at no API cost;
  options can be added as defaulted parameters later.
- **Failing on a bad ancillary chunk.** Ancillary chunks carry no pixels; a decoder that stops on a bad `tEXt` refuses
  usable files. A bad CRC on any chunk is still `Checksum`, as libpng does for critical ones and as a strict reader should.
