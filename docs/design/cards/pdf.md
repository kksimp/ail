# pdf
entry: main in pdf.ail
tests: 29 (`ail test` lists them)

## (root)
```
USAGE := ...
struct Args
fn parse_args(List[str]) Args
fn open_doc(str) reader.Doc ! reader.PdfError [fail fs io]
fn chosen(reader.Doc, Args) List[reader.Page] ! reader.PdfError [fail]
fn pick(reader.Doc, Args, ocr.Ocr) List[reader.Page] ! reader.PdfError [fail]
fn ocr_of(Args) ocr.Ocr
fn text_cmd(Args) !reader.PdfError [fail fs io]
fn grep_cmd(Args) bool ! reader.PdfError [fail fs io]
fn json_cmd(Args) !reader.PdfError [fail fs io]
fn info_json(reader.Doc) Json
fn info_cmd(Args) !reader.PdfError [fail fs io]
fn outline_cmd(Args) !reader.PdfError [fail fs io]
fn bench_cmd(Args) !reader.PdfError [clock fail fs io]
fn images_cmd(Args) !reader.PdfError [clock fail fs io]
fn layout_cmd(Args) !reader.PdfError [clock fail fs io]
fn train_cmd(Args) !reader.PdfError [fail fs io]
fn main() [clock env fs io ui]
```

## image
```
pub struct image.Rgba
pub fn image.decode(List[int]) image.Rgba ! image.ImageError [fail]
pub fn image.dims(List[int]) List[int]
pub struct image.Font
pub struct image.Canvas
pub fn image.gif(List[int]) image.Rgba ! image.ImageError [fail]
pub struct image.Gray
pub error enum image.ImageError
pub impl ToStr for image.ImageError
pub fn image.from_samples(List[int], int, int, int, int) image.Gray ! image.ImageError [fail]
pub fn image.from_indexed(List[int], int, int, int, List[int], int) image.Gray ! image.ImageError [fail]
pub fn image.ccitt(List[int], int, int, int, bool, bool) image.Gray ! image.ImageError [fail]
pub fn image.ccitt_rows(List[int], int, int, int, bool) List[List[int]] ! image.ImageError [fail]
pub fn image.jpeg(List[int]) image.Gray ! image.ImageError [fail]
pub fn image.jpeg_rgb(List[int]) image.Rgba ! image.ImageError [fail]
pub fn image.png(List[int]) image.Rgba ! image.ImageError [fail]
```

## ocr
```
pub fn ocr.gray_of(reader.Doc, reader.Obj) image.Gray ! reader.PdfError [fail]
pub struct ocr.Ink
pub fn ocr.ink_of(reader.Doc, reader.Obj) ocr.Ink ! reader.PdfError [fail]
pub fn ocr.ink_runs(image.Gray) List[List[ocr.Run]]
pub struct ocr.Run
pub struct ocr.Blob
pub struct ocr.GlyphLine
pub fn ocr.threshold(image.Gray) int
pub fn ocr.runs_of(image.Gray, int) List[List[ocr.Run]]
pub fn ocr.blobs_of(List[List[ocr.Run]]) List[ocr.Blob]
pub fn ocr.lines_of(List[ocr.Blob]) List[ocr.GlyphLine]
pub struct ocr.Ocr
pub struct ocr.Word
pub struct ocr.TextRow
pub fn ocr.read_gray(image.Gray, ocr.Shapes) List[ocr.TextRow]
pub fn ocr.read_runs(List[List[ocr.Run]], ocr.Shapes) List[ocr.TextRow]
pub fn ocr.in_context(List[str]) List[str]
pub fn ocr.recognize(reader.Doc, reader.Page, ocr.Shapes) reader.Page
pub fn ocr.dims() int
pub fn ocr.features_of(List[ocr.Blob], List[ocr.GlyphLine]) List[List[float]]
pub fn ocr.features(ocr.Blob, ocr.GlyphLine) List[float]
pub struct ocr.Shapes
pub struct ocr.Guess
pub ocr.SHAPES := ...
pub ocr.CHARS := ...
pub fn ocr.samples(image.Font, mut List[str], mut List[List[float]]) int [mutates]
pub fn ocr.means(List[str], List[List[float]]) ocr.Shapes
```

## reader
```
pub struct reader.Glyph
pub fn reader.mul(List[float], List[float]) List[float]
pub fn reader.plain(str) str
pub struct reader.Placed
pub struct reader.PageText
pub struct reader.Line
pub fn reader.lines_of(List[reader.Glyph]) List[reader.Line]
pub struct reader.XRef
pub struct reader.Doc
pub struct reader.Encoded
pub fn reader.decode_filter(str, List[int], Map[str, reader.Obj]) List[int] ! reader.PdfError [fail]
pub struct reader.Font
pub struct reader.Code
pub fn reader.glyph_text(str) str?
pub fn reader.utf16(List[int]) str
pub fn reader.inflate_zlib(List[int]) List[int] ! reader.PdfError [fail]
pub enum reader.Obj
pub struct reader.Lexer
pub fn reader.is_white(int) bool
pub fn reader.is_delim(int) bool
pub struct reader.Box
pub struct reader.TextLine
pub struct reader.Link
pub struct reader.PageModel
pub struct reader.OutlineItem
pub fn reader.r2(float) float
pub struct reader.Page
pub fn reader.text_string(List[int]) str
pub fn reader.utf8(List[int]) str
pub struct reader.Pages
pub fn reader.pages_of(List[reader.Page]) reader.Pages
pub fn reader.text_page(int, List[str]) reader.Page
pub struct reader.Hit
pub fn reader.norm_pattern(str, bool) str
pub fn reader.search_page(reader.Page, str, bool) List[reader.Hit]
pub error enum reader.PdfError
pub impl ToStr for reader.PdfError
pub fn reader.read_file(str) List[int] ! reader.PdfError [fail fs io]
pub fn reader.ascii(str) List[int]
pub fn reader.latin1(List[int]) str
pub fn reader.chr(int) char
pub fn reader.bytes_at(List[int], int, List[int]) bool
pub fn reader.rfind(List[int], List[int], int) int?
pub fn reader.find_from(List[int], List[int], int) int?
```

## std.ui.widgets
```
pub struct std.ui.widgets.Rect
pub struct std.ui.widgets.Run
pub enum std.ui.widgets.Action
pub fn std.ui.widgets.color(Map[str, int], str) int
pub fn std.ui.widgets.draw_runs(mut ui.Window, List[std.ui.widgets.Run]) [io mutates ui]
pub fn std.ui.widgets.text_rows(List[std.ui.widgets.Run], int, int) List[str]
pub fn std.ui.widgets.fit(str, int) str
pub fn std.ui.widgets.fuzzy(str, str) bool
pub struct std.ui.widgets.TextField
pub struct std.ui.widgets.ListView
pub struct std.ui.widgets.Button
pub struct std.ui.widgets.Dialog
pub struct std.ui.widgets.Pane
pub struct std.ui.widgets.Node
pub struct std.ui.widgets.Tree
```

## view
```
pub struct view.Layout
pub struct view.Viewer
pub fn view.with_ocr(str, reader.Pages, int, int, ocr.Ocr) view.Viewer
pub fn view.layout(reader.Page, int) view.Layout
pub fn view.natural_width(reader.Page) int
pub fn view.start(str, reader.Pages, Map[str, str]) [io ui]
```

trimmed to fit 4000 tokens (~2157); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence), fields, variants and test names
