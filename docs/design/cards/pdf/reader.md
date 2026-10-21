# pdf: reader
tests: 16 (`ail test` lists them)

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

trimmed to fit 2000 tokens (~1268); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence)
