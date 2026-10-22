# pdf
entry: main in pdf.ail
tests: 24 (`ail test` lists them)

## (root)
```
USAGE := ...
struct Args
fn parse_args(List[str]) Args  ## Words and `--flag [value]` options; `--case`, `--json` and `--print`
fn open_doc(str) reader.Doc ! reader.PdfError [fail fs io]
fn chosen(reader.Doc, Args) List[reader.Page] ! reader.PdfError [fail]  ## The pages `--page` picks: all of them, or one.
fn pick(reader.Doc, Args, ocr.Ocr) List[reader.Page] ! reader.PdfError [fail]
fn ocr_of(Args) ocr.Ocr
fn text_cmd(Args) !reader.PdfError [fail fs io]
fn grep_cmd(Args) bool ! reader.PdfError [fail fs io]
fn json_cmd(Args) !reader.PdfError [fail fs io]
fn info_json(reader.Doc) Json
fn info_cmd(Args) !reader.PdfError [fail fs io]
fn outline_cmd(Args) !reader.PdfError [fail fs io]
fn bench_cmd(Args) !reader.PdfError [clock fail fs io]  ## `bench`: milliseconds for each stage, to track the reader's speed.
fn images_cmd(Args) !reader.PdfError [clock fail fs io]  ## `images`: decodes each image painted on the chosen pages, with its
fn layout_cmd(Args) !reader.PdfError [clock fail fs io]  ## `layout`: the glyph lines OCR finds on scanned pages, as boxes.
fn train_cmd(Args) !reader.PdfError [fail fs io]  ## `train FONT.ttf...
fn main() [clock env fs io ui]
```

## image
```
pub struct image.Font  ## A parsed font: table offsets and the numbers every lookup needs.
pub struct image.Canvas  ## A coverage canvas: each pixel's ink from 0 to 1, y down.
pub struct image.Gray
pub error enum image.ImageError
pub impl ToStr for image.ImageError
pub fn image.from_samples(List[int], int, int, int, int) image.Gray ! image.ImageError [fail]  ## The grey value of samples: `comps` 1 (grey), 3 (RGB) or 4 (CMYK), each
pub fn image.from_indexed(List[int], int, int, int, List[int], int) image.Gray ! image.ImageError [fail]  ## Samples through a palette of `base` components each (8.6.6.3).
pub fn image.ccitt(List[int], int, int, int, bool, bool) image.Gray ! image.ImageError [fail]  ## Decodes CCITT fax data (8.4 CCITTFaxDecode): `k` < 0 is Group 4, 0 is
pub fn image.ccitt_rows(List[int], int, int, int, bool) List[List[int]] ! image.ImageError [fail]  ## Each row's changing elements: where the colour flips, starting white,
pub fn image.jpeg(List[int]) image.Gray ! image.ImageError [fail]  ## Decodes a baseline (or extended sequential, Huffman) JPEG to grey: the
```

## ocr
```
pub fn ocr.gray_of(reader.Doc, reader.Obj) image.Gray ! reader.PdfError [fail]  ## The grey bitmap of an image XObject or inline image's stream.
pub struct ocr.Ink  ## A page image as ink: black runs per row (see `Run`).
pub fn ocr.ink_of(reader.Doc, reader.Obj) ocr.Ink ! reader.PdfError [fail]  ## The ink of an image: fax data straight from its changing elements,
pub fn ocr.ink_runs(image.Gray) List[List[ocr.Run]]  ## Black runs of a grey image at its Otsu threshold; when most of the
pub struct ocr.Run  ## A black run on row `y`, from `x0` up to but not including `x1`.
pub struct ocr.Blob  ## A glyph's ink: its box (`x1`, `y1` exclusive) and runs.
pub struct ocr.GlyphLine  ## A line of glyphs, left to right; `base` is the baseline row and `size`
pub fn ocr.threshold(image.Gray) int  ## The grey level that best splits ink from paper (Otsu's method).
pub fn ocr.runs_of(image.Gray, int) List[List[ocr.Run]]  ## Black runs row by row: pixels at or under `t` are ink.
pub fn ocr.blobs_of(List[List[ocr.Run]]) List[ocr.Blob]  ## Eight-connected components of the runs: runs on neighbouring rows join
pub fn ocr.lines_of(List[ocr.Blob]) List[ocr.GlyphLine]  ## Groups blobs into lines and words.
pub struct ocr.Ocr
pub struct ocr.Word  ## A recognized word in image pixels.
pub struct ocr.TextRow  ## A recognized line: its words, baseline and text size in pixels.
pub fn ocr.read_gray(image.Gray, ocr.Shapes) List[ocr.TextRow]  ## Recognizes the text of a grey image.
pub fn ocr.read_runs(List[List[ocr.Run]], ocr.Shapes) List[ocr.TextRow]  ## Recognizes the text in rows of black runs.
pub fn ocr.in_context(List[str]) List[str]  ## Glyphs that look alike (l, I, 1, |, !; o, O, 0; S, 5; Z, 2) settled
pub fn ocr.recognize(reader.Doc, reader.Page, ocr.Shapes) reader.Page  ## Runs OCR over every image on the page and maps the words to page
pub fn ocr.dims() int
pub fn ocr.features_of(List[ocr.Blob], List[ocr.GlyphLine]) List[List[float]]  ## The features of many glyphs at once: their fine grids through the
pub fn ocr.features(ocr.Blob, ocr.GlyphLine) List[float]  ## The features of glyph `b` on line `l`.
pub struct ocr.Shapes  ## Reference shapes: a label and its feature row each.
pub struct ocr.Guess  ## A glyph's best label, how sure it is (0 to 1), and its distance.
pub ocr.SHAPES := ...
pub ocr.CHARS := ...  ## The characters OCR knows: printable ASCII and common Latin-1 and
pub fn ocr.samples(image.Font, mut List[str], mut List[List[float]]) int [mutates]  ## Samples of one font: each character between `H` and `x`, so the line
pub fn ocr.means(List[str], List[List[float]]) ocr.Shapes  ## The mean shape per label of one font's samples.
```

## reader
```
pub struct reader.Glyph  ## One shown glyph, in default user space (points, origin bottom left).
pub fn reader.mul(List[float], List[float]) List[float]  ## `a` then `b`: the product a x b of two [a b c d e f] matrices (8.3.4).
pub fn reader.plain(str) str  ## Expands the Latin ligatures U+FB00..FB06 and drops control characters, so
pub struct reader.Placed  ## An image painted on a page: its stream and the matrix that maps its
pub struct reader.PageText  ## The glyphs of a page, in content order, and how many images it paints.
pub struct reader.Line  ## A line of text: its characters and where each one starts and ends.
pub fn reader.lines_of(List[reader.Glyph]) List[reader.Line]  ## Groups glyphs into lines in content order: a new line where the
pub struct reader.XRef  ## Where an object lives: `kind` 1 is a file offset `a`; kind 2 is entry
pub struct reader.Doc
pub struct reader.Encoded  ## An image stream with every filter applied but a last image codec
pub fn reader.decode_filter(str, List[int], Map[str, reader.Obj]) List[int] ! reader.PdfError [fail]
pub struct reader.Font
pub struct reader.Code  ## One shown code: its text, its width in text space (font size 1), and
pub fn reader.glyph_text(str) str?  ## The Unicode text of a glyph name: the tables, `uniXXXX`, `uXXXX[XX]`,
pub fn reader.utf16(List[int]) str  ## Text from UTF-16BE units, pairing surrogates.
pub fn reader.inflate_zlib(List[int]) List[int] ! reader.PdfError [fail]  ## Inflates a zlib stream (RFC 1950): a 2-byte header, deflate data, and an
pub enum reader.Obj
pub struct reader.Lexer
pub fn reader.is_white(int) bool  ## Whitespace (ISO 32000-2 table 1).
pub fn reader.is_delim(int) bool  ## Delimiters (ISO 32000-2 table 2).
pub struct reader.Box
pub struct reader.TextLine
pub struct reader.Link
pub struct reader.PageModel
pub struct reader.OutlineItem
pub fn reader.r2(float) float  ## Rounds to 0.01 pt, which is finer than any glyph and keeps JSON short.
pub struct reader.Page  ## A page model plus the line geometry search needs.
pub fn reader.text_string(List[int]) str  ## A text string (7.9.2.2): UTF-16BE or UTF-8 with a byte order mark, else
pub fn reader.utf8(List[int]) str  ## UTF-8 bytes as text; a bad sequence gives U+FFFD.
pub struct reader.Pages  ## The pages of a document, each read the first time it is asked for
pub fn reader.pages_of(List[reader.Page]) reader.Pages  ## Pages already read, for tests and tools that build pages themselves.
pub fn reader.text_page(int, List[str]) reader.Page  ## A US Letter page of left-aligned 10 pt lines 14 pt apart from the top,
pub struct reader.Hit
pub fn reader.norm_pattern(str, bool) str  ## A pattern as search compares it.
pub fn reader.search_page(reader.Page, str, bool) List[reader.Hit]  ## Every hit of `pattern` on a page.
pub error enum reader.PdfError
pub impl ToStr for reader.PdfError
pub fn reader.read_file(str) List[int] ! reader.PdfError [fail fs io]  ## Reads a whole file as bytes (ADR 0412); the reader works on their
pub fn reader.ascii(str) List[int]  ## Bytes of an ASCII or Latin-1 string, one per character.
pub fn reader.latin1(List[int]) str  ## A byte string as text, one character per byte (Latin-1).
pub fn reader.chr(int) char  ## The character with code `n`; U+FFFD for a code that isn't one.
pub fn reader.bytes_at(List[int], int, List[int]) bool  ## Whether `pat` occurs in `b` at `at`.
pub fn reader.rfind(List[int], List[int], int) int?  ## The last start of `pat` in `b` at or before `before`, if any.
pub fn reader.find_from(List[int], List[int], int) int?  ## The first start of `pat` in `b` at or after `from`, if any.
```

## std.ui.widgets
```
pub struct std.ui.widgets.Rect  ## A rectangle of cells: its top-left corner and its size.
pub struct std.ui.widgets.Run  ## One run of text: where it starts, its colors, and whether it is bold.
pub enum std.ui.widgets.Action  ## What an event did to a widget (std.ui.21).
pub fn std.ui.widgets.color(Map[str, int], str) int  ## The color of `kind` in a theme (std.ui.20): the editor's theme kinds,
pub fn std.ui.widgets.draw_runs(mut ui.Window, List[std.ui.widgets.Run]) [io mutates ui]  ## Draws runs on a window.
pub fn std.ui.widgets.text_rows(List[std.ui.widgets.Run], int, int) List[str]  ## The text of `runs` on a grid of `cols` by `rows` cells, row by row: what
pub fn std.ui.widgets.fit(str, int) str  ## `text` cut or padded to exactly `n` characters.
pub fn std.ui.widgets.fuzzy(str, str) bool  ## Whether `item` holds the letters of `filter` in order, ignoring case
pub struct std.ui.widgets.TextField  ## A one-line text input with a label (std.ui.23).
pub struct std.ui.widgets.ListView  ## A list to choose from, narrowed by a fuzzy filter as the user types
pub struct std.ui.widgets.Button  ## A labelled button: Enter or Space when focused, or a click, presses it.
pub struct std.ui.widgets.Dialog  ## A message with choices; Left, Right and Tab move between them (std.ui.24).
pub struct std.ui.widgets.Pane  ## Scrolling text with a cursor line (std.ui.24).
pub struct std.ui.widgets.Node  ## A node of a tree, in depth-first order: its depth and whether it is open.
pub struct std.ui.widgets.Tree  ## An outline: nodes in depth-first order; a node's children follow it at a
```

## view
```
pub struct view.Layout  ## One laid-out page: text rows, and for each placed character which line
pub struct view.Viewer
pub fn view.with_ocr(str, reader.Pages, int, int, ocr.Ocr) view.Viewer
pub fn view.layout(reader.Page, int) view.Layout  ## Lays a page out for `cols` columns: one cell per typical character
pub fn view.natural_width(reader.Page) int  ## The columns a page needs for one cell per typical character.
pub fn view.start(str, reader.Pages, Map[str, str]) [io ui]  ## The viewer from the command line: `--page N`, `--find TEXT`, then
```

trimmed to fit 4000 tokens (~3802); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence)
