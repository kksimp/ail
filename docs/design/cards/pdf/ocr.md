# pdf: ocr
tests: ocr.ocr.finds_lines_and_words, ocr.ocr.saves_and_loads_shapes

## ocr
```
pub fn ocr.gray_of(doc reader.Doc, obj reader.Obj) image.Gray ! reader.PdfError [fail]  ## The grey bitmap of an image XObject or inline image's stream.
pub struct ocr.Ink  ## A page image as ink: black runs per row (see `Run`).
    pub width int
    pub height int
    pub rows List[List[ocr.Run]]
pub fn ocr.ink_of(doc reader.Doc, obj reader.Obj) ocr.Ink ! reader.PdfError [fail]  ## The ink of an image: fax data straight from its changing elements,
pub fn ocr.ink_runs(g image.Gray) List[List[ocr.Run]]  ## Black runs of a grey image at its Otsu threshold; when most of the
pub struct ocr.Run  ## A black run on row `y`, from `x0` up to but not including `x1`.
    pub y int
    pub x0 int
    pub x1 int
pub struct ocr.Blob  ## A glyph's ink: its box (`x1`, `y1` exclusive) and runs.
    pub x0 int
    pub y0 int
    pub x1 int
    pub y1 int
    pub runs List[ocr.Run]
    pub fn ocr.Blob.w(self) int
    pub fn ocr.Blob.h(self) int
pub struct ocr.GlyphLine  ## A line of glyphs, left to right; `base` is the baseline row and `size`
    pub glyphs List[ocr.Blob]
    pub words List[int]
    pub base int
    pub size int
    pub x0 int
    pub y0 int
    pub x1 int
    pub y1 int
pub fn ocr.threshold(g image.Gray) int  ## The grey level that best splits ink from paper (Otsu's method).
pub fn ocr.runs_of(g image.Gray, t int) List[List[ocr.Run]]  ## Black runs row by row: pixels at or under `t` are ink.
pub fn ocr.blobs_of(rows List[List[ocr.Run]]) List[ocr.Blob]  ## Eight-connected components of the runs: runs on neighbouring rows join
pub fn ocr.lines_of(blobs List[ocr.Blob]) List[ocr.GlyphLine]  ## Groups blobs into lines and words. Blobs much taller than the text
pub struct ocr.Ocr
    cache ref ocr.ShapeCache
    pub on bool
    pub fn ocr.Ocr.new(on bool) ocr.Ocr
    pub fn ocr.Ocr.shapes(self) ocr.Shapes [mutates]
    pub fn ocr.Ocr.page(self, ps reader.Pages, i int) reader.Page  ## The page with OCR lines when it is a scan and OCR is on; the result is
pub struct ocr.Word  ## A recognized word in image pixels.
    pub text str
    pub x0 int
    pub y0 int
    pub x1 int
    pub y1 int
    pub sure float
    pub starts List[int]  ## Each character's left and right edge.
    pub ends List[int]
pub struct ocr.TextRow  ## A recognized line: its words, baseline and text size in pixels.
    pub words List[ocr.Word]
    pub base int
    pub size int
pub fn ocr.read_gray(g image.Gray, shapes ocr.Shapes) List[ocr.TextRow]  ## Recognizes the text of a grey image.
pub fn ocr.read_runs(runs List[List[ocr.Run]], shapes ocr.Shapes) List[ocr.TextRow]  ## Recognizes the text in rows of black runs.
pub fn ocr.in_context(labels List[str]) List[str]  ## Glyphs that look alike (l, I, 1, |, !; o, O, 0; S, 5; Z, 2) settled
pub fn ocr.recognize(doc reader.Doc, p reader.Page, shapes ocr.Shapes) reader.Page  ## Runs OCR over every image on the page and maps the words to page
pub fn ocr.dims() int
pub fn ocr.features_of(glyphs List[ocr.Blob], lines List[ocr.GlyphLine]) List[List[float]]  ## The features of many glyphs at once: their fine grids through the
pub fn ocr.features(b ocr.Blob, l ocr.GlyphLine) List[float]  ## The features of glyph `b` on line `l`.
pub struct ocr.Shapes  ## Reference shapes: a label and its feature row each.
    pub labels List[str]
    pub rows List[float]
    pub fn ocr.Shapes.load(text str) ocr.Shapes  ## Reads shapes saved by `Shapes.save`: one per line, the label's code
    pub fn ocr.Shapes.save(self) str
    pub fn ocr.Shapes.classify(self, feats List[List[float]]) List[ocr.Guess]  ## The nearest shape's label for each feature row, by Euclidean distance:
    pub fn ocr.Shapes.reduce(self, k int) ocr.Shapes  ## At most `k` shapes per label: the two nearest shapes of a label merge
pub struct ocr.Guess  ## A glyph's best label, how sure it is (0 to 1), and its distance.
    pub label str
    pub sure float
    pub dist float
pub ocr.SHAPES := ...
pub ocr.CHARS := ...  ## The characters OCR knows: printable ASCII and common Latin-1 and
pub fn ocr.samples(f image.Font, mut labels List[str], mut rows List[List[float]]) int [mutates]  ## Samples of one font: each character between `H` and `x`, so the line
pub fn ocr.means(labels List[str], rows List[List[float]]) ocr.Shapes  ## The mean shape per label of one font's samples.
```

trimmed to fit 2000 tokens (~1450); dropped: private items
