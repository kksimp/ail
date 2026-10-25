# 0513. Image decoders and a TrueType rasterizer written in AIL

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** Tooling, standard library (pixel layer)
- **Spec rules:** none yet (a tool module; rules come when it moves into `std.ui`)
- **Files:** `tools/pdf/image/image.ail`, `tools/pdf/image/font.ail`, [design note](https://github.com/kksimp/garlic-press/blob/main/docs/pdf-viewer.md#the-pixel-layer-adr-0513)

## Context

OCR (ADR 0512) needs a scanned page's pixels, so it needs the codecs scans
use: raw samples behind Flate, CCITT Group 3 and 4 fax, and baseline JPEG.
Its reference shapes need glyphs drawn from TrueType outlines. The PDF
viewer's pixel view, Garlic and the editor need the same pieces. AIL has
none of them, and ADR 0009 and ADR 0010 rule out libjpeg, libtiff and
FreeType.

## Options considered

### Option A — write them in AIL from the standards (chosen)
ITU-T T.4 and T.6 for fax (code tables as text, a 13-bit lookahead table,
changing elements per row, so work is per colour change); ITU-T T.81
baseline and extended Huffman JPEG (16-bit lookahead Huffman tables; the
inverse DCT of a whole plane as one 64-column matrix product); the OpenType
`glyf`, `loca`, `cmap` (formats 4 and 12) and `hmtx` tables with composite
glyphs; an anti-aliased rasterizer that adds each edge's signed area to the
pixels it crosses and sums along the row (non-zero fill).
- **Pros:** no dependency; shared by every pixel user; decoded output
  matches the reference decoders (fax and Flate exactly, JPEG within one
  grey level of libjpeg on the test scans).
- **Cons:** interpreter speed: a 1654 by 2339 JPEG page decodes in ~5.5 s,
  a Group 4 page in ~0.7 s; progressive JPEG, JBIG2, JPEG 2000 and CFF
  (OpenType PostScript) outlines are not covered yet.

### Option B — Rust built-ins in the bootstrap
- **Cons:** the bootstrap is to be retired (ADR 0023); code written now in
  Rust is written twice. Speed comes from the native backend instead.

### Option C — leave scans unsupported
- **Cons:** the designer's goal is searchable scans.

## Decision

Option A. Criterion 2 favours one readable implementation in the language
itself; criterion 4's cost falls to the native backend and to tensor
kernels (the JPEG inverse DCT already runs as a matrix product).

## Consequences

- The modules move beside `std.ui` as `std.ui.image` and `std.ui.raster`
  (agreed with the window-layer owner), with spec rule IDs, once OCR has
  settled their API. The first shared pixel type is `Gray` (0 to 255); an
  RGB bitmap joins it when a colour user needs one.
- Unsupported codecs report `ImageError.Unsupported`, and OCR skips those
  images rather than failing the page.

## Rejected alternatives

- Rust built-ins in the bootstrap (option B): rejected because the bootstrap is to be retired (ADR 0023), so code written now in Rust would be written twice. Speed comes from the native backend instead.
- Leaving scans unsupported (option C): rejected because the designer's goal is searchable scans.
