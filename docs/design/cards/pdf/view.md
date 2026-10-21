# pdf: view
tests: view.view.finds_and_steps_through_hits, view.view.lays_text_out_at_its_positions, view.view.types_a_query

## view
```
view.PAPER_BG := 16052714
view.PAPER_FG := 2039583
view.EDGE_BG := 3815994
view.BAR_BG := 31436
view.BAR_FG := 16777215
view.HIT_BG := 16766287
view.CUR_BG := 16748288
view.LINK_FG := 1728436
pub struct view.Layout  ## One laid-out page: text rows, and for each placed character which line
    pub rows List[str]
    cells List[List[List[int]]]
    pub width int
    link_rows Map[int, bool]
pub struct view.Viewer
    pub path str
    pages reader.Pages
    reader_ocr ocr.Ocr  ## Scanned pages are read by OCR when shown or searched.
    pub page int
    pub top int
    pub cols int
    pub rows int
    pub query str
    pub typing bool
    pub hits List[reader.Hit]
    pub hit int
    pub message str
    pub quit bool
    layouts Map[int, view.Layout]
    pub fn view.Viewer.new(path str, pages reader.Pages, cols int, rows int) view.Viewer
    fn view.Viewer.layout_of(mut self, i int) view.Layout [mutates]
    fn view.Viewer.body_rows(self) int  ## Rows the page area has: the window less the title and status rows.
    pub fn view.Viewer.find(mut self, q str) [mutates]  ## Searches for `q` from the current page on, reading pages only until
    fn view.Viewer.seek(mut self, from int, dir int, last bool) bool [mutates]  ## The first page with a match from page `from`, stepping `dir` and
    fn view.Viewer.step_hit(mut self, dir int) [mutates]  ## The next (`dir` 1) or previous (-1) match, moving to the next page with
    fn view.Viewer.go_hit(mut self, i int) [mutates]
    fn view.Viewer.go_page(mut self, n int) [mutates]
    pub fn view.Viewer.on_key(mut self, key str, ctrl bool, shift bool) [mutates]  ## One key, as the window sends it (std.ui.4): `key` is a name such as
    pub fn view.Viewer.scroll(mut self, n int) [mutates]  ## Scroll by `n` rows, from the mouse wheel.
    fn view.Viewer.hit_at(self, pi int, li int, ci int) int  ## Whether page `pi`'s character `ci` of line `li` is in a hit; 2 for the
    pub fn view.Viewer.render(mut self) List[std.ui.widgets.Run] [mutates]  ## The screen as runs: a title bar, the page on paper, and a status bar.
    pub fn view.Viewer.state_json(mut self) Json [mutates]  ## What is on screen, for an agent: the page, the scroll, the query and
    fn view.Viewer.visible_lines(self, l view.Layout) List[int]  ## The page's lines with a character on screen, in page order: the words a
pub fn view.with_ocr(path str, pages reader.Pages, cols int, rows int, o ocr.Ocr) view.Viewer
fn view.median(xs List[float]) float  ## The median of a non-empty list.
pub fn view.layout(p reader.Page, cols int) view.Layout  ## Lays a page out for `cols` columns: one cell per typical character
pub fn view.natural_width(p reader.Page) int  ## The columns a page needs for one cell per typical character.
fn view.overlaps(spans List[List[int]], a int, b int) bool
fn view.row_of(l view.Layout, line int) int
fn view.bar(row int, text str, cols int) std.ui.widgets.Run
fn view.base(path str) str
fn view.press(spec str) List[str]
pub fn view.start(path str, pages reader.Pages, flags Map[str, str]) [io ui]  ## The viewer from the command line: `--page N`, `--find TEXT`, then
fn view.window(mut v view.Viewer) [io mutates ui]
fn view.sample() view.Viewer
```
