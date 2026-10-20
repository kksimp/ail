# PDF viewer: search the text layer, no OCR

> Non-normative design note, 2026-10-20. The decision is
> [ADR 0520](../decisions/0520-pdf-reader-on-the-text-layer.md); the tool is
> [`tools/pdf`](../../tools/pdf/README.md).

## The claim, stated plainly

The designer asked for a PDF viewer that searches PDFs without OCR, using
AIL's machine-first awareness of what is on screen. Most PDFs already carry
their text: a page's content stream shows strings of glyph codes in a font,
and the font says what Unicode each code is (a `/ToUnicode` CMap, or an
encoding whose glyph names map to Unicode, ISO 32000-2 sections 9.6-9.10).
A reader that parses the file, decompresses its streams and runs the text
operators gets every glyph's text and position exactly, with no OCR and no
pixels. That is what `tools/pdf` does.

Two limits are real and no window awareness changes them:

- **Scanned PDFs** are pictures of pages. They have no glyphs, so there is
  no text without OCR. The reader detects them (images painted, no text)
  and says so per page (`image_only`).
- **Fonts without a Unicode map** (some Type3 and symbolic fonts with no
  `/ToUnicode`) give glyph codes with no meaning. The reader counts them
  (`unreadable`) instead of guessing.

## Architecture

```
tools/pdf/
  pdf.ail            CLI: text, grep, json, info, outline, bench, view
  reader/            the reader module (no effects except read_file)
    source.ail       bytes (List[int]) and PdfError; read_file (stopgap)
    inflate.ail      RFC 1950/1951 inflate (stopgap)
    lex.ail          tokens and objects (7.2-7.3)
    doc.ail          xref tables and streams, object streams, repair, pages (7.5, 7.7)
    filters.ail      Flate, LZW, ASCII85, ASCIIHex, RunLength, PNG/TIFF predictors (7.4)
    fonts.ail        codes, ToUnicode CMaps, encodings, glyph names, widths (9.5-9.10)
    content.ail      text state machine to glyphs, glyphs to lines (8.4, 9.3, 9.4)
    model.ail        the page model, links, outline, text strings (7.9, 12.3, 12.5)
    search.ail       normalised search with boxes
  view/view.ail      headless viewer core plus the std.ui window
```

The viewer follows the editor's shape (ADR 0181): a `Viewer` value, keys
that change it, and a pure `render` to runs. The window, `--print` and the
tests draw the same runs.

## The PDF subset read today

| Area | Supported | Not yet |
|---|---|---|
| File structure | classic tables, cross-reference streams, hybrid files, `/Prev` chains, object streams, rebuild by scanning when damaged | linearization hints (ignored, harmless) |
| Filters | Flate and LZW with predictors, ASCII85, ASCIIHex, RunLength | image codecs (DCT, JPX, CCITT, JBIG2): pixels only |
| Security | none | encryption; most "encrypted" files use an empty user password (RC4 or AES with MD5/SHA-256), so this is the next slice |
| Fonts | Type1, TrueType, Type3, Type0 with Identity or embedded CMaps; ToUnicode bfchar/bfrange; Standard and WinAnsi encodings with Differences; glyph names incl. `uniXXXX` | MacRoman and MacExpert tables; predefined CJK CMaps (UniGB-UCS2-H and friends) |
| Text state | `Tc Tw Tz TL Ts Tf Td TD Tm T* Tj TJ ' "`, `q Q cm`, form XObjects, inline images skipped | vertical writing; rotated text is read but laid out as horizontal |
| Document | page tree inheritance, CropBox/MediaBox, link annotations (URI, GoTo, named destinations), outline, Info | tagged-PDF structure tree (reading order from tags), form fields, annotations' text |

Reading order is content order with lines grouped by baseline. That matches
`pdftotext -raw` and keeps label-value pairs together; tagged PDFs give a
better order and are the planned improvement.

## The agent-facing surface

Agents should never need a screenshot of a PDF. Every command has a
machine form:

- `pdf json FILE [--page N]`: per page `width`, `height`, `rotate`,
  `lines` (`text`, `x`, `y`, `size`, `right`), `links` (`rect`, `uri`,
  `page`), `images`, `unreadable`, `image_only`; plus `info` and `outline`.
  Coordinates are points from the crop box's lower left, rounded to 0.01.
