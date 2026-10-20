# 0520. A PDF reader and viewer in AIL that searches the text layer

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** Tooling
- **Spec rules:** none (a tool, not language behaviour)
- **Files:** `tools/pdf/`, `bootstrap/tests/tools.rs`, [design note](../design/pdf-viewer.md)

## Context

The designer asked (2026-10-20) for a PDF viewer that can search PDFs
without OCR, using AIL's machine-first awareness of what is on screen. Most
PDFs carry their text as glyph codes plus a map from codes to Unicode, so
OCR is unnecessary wherever that map exists; scans are the exception. AIL
had no binary file reads (lane 123 adds `bytes`) and `std.compress` returns
`str` (lane 128), which fails on binary streams. The window draws text cells
only.

## Options considered

### Option A — a reader of the text layer in AIL, with a text-cell viewer (chosen)
Parse the file, decompress streams, run the text operators, and map glyph
codes to Unicode through the fonts (ISO 32000-2, written from the
standard). Expose a page model (lines with positions, links, outline,
images, unreadable glyphs, image-only pages) as JSON, search with boxes, and
a viewer whose headless core renders runs, as the editor does (ADR 0181).
Bridge the two missing pieces with stopgaps behind one function each:
`read_file` through POSIX `od`, and `inflate_zlib` in AIL.
- **Pros:** exact text and positions, no recognition errors; agents get the
  same model the viewer draws (`--print`, `--json`), never a screenshot;
  works today; the stopgaps swap out in one line each.
- **Cons:** about 40 times slower than `pdftotext` on the interpreter
  (2.9 s for 53 pages); a second inflate exists until `bytes` lands.

### Option B — OCR on rendered pages
- **Cons:** needs a renderer and a recognition model; lossy; slow; solves
  only scans, which Option A detects and reports.

### Option C — bind Poppler, MuPDF or pdf.js
- **Cons:** violates ADR 0009 (custom tooling) and ADR 0010 (no external
  backend); MuPDF is AGPL, incompatible with ADR 0510.

### Option D — wait for `bytes` and `std.compress`
- **Cons:** blocks the parser, fonts, search and viewer, which don't depend
  on how bytes are read.

## Decision

Option A. Decided by criterion 2 (AI understanding): an agent queries
exact text, positions and boxes, and the viewer reports the document
character in each cell, so nothing is inferred from pixels. Criterion 1
(tokens) agrees: a page's JSON is far smaller than an image of it. Runtime
speed (criterion 4) is the cost, and it falls to the native backend.

## Consequences

- `tools/pdf` is a package: `pdf.ail` (CLI), `reader/` (no effects but
  `read_file`), `view/` (`std.ui`).
- `PdfError.Encrypted` is reported until decryption lands (next slice).
- When lane 123 lands, `read_file` becomes `fs.read_bytes` and `List[int]`
  becomes `bytes`; when `std.compress.unzlib` returns `bytes`, it replaces
  `inflate_zlib`, which stays as its differential test.
- A pixel view needs a surface, glyph rasterisation and image decoders in
  the window layer ([design note](../design/pdf-viewer.md#what-the-window-layer-needs-for-pixels)).
