# pdf: (root)
entry: main in pdf.ail

## (root)
```
USAGE := "usage: pdf COMMAND ...\n  text FILE [--page N]                  the text, pages split by form feeds\n  grep [--case] [--json] [--first] PATTERN FILE...  FILE:PAGE: line, for each line with a hit\n  json FILE [--page N]                  the page model: lines, boxes, links, outline\n  info FILE                             pages, metadata, and what could not be read\n  outline FILE                          the bookmarks, with their pages\n  bench FILE                            the time each stage of reading takes\n  images FILE [--page N] [--out F.pgm]  the page's images, decoded; --out saves the largest\n  view FILE [--page N] [--find TEXT] [--print] [--cols N]\n                                        the text layer in a window; --print shows it here"
struct Args
    words List[str]
    flags Map[str, str]
    fn Args.word(self, i int) str?
fn parse_args(raw List[str]) Args  ## Words and `--flag [value]` options; `--case`, `--json` and `--print`
fn open_doc(path str) reader.Doc ! reader.PdfError [fail fs io]
fn chosen(doc reader.Doc, a Args) List[reader.Page] ! reader.PdfError [fail]  ## The pages `--page` picks: all of them, or one.
fn pick(doc reader.Doc, a Args, o ocr.Ocr) List[reader.Page] ! reader.PdfError [fail]
fn ocr_of(a Args) ocr.Ocr
fn text_cmd(a Args) !reader.PdfError [fail fs io]
    fails with reader.PdfError
fn grep_cmd(a Args) bool ! reader.PdfError [fail fs io]
fn json_cmd(a Args) !reader.PdfError [fail fs io]
    fails with reader.PdfError
fn info_json(doc reader.Doc) Json
fn info_cmd(a Args) !reader.PdfError [fail fs io]
    fails with reader.PdfError
fn outline_cmd(a Args) !reader.PdfError [fail fs io]
    fails with reader.PdfError
fn bench_cmd(a Args) !reader.PdfError [clock fail fs io]  ## `bench`: milliseconds for each stage, to track the reader's speed.
    fails with reader.PdfError
fn images_cmd(a Args) !reader.PdfError [clock fail fs io]  ## `images`: decodes each image painted on the chosen pages, with its
    fails with reader.PdfError
fn layout_cmd(a Args) !reader.PdfError [clock fail fs io]  ## `layout`: the glyph lines OCR finds on scanned pages, as boxes.
    fails with reader.PdfError
fn train_cmd(a Args) !reader.PdfError [fail fs io]  ## `train FONT.ttf... --out FILE`: draws each font's characters and saves
    fails with reader.PdfError
fn main() [clock env fs io ui]
```
