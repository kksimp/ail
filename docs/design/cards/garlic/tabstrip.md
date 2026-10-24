# garlic: tabstrip
tests: tabstrip.tabstrip.an_open_menu_takes_the_clicks_below_the_bar, tabstrip.tabstrip.clicks_land_on_the_tab_its_x_and_the_plus, tabstrip.tabstrip.many_tabs_shrink_to_fit_the_strip, tabstrip.tabstrip.the_menu_rows_depend_on_the_tab_and_what_was_closed

## tabstrip
```
pub struct tabstrip.Item  ## One tab as the strip shows it.
    pub title str
    pub url str
pub struct tabstrip.Theme  ## The colors of the strip; a theme is one of these.
    pub bar int
    pub tab int
    pub active int
    pub text int
    pub active_text int
    pub accent int
    pub menu int
    pub menu_text int
    pub menu_hot int
pub fn tabstrip.dark() tabstrip.Theme
pub tabstrip.BAR_H := 36.0  ## The strip's height without a menu, a menu row's height and the menu's width.
pub tabstrip.ROW_H := 26.0
pub tabstrip.MENU_W := 210.0
tabstrip.MAX_TAB_W := 220.0
tabstrip.MIN_TAB_W := 60.0
tabstrip.GAP := 2.0
tabstrip.LEFT := 8.0
tabstrip.PLUS_W := 30.0
tabstrip.CLOSE_W := 20.0
pub struct tabstrip.Hit  ## What a click landed on: `kind` is `tab`, `close`, `plus`, `menu` (row `k`
    pub kind str
    pub k int
fn tabstrip.miss() tabstrip.Hit
pub fn tabstrip.tab_w(n int, width float) float  ## A tab's width when `n` tabs share a strip `width` wide.
pub fn tabstrip.tab_x(k int, n int, width float) float
pub fn tabstrip.menu_rows(k int, n int, closed int) List[str]  ## The rows of the menu for tab `k` of `n`, `closed` being how many tabs
pub fn tabstrip.height(menu_rows_n int) float  ## The height the canvas needs: the bar, plus the open menu below it.
pub fn tabstrip.hit(x float, y float, n int, width float, menu_n int, menu_x float) tabstrip.Hit  ## What a click at (x, y) landed on. `menu_n` rows are open at `menu_x`.
fn tabstrip.text(x float, y float, s str, fg int, bold bool, size float) std.app.Op
pub fn tabstrip.draw(th tabstrip.Theme, items List[tabstrip.Item], cur int, width float, menu List[str], menu_x float) List[std.app.Op]  ## The display list: the tabs (the shown one lifted, with an accent line), a
```
