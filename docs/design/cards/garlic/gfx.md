# garlic: gfx

## gfx
```
pub struct gfx.Hit  ## The screen rectangle of a link, in page pixels.
    pub x float
    pub y float
    pub w float
    pub h float
    pub href str
    pub node int  ## The form control's node for a click that uses it, else -1.
    pub anchor int  ## The link's node (the `a` element) for a link's rectangle, else -1.
pub struct gfx.Scene  ## A laid-out page: its drawing operations, links and size.
    pub ops List[draw.Op]
    pub hits List[gfx.Hit]
    pub height float
    pub bg int  ## The canvas color (the `body` or `html` background, else white).
    pub probes List[gfx.Probe]  ## The text runs whose widths were estimated, not measured.
    pub bits Map[str, images.Bitmap]  ## Pictures drawn by the layout itself (inline SVG), by the `src` of their `Pic`.
    pub fn gfx.Scene.hit_at(self, x float, y float) gfx.Hit?  ## The link or control under point (`x`, `y`) of the page, if any.
    pub fn gfx.Scene.link_at(self, x float, y float) str?  ## The link under point (`x`, `y`) of the page, if any.
pub struct gfx.Probe  ## A text run whose width the host can measure (docs/spec/std-app.md,
    pub text str
    pub size float
    pub bold bool
    pub mono bool
pub fn gfx.probe_key(text str, size float, bold bool, mono bool) str  ## The key of a run in the measured table.
pub fn gfx.text_width(text str, size float, bold bool, mono bool) float  ## The width in pixels of `text` at `size`, in a proportional or (`mono`)
pub fn gfx.length(v str, size float, rel float) float?  ## A length in pixels: `px`, `em`, `rem`, `pt`, `%` (of `rel`) or `0`.
pub fn gfx.color(v str) int?  ## A color: `#rgb`, `#rrggbb`, `rgb()`, `rgba()` or a name. `none` for
pub fn gfx.color_over(v str, under int) int?  ## A color as `color` reads it, with `rgba()` and `#rrggbbaa` mixed into the
pub fn gfx.layout(d html.Doc, url str, width float, sheet css.Sheet) gfx.Scene  ## Lays `d` out in a column `width` pixels wide with `sheet` (the linked
pub fn gfx.layout_with(d html.Doc, url str, width float, sheet css.Sheet, sizes Map[str, List[int]]) gfx.Scene  ## `layout` with the pictures' sizes (`images.prefetch`), so an `<img>`
pub fn gfx.layout_measured(d html.Doc, url str, width float, sheet css.Sheet, sizes Map[str, List[int]], meas Map[str, float], editing int) gfx.Scene  ## `layout_with` using the run widths in `meas` (`probe_key`), drawing text
```

trimmed to fit 2000 tokens (~820); dropped: private items
