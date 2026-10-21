# 0512. OCR of scanned PDF pages by nearest reference shape, on demand

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** Designer (goal: scanned PDFs searchable by recognising text pixels, 2026-10-20; card "Speed and lazy first, then OCR"; card "DejaVu plus OFL fonts" for the reference shapes), details AI (delegated, ADR 0076)
- **Topic:** Tooling
- **Spec rules:** none (a tool, not language behaviour)
- **Files:** `tools/pdf/ocr/`, `tools/pdf/pdf.ail`, `tools/pdf/view/view.ail`, `tools/pdf/reader/model.ail`, [design note](../design/pdf-viewer.md#ocr-of-scanned-pages-adr-0512)

## Context

ADR 0520 reads the text layer and reports image-only pages as needing OCR.
The designer set the goal on 2026-10-20: AIL should recognise the shape of
text pixels so scanned PDFs can be searched, with speed first (ADR 0511:
only pages shown or searched are read). AIL has no OCR library, and ADR
0009 and ADR 0010 rule out binding Tesseract or similar.

## Options considered

### Option A — nearest reference shape over connected components (chosen)
Threshold the page (Otsu), take black runs per row, join them into
eight-connected components, group components into lines by vertical
overlap and into words by gap size, and attach marks (dots, accents,
cedillas) to the glyph they sit over or under. Each glyph becomes 212
features: ink on an 8 by 8 grid over its box, its outline's left, right,
top and bottom edges on four 6 by 6 grids (stroke direction varies less
between fonts than exact ink), all soft-binned so a one-pixel shift moves
them a little, plus width, height, top and bottom against its line's
baseline and text size. Words then settle look-alikes by context (l, I,
1; o, 0; S, 5). The nearest of a set of reference shapes names a glyph;
the margin to the nearest other label is its confidence. Reference shapes are drawn from fonts by an AIL TrueType
rasterizer (ADR 0513) and cut out by the same layout code, so they are
measured exactly as scanned glyphs are. One matrix product scores a page's
glyphs against every shape.
- **Pros:** all AIL, no model weights beyond 600 KB of shape text
  (1,399 shapes); deterministic; explainable (an agent can see which shape won
  and by how much); fast enough per page on the interpreter; the same
  lines feed search, JSON and the viewer unchanged.
- **Cons:** touching or broken glyphs are misread; fonts unlike every
  reference font (script, blackletter) read poorly; no language model yet.

### Option B — a small neural network (MLP or CNN) trained on rendered glyphs
- **Pros:** better on degraded scans and unusual fonts.
- **Cons:** needs a training loop and weights; less explainable; slower on
  the interpreter until tensor kernels cover convolutions. A later step can
  replace the classifier behind the same features and data.

### Option C — whole-line recognition (LSTM/CTC, as Tesseract 4)
- **Cons:** large models, training infrastructure, far slower here; the
  best accuracy, but out of proportion for searching scans.

### Option D — bind Tesseract
- **Cons:** violates ADR 0009 and ADR 0010; Apache-2.0, but a C++ library.

Measured on 2026-10-20 (one page of business prose, 186 words, made with
LibreOffice and scanned by rendering, thresholding and compressing;
FreeSerif and FreeSans are not among the reference fonts):

| Scan | Character error rate | Words found |
|---|---|---|
| 300 dpi Group 4, Liberation Serif | 0.7% | 182/186 |
| 300 dpi Group 4, FreeSerif (unseen) | 0.4% | 183/186 |
| 300 dpi Group 4, FreeSans (unseen) | 3.3% | 169/186 |
| 300 dpi Group 4, Carlito | 0.8% | 180/186 |
| 200 dpi JPEG, FreeSerif (unseen) | 2.4% | 172/186 |
| 200 dpi JPEG, FreeSans (unseen) | 3.9% | 166/186 |

The 12 by 12 ink grid alone gave 9.8% and 12.3% on the two unseen fonts;
adding edge grids took them to 0.4% and 3.3%.

## Decision

Option A. Criterion 2 (AI understanding) favours it: an agent gets text
with positions and a confidence per page, and a wrong letter can be traced
to a shape. Criterion 4 (speed) holds through the on-demand rule: only a
scanned page that is shown or searched is recognised, once.

Reference shapes come from DejaVu (Bitstream Vera licence) and Liberation,
Carlito and Caladea (SIL Open Font License 1.1), chosen by the designer:
the OFL fonts are metric twins of Times, Arial, Calibri and Cambria, which
most scanned documents use. Only derived shape data is committed, never a
font file; `NOTICE` lists the fonts and their licences.

## Consequences

- `PageModel` gains `ocr` (lines came from OCR) and `confidence` (mean per
  word, 0 to 1). `--no-ocr` keeps the old behaviour.
- `pdf train FONT.ttf... --out tools/pdf/ocr/shapes.ail` rebuilds the
  shapes; `pdf layout` and `pdf images` show the intermediate steps.
- Lane 154 (tensor): an `argmin`/`argmax` along an axis (and a top-two)
  would remove the classifier's last per-element loop.
- Next: page-adaptive shapes (re-learn confident glyphs on the page),
  word-level correction against the document's own vocabulary, skew
  correction, and touching-glyph splitting.

## Rejected alternatives

- A small neural network trained on rendered glyphs (option B): not chosen now. It needs a training loop and weights, is less explainable, and is slower on the interpreter until tensor kernels cover convolutions. A later step can replace the classifier behind the same features.
- Whole-line recognition with LSTM/CTC, as Tesseract 4 (option C): rejected because it needs large models and training infrastructure, and is far slower here. It has the best accuracy, but out of proportion for searching scans.
- Binding Tesseract (option D): rejected because it violates ADR 0009 and ADR 0010. It is Apache-2.0, but a C++ library.
