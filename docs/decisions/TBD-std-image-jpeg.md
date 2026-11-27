# TBD. `std.image.jpeg`: baseline and progressive JPEG decoding in AIL, matching libjpeg-turbo

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for a fuller std on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [std.jpeg.1-6](../spec/std-jpeg.md)
- **Relates to:** [ADR 0513](0513-image-decoders-and-font-rasterizer-in-ail.md) (a baseline decoder for PDF scans, now in the AGPL garlic-press repository); [ADR 1250](1250-std-image-png.md) (`std.image`)
- **Number:** the integrator assigns it; rename this file and the links when it does.

## Context

JPEG is the most common photo format; programs that make thumbnails, read camera files or test image pipelines need to
read it. ADR 0513's decoder lives in an AGPL application repository, is baseline only and was within one grey level of
libjpeg, not equal. `std.image` gives the shared RGBA8 buffer.

## Decision

- `std.image.jpeg` (`std/image/jpeg.ail`, module `jpeg`), written in AIL from ITU-T T.81: `decode`, `decode_upright`,
  `info` and `orient`. SOF0, SOF1 and SOF2 with 8-bit samples; gray, YCbCr, RGB, CMYK and YCCK; whole sampling factors;
  restart intervals; the EXIF orientation.
- **The output equals libjpeg-turbo's default decode byte for byte** (what Pillow, browsers built on it and most tools
  show): the `islow` integer IDCT, its fancy upsampling rules (including the cases where it falls back to repeating
  samples), its YCbCr tables and its colour-space rules. CMYK follows Pillow's conversion. Exact output makes the tests
  digests of a reference decode, and a program gets the same pixels as the tools it is compared with.
- Speed: Huffman codes up to 9 bits decode with one table lookup; a 64-bit bit buffer refills a byte at a time;
  baseline blocks go through the IDCT as soon as they are read, so only the sample planes (one byte a sample) are kept;
  progressive coefficients are kept as `List[u16]` (two bytes, biased by 32768). The IDCT skips columns and rows whose
  AC terms are zero, as libjpeg does, with the same result. Upsampling and colour conversion run a row at a time.
- `decode` does not rotate (as libjpeg and Pillow); `decode_upright` applies the EXIF orientation; `orient` is public
  so a program can apply an orientation it read with `info`.
- Failures are `JpegError { kind, why }` with `kind` `NotJpeg`, `Truncated`, `Corrupt` or `Unsupported`; decoding never
  traps. Values a corrupt stream could push out of range are clamped as libjpeg's 16-bit coefficients would hold them.
- Implicit (std.use.4) as `jpeg`. No runtime, checker or lowering work; no syntax changes.

## Rejected alternatives

- **The AAN (`ifast`) or a float IDCT for speed.** Faster in C with SIMD, but not exact with libjpeg-turbo's default,
  so tests could only check a tolerance and results would differ from other tools; `islow` with zero shortcuts is
  close in cost in AIL.
- **Moving ADR 0513's decoder in.** It is AGPL in an application repository, baseline only and not exact.
- **Keeping all coefficients for baseline images too.** A 12-megapixel 4:2:0 photo would hold 18 million coefficients
  (144 MB as `int`); decoding block by block keeps only the planes.
- **Applying the EXIF orientation in `decode`.** libjpeg and Pillow do not; silently turned pixels would surprise
  code that reads the orientation itself. `decode_upright` names the behaviour.
- **Arithmetic coding, 12-bit samples, lossless and hierarchical JPEG.** Rare in files met today; they fail with
  `Unsupported` and can be added later without changing the API.
- **Partial images from truncated files (as browsers show).** A program would get a picture with a grey bottom and no
  sign of it; it fails with `Truncated`. A missing end-of-image marker after whole scans is common and accepted.
- **An encoder in the same change.** Decoding is the common need; an encoder is its own lane.