- `pdf grep --json PATTERN FILE...`: one object per hit with `page`,
  `line`, `text`, `boxes` and `spans` (`[line, a, b]` character ranges).
  Search ignores case and runs of whitespace, crosses line breaks, and joins
  a word hyphenated at a line end. Exit status 1 when nothing matches.
- `pdf view FILE --print` and `--json`: the viewer's screen as text, and its
  state (page, scroll, query, hits, the rows on screen). This is the "window
  awareness": the viewer knows which document character is in every cell,
  so what it reports is exactly what a person sees, not a recognition of it.

## Measurements (2026-10-20, Linux x86-64, release bootstrap)

| Input | AIL reader | `pdftotext` (C++) |
|---|---|---|
| 2-page LibreOffice report, 29 KB | 125 ms for `pdf text`, process start included | 12 ms |
| 53-page LibreOffice document, 231 KB, 252,529 glyphs | 2.9 s (read 0.1 s, pages 2.8 s, search 0.2 s) | 0.08 s |

Text output matches `pdftotext` on the LibreOffice fixtures and on a
10-page ReportLab PDF, apart from line order on the ReportLab one (content
order versus `pdftotext`'s layout order). The cost is the interpreter: about
7 µs per glyph on the bytecode tier. The first speed pass (decode each
font's codes once, move the text matrix by its translation only) took the
53-page file from 4.3 s to 2.9 s. The native backend (lane 120) is where
the rest comes from; `pdf bench FILE` tracks it. The source is 3,249 lines,
33,570 tokens (tokbench mean).

## Dependencies and stopgaps

| Need | Today | Replaced by |
|---|---|---|
| Binary file reads | `read_file` runs POSIX `od` (ADR 0520) | `fs.read_bytes` (lane 123, ADRs 0412-0417) |
| Flate | `inflate_zlib` in AIL, behind one function | `std.compress.unzlib` once it returns `bytes` (lane 128 returns `str` today, which fails on binary streams) |
| Byte strings | `List[int]` | `bytes` (lane 123) |
| Regex search | plain substring | `std.regex` (lane 127, merged) as `pdf grep --regex` |

## What the window layer needs for pixels

The viewer draws the text layer on the text-cell grid. Drawing pages as
they look needs, in `std.ui` or a new `std.canvas`:

1. **A pixel surface**: `blit(x, y, w, h, rgba)` of a framebuffer region,
   or a canvas with `fill_rect`, `fill_path` (non-zero and even-odd) and
   `stroke_path`, with a transform and clip. The bootstrap's minifb window is
   already a framebuffer, so this is a thin addition.
2. **Glyph outlines**: parsing embedded TrueType (`glyf`) and CFF
   (Type 2 charstrings) fonts in AIL, then an anti-aliased scanline
   rasteriser. This also gives the editor real fonts.
3. **Images**: a baseline JPEG (DCT) decoder in AIL, plus Flate images with
   the predictors already written; JPX and JBIG2 later.
4. **Colour**: DeviceRGB/Gray/CMYK, ICC as sRGB approximations.
5. **HiDPI and scaling** (already listed in [gui-path.md](gui-path.md)).

None of this is needed for search or for agents. A pixel view would overlay
the same page model, so search hits and agent queries keep working on it.

## Rejected alternatives

- **OCR on rendered pages.** Slower, lossy, and pointless where the text is
  in the file. It stays the only way to read a scan, and is out of scope.
- **Linking a C library (Poppler, MuPDF, pdf.js).** ADR 0009 (custom
  tooling) and ADR 0010 (no external backend) rule it out; MuPDF's AGPL also
  conflicts with the licence (ADR 0510).
- **Waiting for `bytes` and `std.compress` before starting.** The stopgaps
  are two functions; the parser, fonts, search and viewer are written
  against `List[int]` and are unaffected by the swap.
- **Reading-order heuristics (column detection) first.** Content order is
  right for most generated PDFs; tagged structure is the better source and
  comes next.

## Next slices

1. Swap the stopgaps when lanes 123 and 128 land bytes.
2. Decryption with an empty user password (RC4, AES-128, AES-256).
3. `--regex` through `std.regex`; MacRoman; predefined CJK CMaps.
4. Tagged-PDF reading order; form fields and annotation text.
5. Pixel rendering once the window layer has a surface (above).
