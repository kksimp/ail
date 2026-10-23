# garlic: svg

## svg
```
struct svg.Shape  ## One shape's outline: closed or open subpaths as flat `x, y` lists.
    subs List[List[float]]
    closed List[bool]
struct svg.Paint  ## The paint in force: fill and stroke as `0xRRGGBB` or -1 for none, the
    fill int
    stroke int
    width float
    alpha float
    ink int
pub fn svg.render(d html.Doc, id int, w int, h int, ink int) images.Bitmap?  ## The pixels of the `<svg>` element `id`, `w` by `h`, with `ink` standing
fn svg.numbers(s str) List[float]  ## The numbers in `s`, split at anything that is not part of one.
fn svg.attr_num(d html.Doc, id int, name str) float
fn svg.paint_colour(v str, p svg.Paint) int  ## A colour word or `#rgb`/`#rrggbb`, else -1 (none), -2 (currentColor).
fn svg.parse_hex(h str) int
fn svg.paint_of(d html.Doc, id int, p svg.Paint) svg.Paint  ## The paint `id` sets on top of `p`: attributes, then its `style`.
fn svg.paint_node(d html.Doc, id int, parent svg.Paint, sc float, ox float, oy float, w int, h int, mut canvas List[float]) bool [mutates]  ## Draws element `id` and what it holds; whether anything was drawn.
fn svg.shape_of(d html.Doc, id int, tag str) svg.Shape
fn svg.ellipse(cx float, cy float, rx float, ry float) List[float]
fn svg.path_subs(dd str, mut subs List[List[float]], mut closed List[bool]) [mutates]  ## Splits path data into subpaths of line segments (curves are flattened).
fn svg.arc(x1 float, y1 float, rx0 float, ry0 float, rot float, large bool, sweep bool, x2 float, y2 float, mut out List[float]) [mutates]  ## An SVG elliptical arc from (x1, y1) to (x2, y2) as line segments.
fn svg.atan2(y float, x float) float
fn svg.fill_shape(shape svg.Shape, colour int, alpha float, sc float, ox float, oy float, w int, h int, mut canvas List[float]) [mutates]  ## Fills the shape (non-zero winding) onto `canvas`.
fn svg.stroke_shape(shape svg.Shape, colour int, alpha float, width float, sc float, ox float, oy float, w int, h int, mut canvas List[float]) [mutates]  ## Strokes each segment of the shape as a quad `width` wide (user units),
fn svg.quad(x1 float, y1 float, x2 float, y2 float, x3 float, y3 float, x4 float, y4 float, mut e List[float]) [mutates]
fn svg.cover_edges(edges List[float], colour int, alpha float, w int, h int, mut canvas List[float]) [mutates]  ## Scan-converts `edges` (`x1, y1, x2, y2` each, in pixels) with four
fn svg.span(x0 float, x1 float, w int, mut cov List[float]) [mutates]  ## Adds the coverage of the span [x0, x1) to the pixels of one sub-scanline.
```
