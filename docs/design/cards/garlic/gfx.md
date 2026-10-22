# garlic: gfx

## gfx
```
pub struct gfx.Hit  ## The screen rectangle of a link, in page pixels.
    pub x float
    pub y float
    pub w float
    pub h float
    pub href str
pub struct gfx.Scene  ## A laid-out page: its drawing operations, links and size.
    pub ops List[draw.Op]
    pub hits List[gfx.Hit]
    pub height float
    pub bg int  ## The canvas color (the `body` or `html` background, else white).
    pub fn gfx.Scene.link_at(self, x float, y float) str?  ## The link under point (`x`, `y`) of the page, if any.
gfx.WIDTHS := ...  ## Advance widths of ASCII 32 to 126 in a 1000-unit em.
pub fn gfx.text_width(text str, size float, bold bool, mono bool) float  ## The width in pixels of `text` at `size`, in a proportional or (`mono`)
fn gfx.fmax(a float, b float) float
fn gfx.fmin(a float, b float) float
pub fn gfx.length(v str, size float, rel float) float?  ## A length in pixels: `px`, `em`, `rem`, `pt`, `%` (of `rel`) or `0`.
gfx.NAMED := ...
fn gfx.hex_digit(c char) int?
pub fn gfx.color(v str) int?  ## A color: `#rgb`, `#rrggbb`, `rgb()`, `rgba()` or a name. `none` for
fn gfx.tokens(v str) List[str]  ## Splits `v` at spaces that are not inside parentheses.
fn gfx.sides(v str, size float, rel float) (float, float, float, float, bool, bool)  ## One to four lengths as (top, right, bottom, left), and whether the
struct gfx.St  ## The inherited text style.
    color int
    size float
    bold bool
    italic bool
    underline bool
    mono bool
    pre bool
    align str
    lh float
    href str
    upper bool
    nowrap bool
struct gfx.Bx  ## A block's own box: margins, borders, padding, background and width.
    bg int?
    mt float
    mr float
    mb float
    ml float
    pt float
    pr float
    pb float
    pl float
    bt float
    br float
    bb float
    bl float
    bcolor int
    width float?
    max_width float?
    ml_auto bool
    mr_auto bool
    fn gfx.Bx.boxed(self) bool
    fn gfx.Bx.padded(self) bool
gfx.LINK := 238
gfx.INK := 0
gfx.RULE := 10132122
fn gfx.border_part(v str, size float, ink int) (float, int)  ## `border` and `border-*` values: width (default 3px for a named style)
fn gfx.size_keyword(v str) float?  ## The font-size keywords and their sizes in pixels.
gfx.HEADING_SIZE := ...
fn gfx.text_style(tag str, cs Map[str, str], parent gfx.St) gfx.St  ## The style of text inside element `tag` with the cascaded values `cs`,
fn gfx.box_of(tag str, cs Map[str, str], st gfx.St, avail float) gfx.Bx  ## The box of element `id`, from its tag defaults and cascaded values.
struct gfx.Atom  ## One piece of an inline run: a word, a space, a line break, an image or
    kind int
    text str
    w float
    h float
    size float
    st gfx.St
    src str
    fill int  ## The look of a control: fill, edge and text colors.
    edge int
gfx.WORD := 0
gfx.SPACE := 1
gfx.BREAK := 2
gfx.IMAGE := 3
gfx.CONTROL := 4
struct gfx.Ctx
    d html.Doc
    sheet css.Sheet
    url str
    ops List[draw.Op]
    hits List[gfx.Hit]
    atoms List[gfx.Atom]
    y float  ## The next line starts here, and the margin still owed above it.
    pend float
    marker str  ## The list marker waiting for its first line, and its right edge.
    marker_x float
    canvas int
    fn gfx.Ctx.visible(self, id int) bool
    fn gfx.Ctx.element_style(self, id int) Map[str, str]
    fn gfx.Ctx.is_block(self, id int, cs Map[str, str]) bool
    fn gfx.Ctx.settle(mut self) [mutates]  ## Moves the pending margin into the flow.
    fn gfx.Ctx.add_text(mut self, raw str, st gfx.St) [mutates]
    fn gfx.Ctx.word_atom(self, text str, st gfx.St) gfx.Atom
    fn gfx.Ctx.space_atom(self, st gfx.St) gfx.Atom
    fn gfx.Ctx.break_atom(self, st gfx.St) gfx.Atom
    fn gfx.Ctx.flush(mut self, x float, w float, align str) [mutates]  ## Lays out the pending inline atoms as lines in the column from `x`, `w`
    fn gfx.Ctx.has_content(self, line List[gfx.Atom]) bool
    fn gfx.Ctx.emit(mut self, atoms_in List[gfx.Atom], x float, w float, align str, empty float) [mutates]  ## Draws one line of atoms at `y`; `empty` is the height of an empty
    fn gfx.Ctx.image_atom(self, id int, st gfx.St, avail float) gfx.Atom?
    fn gfx.Ctx.control_atom(self, id int, st gfx.St) gfx.Atom?
    fn gfx.Ctx.inline(mut self, id int, parent gfx.St, avail float) [mutates]  ## Adds element `id`'s inline content to the pending atoms.
    fn gfx.Ctx.children(mut self, id int, st gfx.St, x float, w float) [mutates]  ## Lays out the children of element `id` (style `st`) in the column from
    fn gfx.Ctx.intrinsic(self, id int, st gfx.St) float  ## The width of the text in `id` on one line, for shrink-to-fit items.
    fn gfx.Ctx.block(mut self, id int, parent gfx.St, x float, w float) [mutates]  ## Lays out block `id` inside a column from `x`, `w` wide, with the
    fn gfx.Ctx.li_marker(mut self, id int, st gfx.St, x float) [mutates]  ## Sets the marker of list item `id` whose content starts at `x`.
    fn gfx.Ctx.items(self, id int) List[int]  ## The child elements that take part in a row, column or grid layout.
    fn gfx.Ctx.row(mut self, items List[int], parent gfx.St, x float, widths List[float], offsets List[float]) [mutates]  ## Lays out `items` side by side in columns of `widths` starting at `x`
    fn gfx.Ctx.flex(mut self, id int, st gfx.St, cs Map[str, str], x float, w float) [mutates]
    fn gfx.Ctx.grid(mut self, id int, st gfx.St, cs Map[str, str], x float, w float) [mutates]
    fn gfx.Ctx.table(mut self, id int, st gfx.St, x float, w float) [mutates]
    fn gfx.Ctx.collect_rows(self, id int, mut out List[int]) [mutates]
gfx.CONTROLS := ...
gfx.FLOW_FLEX := ...
pub fn gfx.layout(d html.Doc, url str, width float, sheet css.Sheet) gfx.Scene  ## Lays `d` out in a column `width` pixels wide with `sheet` (the linked
```
