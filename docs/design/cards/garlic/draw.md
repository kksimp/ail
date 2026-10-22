# garlic: draw

## draw
```
pub draw.BG := 1973790
pub draw.FG := 13948116
pub draw.QUIET_FG := 8750469
pub draw.LINK_FG := 5676246
pub draw.FOCUS_BG := 2510712
pub draw.HEADING_FG := 14474410
pub draw.CODE_FG := 13537656
pub draw.CONTROL_FG := 5163440
pub draw.STATUS_BG := 31436
pub draw.STATUS_FG := 16777215
pub enum draw.Op  ## One drawing operation. Later operations paint over earlier ones.
    Rect(x float, y float, w float, h float, color int)
    Text(x float, y float, text str, fg int, bold bool, italic bool, underline bool)
    Str(x float, y float, text str, font str, size float, fg int, bold bool, italic bool, underline bool)  ## Proportional text from the pixel layout (`gfx`), `font` "sans" or "mono".
    Pic(x float, y float, w float, h float, src str)  ## An image, drawn from a URL or file path.
pub fn draw.color(st page.Style) int  ## The text color of a run in style `st`.
pub fn draw.display_list(pg page.Page, scroll int, rows int, cols int, focus int, cell_w float, cell_h float) List[draw.Op]  ## The operations that draw `rows` rows of page `pg` from row `scroll`, in
pub fn draw.cell_at(x float, y float, cell_w float, cell_h float) (int, int)  ## The cell under point (`x`, `y`) of the page area, as (col, row).
pub fn draw.grid_size(w float, h float, cell_w float, cell_h float) (int, int)  ## The columns and rows that fit an area `w` by `h` points, at least 1 by 1.
pub fn draw.shift(ops List[draw.Op], bg int, w float, h float, top float) List[draw.Op]  ## The part of page-pixel operations `ops` that shows in an area `w` by `h`
```
