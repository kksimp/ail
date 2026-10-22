# garlic: image
tests: image.image.decodes_a_group_4_row, image.image.dims_reads_each_format_header, image.image.fax_tables_are_prefix_free, image.image.gif_decodes_the_first_frame, image.image.jpeg_decodes_baseline_progressive_subsampled_and_restarts, image.image.png_decodes_every_colour_type_depth_and_interlace, image.image.reads_samples, image.image.unknown_data_is_refused

## image
```
pub struct image.Rgba  ## Pixels row by row, `px[y * width + x]` as `0xAARRGGBB` (alpha 255 is opaque).
    pub width int
    pub height int
    pub px List[int]
    pub fn image.Rgba.new(width int, height int, color int) image.Rgba
    pub fn image.Rgba.bytes(self) List[int]  ## The image as bytes `R, G, B, A` per pixel, for a window's bitmap surface.
pub fn image.decode(data List[int]) image.Rgba ! image.ImageError [fail]  ## The image `data` (PNG, JPEG or GIF by its first bytes) as colour.
pub fn image.dims(data List[int]) List[int]  ## The `[width, height]` of a PNG, JPEG or GIF read from its header alone, or
pub struct image.Font  ## A parsed font: table offsets and the numbers every lookup needs.
    data List[int]
    tables Map[str, int]
    pub units int
    long_loca bool
    pub glyphs int
    metrics int
    cmap int
    cmap_format int
    pub ascent int
    pub descent int
    pub fn image.Font.parse(data List[int]) image.Font ! image.ImageError [fail]
    pub fn image.Font.glyph(self, ch char) int  ## The glyph index of a character, 0 (the missing glyph) when absent.
    pub fn image.Font.advance(self, gid int) int  ## A glyph's advance width in font units.
    pub fn image.Font.outline(self, gid int) List[List[float]]  ## A glyph's outline in font units, y up: quadratic pieces
pub struct image.Canvas  ## A coverage canvas: each pixel's ink from 0 to 1, y down.
    pub width int
    pub height int
    acc List[float]
    pub fn image.Canvas.new(width int, height int) image.Canvas
    pub fn image.Canvas.glyph(mut self, f image.Font, gid int, x float, y float, px float) [mutates]  ## Fills glyph `gid` with its origin at pixel (`x`, `y`) (the baseline),
    pub fn image.Canvas.text(mut self, f image.Font, text str, x float, y float, px float, track float) float [mutates]  ## Draws `text` from (`x`, `y`) on the baseline and returns the pen's x
    pub fn image.Canvas.gray(self) image.Gray  ## The ink so far as a grey bitmap: 0 where covered, 255 where bare.
pub fn image.gif(d List[int]) image.Rgba ! image.ImageError [fail]  ## Decodes the first frame of a GIF to colour.
pub struct image.Gray
    pub width int
    pub height int
    pub px List[int]
    pub fn image.Gray.new(width int, height int, value int) image.Gray
    pub fn image.Gray.at(self, x int, y int) int
    pub fn image.Gray.inverted(self) image.Gray  ## Inverts every pixel, for a `/Decode [1 0]` image.
    pub fn image.Gray.pgm(self) List[int]  ## A PGM (P5) file of the image, for looking at what a decoder produced.
pub error enum image.ImageError
    Bad(why str)
    Unsupported(what str)
pub impl ToStr for image.ImageError
    pub fn to_str(self) str
pub fn image.from_samples(data List[int], w int, h int, bpc int, comps int) image.Gray ! image.ImageError [fail]  ## The grey value of samples: `comps` 1 (grey), 3 (RGB) or 4 (CMYK), each
pub fn image.from_indexed(data List[int], w int, h int, bpc int, palette List[int], base int) image.Gray ! image.ImageError [fail]  ## Samples through a palette of `base` components each (8.6.6.3).
pub fn image.ccitt(data List[int], cols int, rows int, k int, align bool, black1 bool) image.Gray ! image.ImageError [fail]  ## Decodes CCITT fax data (8.4 CCITTFaxDecode): `k` < 0 is Group 4, 0 is
pub fn image.ccitt_rows(data List[int], cols int, rows int, k int, align bool) List[List[int]] ! image.ImageError [fail]  ## Each row's changing elements: where the colour flips, starting white,
pub fn image.jpeg(d List[int]) image.Gray ! image.ImageError [fail]  ## Decodes a baseline, extended sequential or progressive Huffman JPEG to
pub fn image.jpeg_rgb(d List[int]) image.Rgba ! image.ImageError [fail]  ## Decodes a JPEG to colour. Three components are YCbCr unless an Adobe
pub fn image.png(d List[int]) image.Rgba ! image.ImageError [fail]  ## Decodes a PNG to colour.
```

trimmed to fit 2000 tokens (~1363); dropped: private items
